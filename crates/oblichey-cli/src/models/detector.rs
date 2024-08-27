use super::get_weights_file;
use super::imported::detector::Model;
use crate::camera::Frame;
use crate::geometry::{Rectangle, Vec2D};
use burn::tensor::backend::Backend;
use burn::tensor::{Tensor, TensorData};

/// The size of the image the detector model takes as input
pub const DETECTOR_INPUT_SIZE: Vec2D<u32> = Vec2D { x: 640, y: 480 };
const CONFIDENCE_THRESHOLD: f32 = 0.95;

#[derive(Debug)]
pub struct FaceDetector<B: Backend> {
	device: B::Device,
	model: Model<B>,
}

impl<B: Backend> FaceDetector<B> {
	pub fn new(device: &B::Device) -> Self {
		Self {
			model: Model::from_file(&get_weights_file("detector"), device),
			device: device.clone(),
		}
	}

	/// This is going to panic if the frame has a size other than `DETECTOR_INPUT_SIZE`
	pub fn forward(&self, frame: &Frame) -> Vec<Rectangle<u32>> {
		assert_eq!(
			frame.width(),
			DETECTOR_INPUT_SIZE.x,
			"Image width does not match model requirements!"
		);
		assert_eq!(
			frame.height(),
			DETECTOR_INPUT_SIZE.y,
			"Image height does not match model requirements!"
		);

		let input = self.normalize_input(frame);
		let output = self.model.forward(input);
		Self::interpret_output(&output)
	}

	fn normalize_input(&self, frame: &Frame) -> Tensor<B, 4> {
		// Shape of the image: height, width, channels
		let shape = [
			DETECTOR_INPUT_SIZE.y as usize,
			DETECTOR_INPUT_SIZE.x as usize,
			3_usize,
		];

		// Make into a tensor
		let tensor = Tensor::from_data(TensorData::new(frame.to_vec(), shape), &self.device);

		// Normalize between [-1, 1]
		let normalized = (tensor - Tensor::full(shape, 127, &self.device)) / 128.0;

		// Reorder dimension to have: channels, height, width
		let permutated = normalized.permute([2, 0, 1]);

		// Make the tensor the correct shape: batch, channels, height, width
		permutated.unsqueeze::<4>()
	}

	fn interpret_output(output: &(Tensor<B, 3>, Tensor<B, 3>)) -> Vec<Rectangle<u32>> {
		let (confidences, boxes) = output;
		let confidences = confidences
			.to_data()
			.to_vec::<f32>()
			.expect("Confidences have an unexpected shape!");
		let boxes = boxes
			.to_data()
			.to_vec::<f32>()
			.expect("Boxes have an unexpected shape!");

		let mut face_rectangles = Vec::new();
		for n in (0..confidences.len()).step_by(2) {
			// This produces (i, j): (1, 0), (3, 4), (5, 8), (7, 12)...
			let i = n + 1;
			let j = n * 2;

			if confidences[i] <= CONFIDENCE_THRESHOLD {
				continue;
			}

			#[allow(clippy::cast_sign_loss)]
			face_rectangles.push(Rectangle {
				min: Vec2D::new(
					(boxes[j] * DETECTOR_INPUT_SIZE.x as f32) as u32,
					(boxes[j + 1] * DETECTOR_INPUT_SIZE.y as f32) as u32,
				),
				max: Vec2D::new(
					(boxes[j + 2] * DETECTOR_INPUT_SIZE.x as f32) as u32,
					(boxes[j + 3] * DETECTOR_INPUT_SIZE.y as f32) as u32,
				),
			});
		}

		Rectangle::filter_out_colliding(&mut face_rectangles);

		face_rectangles
	}
}
