use crate::{camera::Camera, processor::Processor};
use std::rc::Rc;

pub fn start(mut camera: Camera, processor: Rc<Processor>) {
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
