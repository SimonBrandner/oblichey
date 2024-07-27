use super::DetectedFace;
use std::fmt::Debug;

pub trait FaceProcessor: Debug {
	fn process_detected_faces(&self, detected_faces: &[DetectedFace]);
	fn is_finished(&self) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct ScanProcessorResult {}

#[derive(Debug)]
pub struct ScanProcessor {
	result: Option<ScanProcessorResult>,
}

impl ScanProcessor {
	pub fn new() -> Self {
		Self { result: None }
	}

	pub fn get_result(&self) -> Option<ScanProcessorResult> {
		self.result.clone()
	}
}

impl FaceProcessor for ScanProcessor {
	fn process_detected_faces(&self, detected_faces: &[DetectedFace]) {}

	fn is_finished(&self) -> bool {
		self.result.is_some()
	}
}

#[derive(Debug, Clone)]
pub struct AuthProcessorResult;

#[derive(Debug)]
pub struct AuthProcessor {
	result: Option<AuthProcessorResult>,
}

impl AuthProcessor {
	pub fn new() -> Self {
		Self { result: None }
	}

	pub fn get_result(&self) -> Option<AuthProcessorResult> {
		self.result.clone()
	}
}

impl FaceProcessor for AuthProcessor {
	fn process_detected_faces(&self, detected_faces: &[DetectedFace]) {}

	fn is_finished(&self) -> bool {
		self.result.is_some()
	}
}
