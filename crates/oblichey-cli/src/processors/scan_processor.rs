use super::{
	face::{FaceEmbedding, FaceForGUIAnnotationWarning, FaceForProcessing},
	face_processor::FaceProcessor,
	FaceForGUI,
};
use crate::processors::face::{FaceForGUIAnnotation, FaceRecognitionError, SIMILARITY_THRESHOLD};
use std::fmt::Debug;

const SCAN_SAMPLE_COUNT: usize = 16;

#[derive(Debug, Clone)]
pub struct ScanProcessorResult {
	pub face_embedding: FaceEmbedding,
}

#[derive(Debug)]
pub struct ScanProcessor {
	result: Option<ScanProcessorResult>,
	embedding_samples: Vec<FaceEmbedding>,
}

impl ScanProcessor {
	pub const fn new() -> Self {
		Self {
			result: None,
			embedding_samples: Vec::new(),
		}
	}

	pub fn get_result(&self) -> Option<ScanProcessorResult> {
		self.result.clone()
	}
}

impl FaceProcessor for ScanProcessor {
	fn is_finished(&self) -> bool {
		self.result.is_some()
	}

	fn process_faces(&mut self, faces: Vec<FaceForProcessing>) -> Vec<FaceForGUI> {
		// Handle edge-cases
		if faces.len() > 1 {
			self.embedding_samples.clear();
			return faces
				.into_iter()
				.map(|f| FaceForGUI {
					rectangle: f.rectangle,
					annotation: FaceForGUIAnnotation::Warning(
						FaceForGUIAnnotationWarning::TooManyFaces,
					),
				})
				.collect();
		};
		let Some(face) = faces.first() else {
			self.embedding_samples.clear();
			return vec![];
		};
		let embedding = match face.face_data {
			Ok(f) => f.embedding,
			Err(e) => match e {
				FaceRecognitionError::TooSmall => {
					self.embedding_samples.clear();
					return vec![FaceForGUI {
						rectangle: face.rectangle,
						annotation: FaceForGUIAnnotation::Warning(
							FaceForGUIAnnotationWarning::TooSmall,
						),
					}];
				}
			},
		};

		// We require multiple samples during the scan for safety purposes. All of these samples
		// have to satisfy the similarity requirement. To avoid having to check each two samples we
		// take and an average of all of them. We calculate the average in each run of this
		// function to give all samples the same weight
		if !self.embedding_samples.is_empty() {
			let average_embedding = FaceEmbedding::average_embedding(&self.embedding_samples);
			let similarity = average_embedding
				.cosine_similarity(&embedding)
				.expect("Similarity should never be None");

			if similarity < SIMILARITY_THRESHOLD {
				self.embedding_samples.clear();
				return vec![];
			}
		}
		self.embedding_samples.push(embedding);

		// If we have enough samples, we consider the scan to be successful, so we set the result
		if self.embedding_samples.len() > SCAN_SAMPLE_COUNT {
			self.result = Some(ScanProcessorResult {
				face_embedding: FaceEmbedding::average_embedding(&self.embedding_samples),
			});
		}

		// Return info to be displayed in the GUI
		vec![FaceForGUI {
			rectangle: face.rectangle,
			annotation: FaceForGUIAnnotation::ScanningState {
				scanned_sample_count: self.embedding_samples.len(),
				required_sample_count: SCAN_SAMPLE_COUNT,
			},
		}]
	}
}

#[cfg(test)]
mod tests {
	use super::ScanProcessor;
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

	#[test]
	fn handles_too_many_faces() {
		let rectangle = Rectangle::new(Vec2D::new(0, 0), Vec2D::new(0, 0));
		let mut processor = ScanProcessor::new();

		let result = processor.process_faces(vec![
			FaceForProcessing {
				rectangle,
				face_data: Ok(FaceRecognitionData {
					embedding: FaceEmbedding::new(&[0.0; EMBEDDING_LENGTH]),
				}),
			},
			FaceForProcessing {
				rectangle,
				face_data: Ok(FaceRecognitionData {
					embedding: FaceEmbedding::new(&[0.0; EMBEDDING_LENGTH]),
				}),
			},
		]);

		assert_eq!(result.len(), 2);
		for face in result {
			if let FaceForGUIAnnotation::Warning(w) = face.annotation {
				assert_eq!(w, FaceForGUIAnnotationWarning::TooManyFaces);
			} else {
				panic!()
			}
		}
		assert!(processor.get_result().is_none());
	}

	#[test]
	fn scans_face() {
		let rectangle = Rectangle::new(Vec2D::new(0, 0), Vec2D::new(0, 0));
		let embedding = FaceEmbedding::new(&[1.0; EMBEDDING_LENGTH]);
		let faces = vec![FaceForProcessing {
			rectangle,
			face_data: Ok(FaceRecognitionData { embedding }),
		}];
		let mut processor = ScanProcessor::new();

		let mut i = 1;
		while !processor.is_finished() {
			assert!(i <= 17);

			let result = processor.process_faces(faces.clone());
			assert_eq!(result.len(), 1);
			if let FaceForGUIAnnotation::ScanningState {
				scanned_sample_count,
				required_sample_count: _,
			} = result[0].annotation
			{
				assert_eq!(scanned_sample_count, i);
			} else {
				panic!();
			}
			i += 1;
		}

		let result = processor.get_result().expect("Failed to get result");
		let similarity = result
			.face_embedding
			.cosine_similarity(&embedding)
			.expect("Failed to calculate similarity");
		assert!((similarity - 1.0).abs() <= f32::EPSILON);
	}

	#[test]
	fn resets_scanning() {
		let rectangle = Rectangle::new(Vec2D::new(0, 0), Vec2D::new(0, 0));
		let face = FaceForProcessing {
			rectangle,
			face_data: Ok(FaceRecognitionData {
				embedding: FaceEmbedding::new(&[1.0; EMBEDDING_LENGTH]),
			}),
		};
		let mut processor = ScanProcessor::new();

		// Different face
		start_over_scanning(&mut processor);
		processor.process_faces(vec![FaceForProcessing {
			rectangle: Rectangle::new(Vec2D::new(0, 0), Vec2D::new(0, 0)),
			face_data: Ok(FaceRecognitionData {
				embedding: FaceEmbedding::new(&{
					let mut embedding = [0.0; EMBEDDING_LENGTH];
					embedding[0] = 1.0;
					embedding
				}),
			}),
		}]);
		assert_eq!(processor.embedding_samples.len(), 0);

		// Too many faces
		start_over_scanning(&mut processor);
		processor.process_faces(vec![face.clone(), face]);
		assert_eq!(processor.embedding_samples.len(), 0);

		// Too small
		start_over_scanning(&mut processor);
		processor.process_faces(vec![FaceForProcessing {
			rectangle: Rectangle::new(Vec2D::new(0, 0), Vec2D::new(0, 0)),
			face_data: Err(FaceRecognitionError::TooSmall),
		}]);
		assert_eq!(processor.embedding_samples.len(), 0);

		// No face
		start_over_scanning(&mut processor);
		processor.process_faces(vec![]);
		assert_eq!(processor.embedding_samples.len(), 0);
	}

	fn start_over_scanning(processor: &mut ScanProcessor) {
		processor.process_faces(vec![FaceForProcessing {
			rectangle: Rectangle::new(Vec2D::new(0, 0), Vec2D::new(0, 0)),
			face_data: Ok(FaceRecognitionData {
				embedding: FaceEmbedding::new(&[1.0; EMBEDDING_LENGTH]),
			}),
		}]);
		assert_eq!(processor.embedding_samples.len(), 1);
	}
}
