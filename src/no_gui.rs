use crate::{
	camera::Camera, embedding_processor::EmbeddingProcessor, frame_processor::FrameProcessor,
};

pub fn start(
	mut camera: Camera,
	frame_processor: FrameProcessor,
	embedding_processor: Box<dyn EmbeddingProcessor>,
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
		embedding_processor.process_embeddings(&frame_processor_state.faces);

		if embedding_processor.is_finished() {
			return;
		}
	}
}
