use crate::geometry::Rectangle;
use num::pow::Pow;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::ops::{AddAssign, Div};

pub const EMBEDDING_LENGTH: usize = 512;
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

	pub fn cosine_similarity(&self, other: &Self) -> Option<f32> {
		let denominator = self.magnitude() * other.magnitude();
		if denominator == 0.0 {
			None
		} else {
			Some(self.dot_product(other) / denominator)
		}
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
	use super::{FaceEmbedding, EMBEDDING_LENGTH};
	use core::f32;

	#[test]
	fn calculates_magnitude() {
		let test_cases = vec![
			([0.0; EMBEDDING_LENGTH], 0.0),
			(
				[[1.0; 256], [0.0; 256]]
					.concat()
					.try_into()
					.expect("Failed to convert to array"),
				16.0,
			),
			(
				[[3.0; 256], [4.0; 256]]
					.concat()
					.try_into()
					.expect("Failed to convert to array"),
				80.0,
			),
			(
				[[6.0; 256], [8.0; 256]]
					.concat()
					.try_into()
					.expect("Failed to convert to array"),
				160.0,
			),
			(
				[[9.0; 256], [12.0; 256]]
					.concat()
					.try_into()
					.expect("Failed to convert to array"),
				240.0,
			),
		];

		for (embedding_data, expected_result) in test_cases {
			let embedding = FaceEmbedding::new(&embedding_data);
			let result = embedding.magnitude();
			assert!((expected_result - result).abs() <= f32::EPSILON);
		}
	}

	#[test]
	fn calculates_dot_product() {
		let test_cases = vec![
			([0.0; EMBEDDING_LENGTH], [0.0; EMBEDDING_LENGTH], 0.0),
			([0.0; EMBEDDING_LENGTH], [1.0; EMBEDDING_LENGTH], 0.0),
			([0.0; EMBEDDING_LENGTH], [10.0; EMBEDDING_LENGTH], 0.0),
			([1.0; EMBEDDING_LENGTH], [1.0; EMBEDDING_LENGTH], 512.0),
			([10.0; EMBEDDING_LENGTH], [1.0; EMBEDDING_LENGTH], 5120.0),
			(
				[[1.0; 256], [3.0; 256]]
					.concat()
					.try_into()
					.expect("Failed to convert to array"),
				[1.0; EMBEDDING_LENGTH],
				1024.0,
			),
		];

		for (embedding_data_a, embedding_data_b, expected_result) in test_cases {
			let embedding_a = FaceEmbedding::new(&embedding_data_a);
			let embedding_b = FaceEmbedding::new(&embedding_data_b);

			let result = embedding_a.dot_product(&embedding_b);
			assert!((expected_result - result).abs() <= f32::EPSILON);
		}
	}

	#[test]
	fn calculates_cosine_similarity() {
		let test_cases = vec![
			([0.0; EMBEDDING_LENGTH], [0.0; EMBEDDING_LENGTH], None),
			([1.0; EMBEDDING_LENGTH], [0.0; EMBEDDING_LENGTH], None),
			([1.0; EMBEDDING_LENGTH], [1.0; EMBEDDING_LENGTH], Some(1.0)),
			([1.0; EMBEDDING_LENGTH], [2.0; EMBEDDING_LENGTH], Some(1.0)),
			(
				{
					let mut embedding = [0.0; EMBEDDING_LENGTH];
					embedding[0] = 1.0;
					embedding
				},
				{
					let mut embedding = [0.0; EMBEDDING_LENGTH];
					embedding[255] = 1.0;
					embedding
				},
				Some(0.0),
			),
		];

		for (embedding_data_a, embedding_data_b, expected_result) in test_cases {
			let embedding_a = FaceEmbedding::new(&embedding_data_a);
			let embedding_b = FaceEmbedding::new(&embedding_data_b);

			let result = embedding_a.cosine_similarity(&embedding_b);
			match (result, expected_result) {
				(Some(result), Some(expected_result)) => {
					assert!((expected_result - result).abs() <= f32::EPSILON);
				}
				(None, None) => {}
				_ => panic!(),
			}
		}
	}

	#[test]
	fn calculates_average_embedding() {
		let test_cases = vec![
			(vec![[1.0; EMBEDDING_LENGTH]], [1.0; EMBEDDING_LENGTH]),
			(
				vec![
					[[1.0; 256], [3.0; 256]]
						.concat()
						.try_into()
						.expect("Failed to create embedding"),
					[[3.0; 256], [1.0; 256]]
						.concat()
						.try_into()
						.expect("Failed to create embedding"),
				],
				[2.0; EMBEDDING_LENGTH],
			),
			(
				vec![
					[[1.0; 256], [3.0; 256]]
						.concat()
						.try_into()
						.expect("Failed to create embedding"),
					[[1.0; 256], [1.0; 256]]
						.concat()
						.try_into()
						.expect("Failed to create embedding"),
				],
				[[1.0; 256], [2.0; 256]]
					.concat()
					.try_into()
					.expect("Failed to create embedding"),
			),
		];

		for (embeddings_data, expected_data) in test_cases {
			let embeddings: Vec<FaceEmbedding> =
				embeddings_data.iter().map(FaceEmbedding::new).collect();
			let average_embedding = FaceEmbedding::average_embedding(&embeddings);
			let expected_embedding = FaceEmbedding::new(&expected_data);

			let similarity = average_embedding
				.cosine_similarity(&expected_embedding)
				.expect("Failed to calculate cosine similarity");
			assert!((similarity - 1.0) <= f32::EPSILON);

			let magnitude_difference =
				(average_embedding.magnitude() - expected_embedding.magnitude()).abs();
			assert!((magnitude_difference) <= f32::EPSILON);
		}
	}
}
