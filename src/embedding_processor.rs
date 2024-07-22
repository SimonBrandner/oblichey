use crate::frame_processor::Face;
use std::fmt::Debug;

pub trait EmbeddingProcessor: Debug {
	fn process_embeddings(&self, faces: &[Face]);
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

impl EmbeddingProcessor for ScanProcessor {
	fn process_embeddings(&self, faces: &[Face]) {}

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

impl EmbeddingProcessor for AuthProcessor {
	fn process_embeddings(&self, faces: &[Face]) {}

	fn is_finished(&self) -> bool {
		self.result.is_some()
	}
}
