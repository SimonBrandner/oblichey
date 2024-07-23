//use burn::tensor::Tensor;
use crate::model::detector;
use burn_ndarray::{NdArray, NdArrayDevice};
use eframe::egui::{Pos2, Rect};
use image::RgbImage;
use std::ops::Add;

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
	pub position: Vec2D,
	pub size: Vec2D,
}

impl FaceRectangle {
	pub fn to_rect(&self) -> Rect {
		Rect {
			min: self.position.to_pos2(),
			max: (self.position + self.size).to_pos2(),
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
	detector: detector::Model<NdArray<f32>>,
}

impl FrameProcessor {
	pub fn new() -> Self {
		let device = NdArrayDevice::default();
		let detector: detector::Model<NdArray<f32>> = detector::Model::default();

		Self { device, detector }
	}

	pub fn process_frame(&self, _frame_buffer: &RgbImage) -> ProcessorState {
		//let input = Tensor::ones([1, 3, 480, 640], &self.device);
		//let (scores, boxes) = self.detector.forward(input);

		ProcessorState {
			faces: vec![Face {
				embedding: FaceEmbedding::default(),
				rectangle: FaceRectangle {
					position: Vec2D { x: 50, y: 100 },
					size: Vec2D { x: 100, y: 150 },
				},
			}],
		}
	}
}
