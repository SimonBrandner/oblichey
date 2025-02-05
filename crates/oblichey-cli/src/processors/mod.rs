pub mod auth_processor;
pub mod face;
pub mod face_processor;
pub mod frame_processor;
pub mod scan_processor;

use log::warn;

use self::{face::FaceForGUI, face_processor::FaceProcessor, frame_processor::FrameProcessor};
use crate::camera::Frame;
use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc, Mutex,
};

/// Starts the processing loop
pub fn start(
	frame: &Arc<Mutex<Option<Frame>>>,
	faces_for_gui: &Arc<Mutex<Vec<FaceForGUI>>>,
	finished: &Arc<AtomicBool>,
	face_processor: &Arc<Mutex<dyn FaceProcessor + Send + Sync>>,
) -> Result<(), String> {
	let frame_processor = FrameProcessor::new();

	loop {
		if finished.load(Ordering::SeqCst) {
			return Ok(());
		}

		let frame_lock = match frame.lock() {
			Ok(l) => l,
			Err(e) => panic!("Failed to get lock: {e}"),
		};
		let Some(new_frame) = (*frame_lock).clone() else {
			warn!("No frame to process");
			continue;
		};
		drop(frame_lock);

		let faces_for_processing = frame_processor.process_frame(&new_frame);
		let mut face_processor_lock = match face_processor.lock() {
			Ok(l) => l,
			Err(e) => panic!("Failed to get lock: {e}"),
		};
		let new_faces_for_gui = face_processor_lock.process_faces(faces_for_processing);
		if face_processor_lock.is_finished() {
			return Ok(());
		}
		drop(face_processor_lock);

		let mut faces_for_gui_lock = match faces_for_gui.lock() {
			Ok(l) => l,
			Err(e) => panic!("Failed to get lock: {e}"),
		};
		faces_for_gui_lock.clone_from(&new_faces_for_gui);
		drop(faces_for_gui_lock);
	}
}
