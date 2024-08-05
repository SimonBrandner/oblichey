pub mod face_processor;
pub mod frame_processor;

use self::{
	face_processor::{FaceProcessor, ScanProcessor},
	frame_processor::FrameProcessor,
};
use crate::{camera::Frame, geometry::Rectangle};
use core::panic;
use num::pow::Pow;
use std::{
	ops::{AddAssign, Div},
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc, Mutex,
	},
};

const EMBEDDING_LENGTH: usize = 512;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FaceRecognitionError {
	/// The detected face was too small for running the recognition model on it
	TooSmall,
}

type EmbeddingData = [f32; EMBEDDING_LENGTH];

#[derive(Debug, Clone, Copy)]
pub struct FaceEmbedding {
	data: EmbeddingData,
}

impl Default for FaceEmbedding {
	fn default() -> Self {
		let data: EmbeddingData = [0.0; EMBEDDING_LENGTH];
		Self { data }
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
		let mut out = FaceEmbedding::default();
		for index in 0..EMBEDDING_LENGTH {
			out.data[index] = self.data[index] / rhs;
		}

		out
	}
}

impl FaceEmbedding {
	pub fn new(data: EmbeddingData) -> Self {
		Self { data }
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

	fn average_embedding(embeddings: &[FaceEmbedding]) -> FaceEmbedding {
		let mut sum_embedding = FaceEmbedding::default();

		for embedding in embeddings {
			sum_embedding += *embedding;
		}

		sum_embedding / embeddings.len() as f32
	}
}

#[derive(Debug, Clone, Copy)]
pub struct FaceRecognitionData {
	embedding: FaceEmbedding,
}

#[derive(Debug, Clone)]
pub struct FaceForProcessing {
	pub rectangle: Rectangle<u32>,
	pub face_data: Result<FaceRecognitionData, FaceRecognitionError>,
}

#[derive(Clone, Debug)]
pub enum FaceForGUIAnnotation {
	Name(String),
	Warning(String),
	ScanningState {
		scanned_sample_count: usize,
		required_sample_count: usize,
	},
}

#[derive(Debug, Clone)]
pub struct FaceForGUI {
	pub rectangle: Rectangle<u32>,
	pub annotation: FaceForGUIAnnotation,
}

pub fn start(
	frame: Arc<Mutex<Option<Frame>>>,
	faces_for_gui: Arc<Mutex<Vec<FaceForGUI>>>,
	finished: Arc<AtomicBool>,
) {
	let frame_processor = FrameProcessor::new();
	let mut face_processor = ScanProcessor::new();

	loop {
		if finished.load(Ordering::SeqCst) {
			return;
		}

		let frame_lock = match frame.lock() {
			Ok(l) => l,
			Err(e) => {
				finished.store(true, Ordering::SeqCst);
				panic!("Failed to get frame lock: {e}");
			}
		};
		let image = match frame_lock.clone() {
			Some(f) => f.clone(),
			None => continue,
		};
		drop(frame_lock);

		let new_faces_for_processing = frame_processor.process_frame(&image);
		let new_faces_for_gui = face_processor.process_detected_faces(new_faces_for_processing);

		let mut faces_for_gui_lock = match faces_for_gui.lock() {
			Ok(l) => l,
			Err(e) => {
				finished.store(true, Ordering::SeqCst);
				panic!("Failed to get detected faces lock: {e}");
			}
		};
		*faces_for_gui_lock = new_faces_for_gui.clone();
		drop(faces_for_gui_lock);

		if face_processor.is_finished() {
			finished.store(true, Ordering::SeqCst);
			return;
		}
	}
}
