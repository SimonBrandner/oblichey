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
