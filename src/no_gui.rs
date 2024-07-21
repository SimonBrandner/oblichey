use crate::{camera::Camera, processor::Processor};

pub fn start(mut camera: Camera, processor: Processor) {
	loop {
		let image = match camera.get_frame() {
			Ok(b) => b,
			Err(e) => {
				println!("Failed to get frame: {e}");
				return;
			}
		};

		let _state = processor.process_frame(&image);
	}
}
