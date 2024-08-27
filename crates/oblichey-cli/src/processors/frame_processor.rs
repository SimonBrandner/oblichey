use super::face::FaceForProcessing;
use crate::{
	camera::Frame,
	geometry::Rectangle,
	models::{
		detector::FaceDetector,
		recognizer::{FaceRecognizer, RECOGNIZER_INPUT_SIZE},
	},
	processors::face::FaceRecognitionError,
};
use burn::backend::{wgpu::WgpuDevice, Wgpu};
use image::imageops::{crop, resize, FilterType};

type Backend = Wgpu<f32, i32>;

/// Checks whether a `Rectangle` is large enough to be passed into the recognizer model. We would
/// not want to pass an upscaled image to it
fn rectangle_large_enough_for_recognition(rectangle: &Rectangle<u32>) -> bool {
	let size = rectangle
		.size()
		.expect("Failed to calculate size of Rectangle!");

	if size.x < RECOGNIZER_INPUT_SIZE.x {
		return false;
	}
	if size.y < RECOGNIZER_INPUT_SIZE.y {
		return false;
	}

	true
}

fn get_face_image(frame: &mut Frame, face_rectangle: &Rectangle<u32>) -> Frame {
	let cropped = crop(
		frame,
		face_rectangle.min.x,
		face_rectangle.min.y,
		face_rectangle.max.x + face_rectangle.min.x,
		face_rectangle.max.y + face_rectangle.min.y,
	);

	resize(
		&cropped.to_image(),
		RECOGNIZER_INPUT_SIZE.x,
		RECOGNIZER_INPUT_SIZE.y,
		FilterType::CatmullRom,
	)
}

#[derive(Debug)]
pub struct FrameProcessor {
	detector: FaceDetector<Backend>,
	recognizer: FaceRecognizer<Backend>,
}

impl FrameProcessor {
	pub fn new() -> Self {
		let device = WgpuDevice::default();

		Self {
			detector: FaceDetector::new(&device),
			recognizer: FaceRecognizer::new(&device),
		}
	}

	/// Process a frame from the `Camera` and return found faces
	pub fn process_frame(&self, frame: &Frame) -> Vec<FaceForProcessing> {
		let face_rectangles = self.detector.forward(frame);

		let mut detected_faces = Vec::new();
		let mut frame = frame.clone();
		for rectangle in face_rectangles {
			detected_faces.push(FaceForProcessing {
				rectangle,
				face_data: if rectangle_large_enough_for_recognition(&rectangle) {
					let face_image = get_face_image(&mut frame, &rectangle);
					let face = self.recognizer.forward(&face_image);
					Ok(face)
				} else {
					Err(FaceRecognitionError::TooSmall)
				},
			});
		}

		detected_faces
	}
}
