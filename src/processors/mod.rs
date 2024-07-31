pub mod face_processor;
pub mod frame_processor;

use self::{
	face_processor::{FaceProcessor, ScanProcessor},
	frame_processor::FrameProcessor,
};
use crate::{camera::Frame, geometry::Rectangle};
use core::panic;
use num::pow::Pow;
use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc, Mutex,
};

const EMBEDDING_LENGTH: usize = 512;

type EmbeddingData = [f32; EMBEDDING_LENGTH];

#[derive(Debug, Clone)]
pub struct Embedding {
	data: EmbeddingData,
}

impl Embedding {
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
}

#[derive(Debug, Clone)]
pub struct Face {
	embedding: Embedding,
}

#[derive(Debug, Clone)]
pub struct DetectedFace {
	pub rectangle: Rectangle<u32>,
	pub face: Face,
}

pub fn start(
	frame: Arc<Mutex<Option<Frame>>>,
	detected_faces: Arc<Mutex<Vec<DetectedFace>>>,
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

		let new_detected_faces = frame_processor.process_frame(&image);
		let mut detected_faces_lock = match detected_faces.lock() {
			Ok(l) => l,
			Err(e) => {
				finished.store(true, Ordering::SeqCst);
				panic!("Failed to get detected faces lock: {e}");
			}
		};
		*detected_faces_lock = new_detected_faces.clone();
		drop(detected_faces_lock);

		face_processor.process_detected_faces(&new_detected_faces);
		if face_processor.is_finished() {
			finished.store(true, Ordering::SeqCst);
			return;
		}
	}
}
