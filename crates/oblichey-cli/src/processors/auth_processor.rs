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
			let similarity = face_data.embedding.cosine_similarity(stored_face_embedding);
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
