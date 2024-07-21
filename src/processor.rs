//use burn::tensor;
//use model::detector::Model;
//use model::recognizer::Model;
//use burn_ndarray::{ NdArrayDevice, NdArray };
use image::RgbaImage;

#[derive(Debug, Clone)]
pub struct FaceData;

#[derive(Debug, Clone)]
pub struct Coordinates {
	pub x: usize,
	pub y: usize,
}

#[derive(Debug, Clone)]
pub struct FaceCoordinates {
	pub location: Coordinates,
	pub size: Coordinates,
}

#[derive(Debug, Clone, Default)]
pub struct ScanProcessorState {
	pub face_coordinates: Option<FaceCoordinates>,
	pub date: Option<FaceData>,
}
#[derive(Debug, Clone, Default)]
pub struct AuthProcessorState {
	pub face_coordinates: Vec<FaceCoordinates>,
	pub authenticated: bool,
}

#[derive(Clone)]
pub enum ProcessorState {
	Scan(ScanProcessorState),
	Auth(AuthProcessorState),
}
pub struct Processor {
	state: ProcessorState,
}

impl Processor {
	pub fn new() -> Self {
		//let device = NdArrayDevice::default();
		//let model: Model<NdArray<f32>> = Model::new(&device);

		Self {
			state: ProcessorState::Scan(ScanProcessorState::default()),
		}
	}

	pub fn process_frame(&self, _frame_buffer: &RgbaImage) -> ProcessorState {
		//let input = tensor::Tensor::<NdArray<f32>, 4>::zeros([1, 1, 28, 28], &device);
		//let output = model.forward(input);

		self.state.clone()
	}
}
