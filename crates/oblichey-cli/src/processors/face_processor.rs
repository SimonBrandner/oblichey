use super::{face::FaceForProcessing, FaceForGUI};
use std::fmt::Debug;

pub trait FaceProcessor: Debug {
	/// Processes faces found by `FrameProcessor`
	fn process_faces(&mut self, detected_faces: Vec<FaceForProcessing>) -> Vec<FaceForGUI>;
	fn is_finished(&self) -> bool;
}
