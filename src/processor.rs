use eframe::egui::{Pos2, Rect};
use image::RgbaImage;
use std::ops::Add;
//use burn::tensor;
//use model::detector::Model;
//use model::recognizer::Model;
//use burn_ndarray::{ NdArrayDevice, NdArray };

#[derive(Debug, Clone)]
pub struct FaceData;

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
pub struct FaceCoordinates {
	pub position: Vec2D,
	pub size: Vec2D,
}

impl FaceCoordinates {
	pub fn to_rect(&self) -> Rect {
		Rect {
			min: self.position.to_pos2(),
			max: (self.position + self.size).to_pos2(),
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct ScanProcessState {
	pub date: Option<FaceData>,
}

#[derive(Debug, Clone, Default)]
pub struct AuthProcessState {
	pub authenticated: bool,
}

#[derive(Debug, Clone)]
pub enum ProcessState {
	Scan(ScanProcessState),
	Auth(AuthProcessState),
}

#[derive(Debug, Clone)]
pub struct ProcessorState {
	pub face_coordinates: Vec<FaceCoordinates>,
	pub process_state: ProcessState,
}

pub struct Processor {
	state: ProcessorState,
}

impl Processor {
	pub fn new() -> Self {
		//let device = NdArrayDevice::default();
		//let model: Model<NdArray<f32>> = Model::new(&device);

		Self {
			state: ProcessorState {
				face_coordinates: vec![FaceCoordinates {
					position: Vec2D { x: 50, y: 100 },
					size: Vec2D { x: 100, y: 150 },
				}],
				process_state: ProcessState::Auth(AuthProcessState {
					authenticated: false,
				}),
			},
		}
	}

	pub fn process_frame(&self, _frame_buffer: &RgbaImage) -> ProcessorState {
		//let input = tensor::Tensor::<NdArray<f32>, 4>::zeros([1, 1, 28, 28], &device);
		//let output = model.forward(input);

		self.state.clone()
	}
}
