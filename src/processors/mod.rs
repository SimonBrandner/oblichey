pub mod face;
pub mod face_processor;
pub mod frame_processor;

use self::{face::FaceForGUI, face_processor::FaceProcessor, frame_processor::FrameProcessor};
use crate::camera::Frame;
use core::panic;
use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc, Mutex,
};

pub fn start(
	frame: &Arc<Mutex<Option<Frame>>>,
	faces_for_gui: &Arc<Mutex<Vec<FaceForGUI>>>,
	finished: &Arc<AtomicBool>,
	face_processor: &Arc<Mutex<dyn FaceProcessor + Send + Sync>>,
) {
	let frame_processor = FrameProcessor::new();

	loop {
		if finished.load(Ordering::SeqCst) {
			return;
		}

		let frame_lock = match frame.lock() {
			Ok(l) => l,
			Err(e) => panic!("Failed to get frame lock: {e}"),
		};
		let Some(new_frame) = (*frame_lock).clone() else {
			continue;
		};
		drop(frame_lock);

		let faces_for_processing = frame_processor.process_frame(&new_frame);
		let mut face_processor_lock = match face_processor.lock() {
			Ok(l) => l,
			Err(e) => panic!("Failed to get face processor lock: {e}"),
		};
		let new_faces_for_gui = face_processor_lock.process_faces(faces_for_processing);
		if face_processor_lock.is_finished() {
			return;
		}
		drop(face_processor_lock);

		let mut faces_for_gui_lock = match faces_for_gui.lock() {
			Ok(l) => l,
			Err(e) => panic!("Failed to get detected faces lock: {e}"),
		};
		faces_for_gui_lock.clone_from(&new_faces_for_gui);
		drop(faces_for_gui_lock);
	}
}
