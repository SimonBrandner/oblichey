use super::face::FaceForProcessing;
#[double]
use crate::models::detector::FaceDetector;
#[double]
use crate::models::recognizer::FaceRecognizer;
use crate::{
	camera::Frame, geometry::Rectangle, models::recognizer::RECOGNIZER_INPUT_SIZE,
	processors::face::FaceRecognitionError,
};
use burn::backend::{wgpu::WgpuDevice, Wgpu};
use image::imageops::{crop, resize, FilterType};
use log::trace;
use mockall_double::double;

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
					trace!("Detected face too small");
					Err(FaceRecognitionError::TooSmall)
				},
			});
		}

		detected_faces
	}
}

#[cfg(test)]
mod tests {
	use image::ImageBuffer;
	use mockall::predicate::eq;

	use crate::{
		geometry::{Rectangle, Vec2D},
		models::{
			detector::MockFaceDetector,
			recognizer::{MockFaceRecognizer, RECOGNIZER_INPUT_SIZE},
		},
		processors::face::{FaceEmbedding, FaceRecognitionData, FaceRecognitionError},
	};

	use super::{rectangle_large_enough_for_recognition, FrameProcessor};

	#[test]
	fn determines_if_rectangle_is_large_enough_for_recognition() {
		let test_cases = vec![
			(Rectangle::new(Vec2D::new(0, 0), Vec2D::new(1, 1)), false),
			(
				Rectangle::new(Vec2D::new(0, 0), RECOGNIZER_INPUT_SIZE),
				true,
			),
			(
				Rectangle::new(RECOGNIZER_INPUT_SIZE, Vec2D::new(0, 0)),
				true,
			),
			(
				{
					let min = Vec2D::new(10, 55);
					Rectangle::new(Vec2D::new(0, 0), min + RECOGNIZER_INPUT_SIZE)
				},
				true,
			),
			(
				{
					let min = Vec2D::new(10, 55);
					Rectangle::new(min + RECOGNIZER_INPUT_SIZE, Vec2D::new(0, 0))
				},
				true,
			),
		];

		for (rectangle, expected_result) in test_cases {
			let result = rectangle_large_enough_for_recognition(&rectangle);
			assert_eq!(result, expected_result);
		}
	}

	#[test]
	fn processes_frame() {
		let frame = ImageBuffer::from_vec(0, 0, vec![]).expect("Failed to create frame");
		let mut detector = MockFaceDetector::default();
		let mut recognizer = MockFaceRecognizer::default();

		detector
			.expect_forward()
			.with(eq(frame.clone()))
			.times(1)
			.return_const(vec![
				Rectangle::new(Vec2D::new(0, 0), Vec2D::new(10, 10)),
				Rectangle::new(Vec2D::new(0, 0), RECOGNIZER_INPUT_SIZE),
			]);
		recognizer
			.expect_forward()
			.times(1)
			.return_const(FaceRecognitionData {
				embedding: FaceEmbedding::default(),
			});

		let result = FrameProcessor {
			detector,
			recognizer,
		}
		.process_frame(&frame);

		assert_eq!(result.len(), 2);
		if let Err(e) = result[0].face_data {
			assert_eq!(e, FaceRecognitionError::TooSmall);
		} else {
			panic!();
		}
		assert!(result[1].face_data.is_ok());
	}
}
