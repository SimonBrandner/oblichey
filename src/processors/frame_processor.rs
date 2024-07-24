use crate::{
	camera::ImageSize,
	model::{detector, recognizer},
	types::{DetectedFace, Face, Rectangle, Vec2D},
};
use burn::tensor::{Tensor, TensorData};
use burn_ndarray::{NdArray, NdArrayDevice};
use image::{
	imageops::{resize, FilterType},
	RgbImage,
};

const CONFIDENCE_THRESHOLD: f32 = 0.95;
const MODEL_IMAGE_SIZE: Vec2D = Vec2D { x: 640, y: 480 };

type Backend = NdArray<f32>;

#[derive(Debug, Clone)]
pub struct ProcessorState {
	pub detected_faces: Vec<DetectedFace>,
}

#[derive(Debug)]
pub struct FrameProcessor {
	device: NdArrayDevice,
	detector: detector::Model<Backend>,
	recognizer: recognizer::Model<Backend>,
}

impl FrameProcessor {
	pub fn new() -> Self {
		let device = NdArrayDevice::default();
		let detector: detector::Model<Backend> = detector::Model::default();
		let recognizer: recognizer::Model<Backend> = recognizer::Model::default();

		Self {
			device,
			detector,
			recognizer,
		}
	}

	pub fn process_frame(&self, frame: &RgbImage) -> ProcessorState {
		let input = self.normalize_detector_input(frame);
		let output = self.detector.forward(input);
		let face_rectangles = self.interpret_detector_output(output, &frame.get_size_vec2D());

		// For now we fill the `face` field with `default()`
		ProcessorState {
			detected_faces: face_rectangles
				.into_iter()
				.map(|rectangle| DetectedFace {
					face: Face::default(),
					rectangle,
				})
				.collect(),
		}
	}

	fn interpret_detector_output(
		&self,
		output: (Tensor<Backend, 3>, Tensor<Backend, 3>),
		frame_size: &Vec2D,
	) -> Vec<Rectangle> {
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
				min: Vec2D {
					x: (boxes[j + 0] * frame_size.x as f32) as usize,
					y: (boxes[j + 1] * frame_size.y as f32) as usize,
				},
				max: Vec2D {
					x: (boxes[j + 2] * frame_size.x as f32) as usize,
					y: (boxes[j + 3] * frame_size.y as f32) as usize,
				},
			});
		}

		// TODO: Filter out colliding faces

		face_rectangles
	}

	fn normalize_detector_input(&self, frame: &RgbImage) -> Tensor<Backend, 4> {
		// We don't really need this at the moment since we're using the `face_detector_640.onnx`
		// model but once we switch to the `yolo5s-face.onnx` model we will
		// Normalize by resizing
		let resized = resize(
			frame,
			MODEL_IMAGE_SIZE.x as u32,
			MODEL_IMAGE_SIZE.y as u32,
			FilterType::CatmullRom,
		);

		// Shape of the image: height, width, channels
		let shape = [MODEL_IMAGE_SIZE.y, MODEL_IMAGE_SIZE.x, 3];

		// Make into a tensor
		let tensor = Tensor::from_data(TensorData::new(resized.to_vec(), shape), &self.device);

		// Normalize between [-1, 1]
		let normalized = (tensor - Tensor::full(shape, 127, &self.device)) / 128;

		// Reorder dimension to have: channels, height, width
		let permutated = normalized.permute([2, 0, 1]);

		// Make the tensor the correct shape: batch, channels, height, width
		let unsqueezed = permutated.unsqueeze::<4>();

		unsqueezed
	}
}
