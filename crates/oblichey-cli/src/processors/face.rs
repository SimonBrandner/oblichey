use crate::geometry::Rectangle;
use num::pow::Pow;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::ops::{AddAssign, Div};

const EMBEDDING_LENGTH: usize = 512;
pub const SIMILARITY_THRESHOLD: f32 = 0.51;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FaceRecognitionError {
	/// The detected face was too small for running the recognition model on it
	TooSmall,
}

/// This is the face embedding data itself
pub type FaceEmbeddingData = [f32; EMBEDDING_LENGTH];

/// This is a struct for working with the face embedding. It supports several useful operations
#[serde_as]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FaceEmbedding {
	#[serde_as(as = "[_; EMBEDDING_LENGTH]")]
	data: FaceEmbeddingData,
}

impl Default for FaceEmbedding {
	fn default() -> Self {
		Self {
			data: [0.0; EMBEDDING_LENGTH],
		}
	}
}

impl AddAssign for FaceEmbedding {
	fn add_assign(&mut self, rhs: Self) {
		for index in 0..EMBEDDING_LENGTH {
			self.data[index] += rhs.data[index];
		}
	}
}

impl Div<f32> for FaceEmbedding {
	type Output = Self;

	fn div(self, rhs: f32) -> Self::Output {
		let mut out = Self::default();
		for index in 0..EMBEDDING_LENGTH {
			out.data[index] = self.data[index] / rhs;
		}

		out
	}
}

impl FaceEmbedding {
	pub const fn new(data: &FaceEmbeddingData) -> Self {
		Self { data: *data }
	}

	pub fn magnitude(&self) -> f32 {
		let mut sum: f32 = 0.0;
		for element in self.data {
			sum += element.pow(2);
		}

		sum.sqrt()
	}

	pub fn dot_product(&self, other: &Self) -> f32 {
		let mut dot_product = 0.0;
		for index in 0..EMBEDDING_LENGTH {
			dot_product += self.data[index] * other.data[index];
		}

		dot_product
	}

	pub fn cosine_similarity(&self, other: &Self) -> f32 {
		self.dot_product(other) / (self.magnitude() * other.magnitude())
	}

	pub fn average_embedding(embeddings: &[Self]) -> Self {
		let mut sum_embedding = Self::default();

		for embedding in embeddings {
			sum_embedding += *embedding;
		}

		sum_embedding / embeddings.len() as f32
	}
}

/// This is the data produced by the recognition model and in the future by other models (such as
/// those for liveness detection)
#[derive(Debug, Clone, Copy)]
pub struct FaceRecognitionData {
	pub embedding: FaceEmbedding,
}

/// This is a struct that fully describes a face in a frame with its location, size and the
/// recognition data
#[derive(Debug, Clone)]
pub struct FaceForProcessing {
	pub rectangle: Rectangle<u32>,
	pub face_data: Result<FaceRecognitionData, FaceRecognitionError>,
}

#[derive(Clone, Debug)]
pub enum FaceForGUIAnnotationWarning {
	NotRecognized,
	TooSmall,
}

#[derive(Clone, Debug)]
pub enum FaceForGUIAnnotation {
	Name(String),
	Warning(FaceForGUIAnnotationWarning),
	ScanningState {
		scanned_sample_count: usize,
		required_sample_count: usize,
	},
}

/// Face for displaying in the GUI
#[derive(Debug, Clone)]
pub struct FaceForGUI {
	pub rectangle: Rectangle<u32>,
	pub annotation: FaceForGUIAnnotation,
}
