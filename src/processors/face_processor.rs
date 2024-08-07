use super::{FaceEmbedding, FaceForGUI, FaceForGUIAnnotation, FaceForProcessing};
use crate::processors::FaceRecognitionError;
use std::{collections::HashMap, fmt::Debug, time::Instant};

const AUTH_TIMEOUT: u64 = 15; // In seconds
const SIMILARITY_THRESHOLD: f32 = 0.51;
const SCAN_SAMPLE_COUNT: usize = 16;

pub trait FaceProcessor: Debug {
	fn process_faces(&mut self, detected_faces: Vec<FaceForProcessing>) -> Vec<FaceForGUI>;
	fn is_finished(&self) -> bool;
}

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
	fn is_finished(&self) -> bool {
		self.result.is_some()
	}

	fn process_faces(&mut self, faces_for_processing: Vec<FaceForProcessing>) -> Vec<FaceForGUI> {
		// Handle edge-cases
		if faces_for_processing.len() > 1 {
			self.embedding_samples.clear();
			return faces_for_processing
				.into_iter()
				.map(|f| FaceForGUI {
					rectangle: f.rectangle,
					annotation: super::FaceForGUIAnnotation::Warning(String::from(
						"Too many faces for scanning",
					)),
				})
				.collect();
		};
		let face_for_processing = match faces_for_processing.get(0) {
			Some(f) => f,
			None => {
				self.embedding_samples.clear();
				return vec![];
			}
		};
		let embedding = match face_for_processing.face_data {
			Ok(f) => f.embedding,
			Err(e) => match e {
				FaceRecognitionError::TooSmall => {
					self.embedding_samples.clear();
					return vec![FaceForGUI {
						rectangle: face_for_processing.rectangle,
						annotation: FaceForGUIAnnotation::Warning(String::from("Too small")),
					}];
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

		// Return info to be display in the GUI
		vec![FaceForGUI {
			rectangle: face_for_processing.rectangle,
			annotation: super::FaceForGUIAnnotation::ScanningState {
				scanned_sample_count: self.embedding_samples.len(),
				required_sample_count: SCAN_SAMPLE_COUNT,
			},
		}]
	}
}

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

	pub fn get_result(&self) -> Option<AuthProcessorResult> {
		self.result
	}

	fn process_face(&self, face_for_processing: &FaceForProcessing) -> FaceForGUI {
		let face_data = match face_for_processing.face_data {
			Ok(d) => d,
			Err(e) => match e {
				FaceRecognitionError::TooSmall => {
					return FaceForGUI {
						rectangle: face_for_processing.rectangle,
						annotation: FaceForGUIAnnotation::Warning("Too small".to_owned()),
					}
				}
			},
		};

		let mut best_match: Option<(String, f32)> = None;
		for (stored_face_embedding_name, stored_face_embedding) in &self.stored_face_embeddings {
			let similarity = face_data
				.embedding
				.cosine_similarity(&stored_face_embedding);
			if similarity < SIMILARITY_THRESHOLD {
				continue;
			}
			match best_match {
				Some((_, best_match_similarity)) => {
					if similarity > best_match_similarity {
						best_match = Some((stored_face_embedding_name.clone(), similarity))
					}
				}
				None => best_match = Some((stored_face_embedding_name.clone(), similarity)),
			}
		}

		FaceForGUI {
			rectangle: face_for_processing.rectangle,
			annotation: match best_match {
				Some((name, _)) => FaceForGUIAnnotation::Name(name),
				None => FaceForGUIAnnotation::Warning("Not recognized".to_owned()),
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
			})
		}

		let mut faces_for_gui = Vec::new();
		for face_for_processing in faces_for_processing {
			let face_for_gui = self.process_face(&face_for_processing);
			if !self.testing_mode {
				if let FaceForGUIAnnotation::Name(_) = face_for_gui.annotation {
					self.result = Some(AuthProcessorResult {
						authenticated: true,
					})
				}
			}

			faces_for_gui.push(face_for_gui);
		}

		faces_for_gui
	}
}
