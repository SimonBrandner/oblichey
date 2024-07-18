//use burn::tensor;
//use model::detector::Model;
//use model::recognizer::Model;
//use burn_ndarray::{ NdArrayDevice, NdArray };
//use v4l::io::traits::Stream;
use image::RgbaImage;

#[derive(Debug, Clone)]
struct FaceData;

#[derive(Debug, Clone)]
struct FaceCoordinates;

#[derive(Debug, Clone, Default)]
pub struct ScanAuthenticatorState {
	face_coordinates: Option<FaceCoordinates>,
	date: Option<FaceData>,
}
#[derive(Debug, Clone, Default)]
pub struct AuthAuthenticatorState {
	face_coordinates: Option<FaceCoordinates>,
	authenticated: bool,
}

#[derive(Clone)]
pub enum AuthenticatorState {
	Scan(ScanAuthenticatorState),
	Auth(AuthAuthenticatorState),
}
pub struct Processor {
	state: AuthenticatorState,
}

impl Processor {
	pub fn new() -> Self {
		//let device = NdArrayDevice::default();
		//let model: Model<NdArray<f32>> = Model::new(&device);

		Self {
			state: AuthenticatorState::Scan(ScanAuthenticatorState::default()),
		}
	}

	pub fn process_frame(&self, _frame_buffer: &RgbaImage) -> AuthenticatorState {
		//let input = tensor::Tensor::<NdArray<f32>, 4>::zeros([1, 1, 28, 28], &device);
		//let output = model.forward(input);

		self.state.clone()
	}
}
