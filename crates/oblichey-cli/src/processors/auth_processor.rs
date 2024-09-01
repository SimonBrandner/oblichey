use super::{
	face::{FaceEmbedding, FaceForGUIAnnotationWarning, FaceForProcessing, SIMILARITY_THRESHOLD},
	face_processor::FaceProcessor,
	FaceForGUI,
};
use crate::processors::face::{FaceForGUIAnnotation, FaceRecognitionError};
use std::{collections::HashMap, fmt::Debug, time::Instant};

const AUTH_TIMEOUT: u64 = 10; // In seconds

#[derive(Debug, Clone, Copy)]
pub struct AuthProcessorResult {
	pub authenticated: bool,
}

#[derive(Debug)]
pub struct AuthProcessor {
	result: Option<AuthProcessorResult>,
	stored_face_embeddings: HashMap<String, FaceEmbedding>,
	start_time: Instant,
	testing_mode: bool,
}

impl AuthProcessor {
	pub fn new(face_embeddings: HashMap<String, FaceEmbedding>, testing_mode: bool) -> Self {
		Self {
			result: None,
			stored_face_embeddings: face_embeddings,
			testing_mode,
			start_time: Instant::now(),
		}
	}

	pub const fn get_result(&self) -> Option<AuthProcessorResult> {
		self.result
	}

	fn process_face(&self, face_for_processing: &FaceForProcessing) -> FaceForGUI {
		let face_data = match face_for_processing.face_data {
			Ok(d) => d,
			Err(e) => match e {
				FaceRecognitionError::TooSmall => {
					return FaceForGUI {
						rectangle: face_for_processing.rectangle,
						annotation: FaceForGUIAnnotation::Warning(
							FaceForGUIAnnotationWarning::TooSmall,
						),
					}
				}
			},
		};

		let mut best_match: Option<(String, f32)> = None;
		for (stored_face_embedding_name, stored_face_embedding) in &self.stored_face_embeddings {
			let similarity = face_data
				.embedding
				.cosine_similarity(stored_face_embedding)
				.expect("Similarity should never be None");
			if similarity < SIMILARITY_THRESHOLD {
				continue;
			}
			match best_match {
				Some((_, best_match_similarity)) => {
					if similarity > best_match_similarity {
						best_match = Some((stored_face_embedding_name.clone(), similarity));
					}
				}
				None => best_match = Some((stored_face_embedding_name.clone(), similarity)),
			}
		}

		FaceForGUI {
			rectangle: face_for_processing.rectangle,
			annotation: match best_match {
				Some((name, _)) => FaceForGUIAnnotation::Name(name),
				None => FaceForGUIAnnotation::Warning(FaceForGUIAnnotationWarning::NotRecognized),
			},
		}
	}

	fn have_timed_out(&self) -> bool {
		if !self.testing_mode && self.start_time.elapsed().as_secs() > AUTH_TIMEOUT {
			return true;
		}

		false
	}
}

impl FaceProcessor for AuthProcessor {
	fn is_finished(&self) -> bool {
		self.result.is_some()
	}

	fn process_faces(&mut self, faces_for_processing: Vec<FaceForProcessing>) -> Vec<FaceForGUI> {
		if self.have_timed_out() {
			self.result = Some(AuthProcessorResult {
				authenticated: false,
			});
		}

		let mut processed_faces = Vec::new();
		for face_for_processing in faces_for_processing {
			let processed_face = self.process_face(&face_for_processing);
			if !self.testing_mode {
				if let FaceForGUIAnnotation::Name(_) = processed_face.annotation {
					self.result = Some(AuthProcessorResult {
						authenticated: true,
					});
				}
			}

			processed_faces.push(processed_face);
		}

		processed_faces
	}
}

#[cfg(test)]
mod tests {
	use super::AuthProcessor;
	use crate::{
		geometry::{Rectangle, Vec2D},
		processors::{
			face::{
				FaceEmbedding, FaceForGUIAnnotation, FaceForGUIAnnotationWarning,
				FaceForProcessing, FaceRecognitionData, FaceRecognitionError, EMBEDDING_LENGTH,
			},
			face_processor::FaceProcessor,
		},
	};
	use core::panic;
	use std::collections::HashMap;

	#[test]
	fn categorizes_faces() {
		let rectangle = Rectangle::new(Vec2D::new(0, 0), Vec2D::new(0, 0));
		let face_name = String::from("alice");
		let embedding = FaceEmbedding::new(&{
			let mut embedding = [0.0; EMBEDDING_LENGTH];
			embedding[0] = 1.0;
			embedding
		});
		let face_embeddings = {
			let mut embeddings = HashMap::new();
			embeddings.insert(face_name.clone(), embedding);
			embeddings
		};
		let mut processor = AuthProcessor::new(face_embeddings, false);

		let result = processor.process_faces(vec![
			FaceForProcessing {
				rectangle,
				face_data: Err(FaceRecognitionError::TooSmall),
			},
			FaceForProcessing {
				rectangle,
				face_data: Ok(FaceRecognitionData {
					embedding: FaceEmbedding::new(&[1.0; EMBEDDING_LENGTH]),
				}),
			},
			FaceForProcessing {
				rectangle,
				face_data: Ok(FaceRecognitionData { embedding }),
			},
		]);

		assert_eq!(result.len(), 3);
		if let FaceForGUIAnnotation::Warning(warning) = &result[0].annotation {
			assert_eq!(*warning, FaceForGUIAnnotationWarning::TooSmall);
		} else {
			panic!()
		}
		if let FaceForGUIAnnotation::Warning(warning) = &result[1].annotation {
			assert_eq!(*warning, FaceForGUIAnnotationWarning::NotRecognized);
		} else {
			panic!()
		}
		if let FaceForGUIAnnotation::Name(name) = &result[2].annotation {
			assert_eq!(*name, face_name);
		} else {
			panic!()
		}
		if let Some(result) = processor.get_result() {
			assert!(result.authenticated);
		} else {
			panic!();
		}
	}

	#[test]
	fn picks_better_match() {
		let rectangle = Rectangle::new(Vec2D::new(0, 0), Vec2D::new(0, 0));
		let face_name = String::from("alice");
		let correct_embedding_data = [1.0; EMBEDDING_LENGTH];
		let correct_embedding = FaceEmbedding::new(&correct_embedding_data);
		let incorrect_embedding_a = FaceEmbedding::new(&{
			let mut embedding = correct_embedding_data;
			embedding[0] = 0.0;
			embedding
		});
		let incorrect_embedding_b = FaceEmbedding::new(&{
			let mut embedding = correct_embedding_data;
			embedding[255] = 0.0;
			embedding
		});

		let face_embeddings = {
			let mut embeddings = HashMap::new();
			embeddings.insert(face_name.clone(), correct_embedding);
			embeddings.insert(String::from("bob"), incorrect_embedding_a);
			embeddings.insert(String::from("charlie"), incorrect_embedding_b);
			embeddings
		};
		let mut processor = AuthProcessor::new(face_embeddings, false);

		let result = processor.process_faces(vec![FaceForProcessing {
			rectangle,
			face_data: Ok(FaceRecognitionData {
				embedding: correct_embedding,
			}),
		}]);

		assert_eq!(result.len(), 1);
		if let FaceForGUIAnnotation::Name(name) = &result[0].annotation {
			assert_eq!(*name, face_name);
		} else {
			panic!()
		}
		if let Some(result) = processor.get_result() {
			assert!(result.authenticated);
		} else {
			panic!();
		}
	}
}
