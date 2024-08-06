use crate::processors::FaceRecognitionError;

use super::{FaceEmbedding, FaceForGUI, FaceForGUIAnnotation, FaceForProcessing};
use std::fmt::Debug;

const SIMILARITY_THRESHOLD: f32 = 0.51;
const SCAN_SAMPLE_COUNT: usize = 16;

#[derive(Clone, Debug)]
pub enum FaceProcessorOutput {
	Finished,
	State(Vec<FaceForGUI>),
}

pub trait FaceProcessor: Debug {
	fn process_detected_faces(
		&mut self,
		detected_faces: Vec<FaceForProcessing>,
	) -> FaceProcessorOutput;
}

#[derive(Debug, Clone)]
pub struct ScanProcessorResult {
	face_embedding: FaceEmbedding,
}

#[derive(Debug)]
pub struct ScanProcessor {
	result: Option<ScanProcessorResult>,
	embedding_samples: Vec<FaceEmbedding>,
}

impl ScanProcessor {
	pub fn new() -> Self {
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
	fn process_detected_faces(
		&mut self,
		faces_for_processing: Vec<FaceForProcessing>,
	) -> FaceProcessorOutput {
		// Handle edge-cases
		if self.result.is_some() {
			return FaceProcessorOutput::Finished;
		}
		if faces_for_processing.len() > 1 {
			self.embedding_samples.clear();
			return FaceProcessorOutput::State(
				faces_for_processing
					.into_iter()
					.map(|f| FaceForGUI {
						rectangle: f.rectangle,
						annotation: super::FaceForGUIAnnotation::Warning(String::from(
							"Too many faces for scanning",
						)),
					})
					.collect(),
			);
		};
		let face_for_processing = match faces_for_processing.get(0) {
			Some(f) => f,
			None => {
				self.embedding_samples.clear();
				return FaceProcessorOutput::State(vec![]);
			}
		};
		let embedding = match face_for_processing.face_data {
			Ok(f) => f.embedding,
			Err(e) => match e {
				FaceRecognitionError::TooSmall => {
					self.embedding_samples.clear();
					return FaceProcessorOutput::State(vec![FaceForGUI {
						rectangle: face_for_processing.rectangle,
						annotation: FaceForGUIAnnotation::Warning(String::from("Too small")),
					}]);
				}
			},
		};

		// We require multiple samples during the scan for safety purposes. All of these samples
		// have to satisfy the similarity requirement. To avoid having to check each two samples we
		// take and an average of all of them. We calculate the average in each run of this
		// function to give all samples the same weight
		if self.embedding_samples.len() > 0 {
			let average_embedding = FaceEmbedding::average_embedding(&self.embedding_samples);
			let similarity = average_embedding.cosine_similarity(&embedding);

			if similarity < SIMILARITY_THRESHOLD {
				self.embedding_samples.clear();
				return FaceProcessorOutput::State(vec![]);
			}
		}
		self.embedding_samples.push(embedding);

		// If we have enough samples, we consider the scan to be successful, so we set the result
		if self.embedding_samples.len() > SCAN_SAMPLE_COUNT {
			self.result = Some(ScanProcessorResult {
				face_embedding: FaceEmbedding::average_embedding(&self.embedding_samples),
			});
			return FaceProcessorOutput::Finished;
		}

		// Return info to be display in the GUI
		FaceProcessorOutput::State(vec![FaceForGUI {
			rectangle: face_for_processing.rectangle,
			annotation: super::FaceForGUIAnnotation::ScanningState {
				scanned_sample_count: self.embedding_samples.len(),
				required_sample_count: SCAN_SAMPLE_COUNT,
			},
		}])
	}
}

#[derive(Debug, Clone)]
pub struct AuthProcessorResult;

#[derive(Debug)]
pub struct AuthProcessor {
	result: Option<AuthProcessorResult>,
	finish_when_authed: bool,
}

impl AuthProcessor {
	pub fn new(finish_when_authed: bool) -> Self {
		Self {
			result: None,
			finish_when_authed,
		}
	}

	pub fn get_result(&self) -> Option<AuthProcessorResult> {
		self.result.clone()
	}
}

impl FaceProcessor for AuthProcessor {
	fn process_detected_faces(
		&mut self,
		detected_faces: Vec<FaceForProcessing>,
	) -> FaceProcessorOutput {
		FaceProcessorOutput::State(vec![])
	}
}
