use crate::{
	camera::ImageSize,
	model::{detector, recognizer},
};
use burn::tensor::{Tensor, TensorData};
use burn_ndarray::{NdArray, NdArrayDevice};
use eframe::egui::{Pos2, Rect};
use image::{
	imageops::{resize, FilterType},
	RgbImage,
};
use std::ops::Add;

const CONFIDENCE_THRESHOLD: f32 = 0.95;
const MODEL_IMAGE_SIZE: Vec2D = Vec2D { x: 640, y: 480 };

type Backend = NdArray<f32>;

#[derive(Debug, Clone, Copy)]
pub struct Vec2D {
	pub x: usize,
	pub y: usize,
}

impl Add<Vec2D> for Vec2D {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl Vec2D {
	pub fn to_pos2(&self) -> Pos2 {
		Pos2 {
			x: self.x as f32,
			y: self.y as f32,
		}
	}
}

#[derive(Debug, Clone)]
pub struct FaceRectangle {
	pub min: Vec2D,
	pub max: Vec2D,
}

impl FaceRectangle {
	pub fn to_rect(&self) -> Rect {
		Rect {
			min: self.min.to_pos2(),
			max: self.max.to_pos2(),
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct FaceEmbedding;

#[derive(Debug, Clone)]
pub struct Face {
	pub rectangle: FaceRectangle,
	pub embedding: FaceEmbedding,
}

#[derive(Debug, Clone)]
pub struct ProcessorState {
	pub faces: Vec<Face>,
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
		let faces = self.interpret_detector_output(output, &frame.get_size_vec2D());

		ProcessorState { faces }
	}

	fn interpret_detector_output(
		&self,
		output: (Tensor<Backend, 3>, Tensor<Backend, 3>),
		frame_size: &Vec2D,
	) -> Vec<Face> {
		let (confidences, boxes) = output;
		let confidences = confidences
			.to_data()
			.to_vec::<f32>()
			.expect("Confidences have an unexpected shape!");
		let boxes = boxes
			.to_data()
			.to_vec::<f32>()
			.expect("Boxes have an unexpected shape!");

		// TODO: Refactor this
		// Create faces out of the boxes with high confidence
		let mut faces = Vec::new();
		let mut i = 0;
		let mut j = 0;
		while i < confidences.len() {
			if confidences[i + 1] <= CONFIDENCE_THRESHOLD {
				i += 2;
				j += 4;
				continue;
			}

			faces.push(Face {
				embedding: FaceEmbedding::default(),
				rectangle: FaceRectangle {
					min: Vec2D {
						x: (boxes[j + 0] * frame_size.x as f32) as usize,
						y: (boxes[j + 1] * frame_size.y as f32) as usize,
					},
					max: Vec2D {
						x: (boxes[j + 2] * frame_size.x as f32) as usize,
						y: (boxes[j + 3] * frame_size.y as f32) as usize,
					},
				},
			});

			i += 2;
			j += 4;
		}

		// TODO: Filter out colliding faces

		faces
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
