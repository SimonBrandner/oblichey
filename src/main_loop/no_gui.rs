use crate::{
	camera::Camera,
	processors::{face_processor::FaceProcessor, frame_processor::FrameProcessor},
};

pub fn start(
	mut camera: Camera,
	frame_processor: FrameProcessor,
	face_processor: Box<dyn FaceProcessor>,
) {
	loop {
		let image = match camera.get_frame() {
			Ok(b) => b,
			Err(e) => {
				println!("Failed to get frame: {e}");
				return;
			}
		};
		let frame_processor_state = frame_processor.process_frame(&image);
		face_processor.process_faces(&frame_processor_state.faces);

		if face_processor.is_finished() {
			return;
		}
	}
}
