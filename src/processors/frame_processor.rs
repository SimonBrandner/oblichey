use super::{DetectedFace, Face};
use crate::{
	camera::Frame,
	geometry::{Rectangle, Vec2D},
	models::{detector, recognizer},
};
use burn::backend::{wgpu::WgpuDevice, Wgpu};
use burn::tensor::{Tensor, TensorData};
use image::RgbImage;

const INTERSECTION_OVER_UNION_THRESHOLD: f32 = 0.5;
const CONFIDENCE_THRESHOLD: f32 = 0.95;
/// The size of the image the detector model takes as input
pub const DETECTOR_INPUT_SIZE: Vec2D<u32> = Vec2D { x: 640, y: 480 };

type Backend = Wgpu<f32, i32>;

#[derive(Debug)]
pub struct FrameProcessor {
	device: WgpuDevice,
	detector: detector::Model<Backend>,
	recognizer: recognizer::Model<Backend>,
}

impl FrameProcessor {
	pub fn new() -> Self {
		let device = WgpuDevice::default();
		let detector: detector::Model<Backend> = detector::Model::default();
		let recognizer: recognizer::Model<Backend> = recognizer::Model::default();

		Self {
			device,
			detector,
			recognizer,
		}
	}

	pub fn process_frame(&self, frame: &RgbImage) -> Vec<DetectedFace> {
		assert_eq!(
			frame.width(),
			DETECTOR_INPUT_SIZE.x,
			"Image width does not match network requirements!"
		);
		assert_eq!(
			frame.height(),
			DETECTOR_INPUT_SIZE.y,
			"Image height does not match network requirements!"
		);

		let detector_input = self.normalize_detector_input(frame);
		let detector_output = self.detector.forward(detector_input);
		let face_rectangles = self.interpret_detector_output(detector_output);

		// For now we fill the `face` field with `default()`
		face_rectangles
			.into_iter()
			.map(|rectangle| DetectedFace {
				face: Face::default(),
				rectangle,
			})
			.collect()
	}

	fn interpret_detector_output(
		&self,
		output: (Tensor<Backend, 3>, Tensor<Backend, 3>),
	) -> Vec<Rectangle<u32>> {
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

			face_rectangles.push(Rectangle {
				min: Vec2D::new(
					(boxes[j + 0] * DETECTOR_INPUT_SIZE.x as f32) as u32,
					(boxes[j + 1] * DETECTOR_INPUT_SIZE.y as f32) as u32,
				),
				max: Vec2D::new(
					(boxes[j + 2] * DETECTOR_INPUT_SIZE.x as f32) as u32,
					(boxes[j + 3] * DETECTOR_INPUT_SIZE.y as f32) as u32,
				),
			});
		}

		// Filter out colliding face rectangles
		let mut i = 0;
		while i < face_rectangles.len() {
			let mut j = i + 1;
			while j < face_rectangles.len() {
				if face_rectangles[i].intersection_over_union(&face_rectangles[j])
					> INTERSECTION_OVER_UNION_THRESHOLD
				{
					face_rectangles.remove(j);
					j -= 1
				}
				j += 1
			}
			i += 1;
		}

		face_rectangles
	}

	fn normalize_detector_input(&self, frame: &Frame) -> Tensor<Backend, 4> {
		// Shape of the image: height, width, channels
		let shape = [
			DETECTOR_INPUT_SIZE.y as usize,
			DETECTOR_INPUT_SIZE.x as usize,
			3 as usize,
		];

		// Make into a tensor
		let tensor = Tensor::from_data(TensorData::new(frame.to_vec(), shape), &self.device);

		// Normalize between [-1, 1]
		let normalized = (tensor - Tensor::full(shape, 127, &self.device)) / 128.0;

		// Reorder dimension to have: channels, height, width
		let permutated = normalized.permute([2, 0, 1]);

		// Make the tensor the correct shape: batch, channels, height, width
		let unsqueezed = permutated.unsqueeze::<4>();

		unsqueezed
	}
}
