pub mod face_processor;
pub mod frame_processor;

use self::{
	face_processor::{AuthProcessor, FaceProcessor},
	frame_processor::FrameProcessor,
};
use crate::{camera::Frame, geometry::Rectangle};
use core::panic;
use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc, Mutex,
};

#[derive(Debug, Clone, Default)]
pub struct Face;

#[derive(Debug, Clone)]
pub struct DetectedFace {
	pub rectangle: Rectangle,
	pub face: Face,
}

pub fn start(
	frame: Arc<Mutex<Option<Frame>>>,
	detected_faces: Arc<Mutex<Vec<DetectedFace>>>,
	finished: Arc<AtomicBool>,
) {
	let frame_processor = FrameProcessor::new();
	let face_processor = AuthProcessor::new();

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
