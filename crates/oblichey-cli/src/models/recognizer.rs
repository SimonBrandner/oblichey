use super::{get_weights_file, imported::recognizer::Model};
use crate::{
	camera::Frame,
	geometry::Vec2D,
	processors::face::{FaceEmbedding, FaceEmbeddingData, FaceRecognitionData},
};
use burn::tensor::{backend::Backend, Tensor, TensorData};

/// The size of the image the recognizer model takes as input
pub const RECOGNIZER_INPUT_SIZE: Vec2D<u32> = Vec2D { x: 128, y: 128 };

#[derive(Debug)]
pub struct FaceRecognizer<B: Backend> {
	device: B::Device,
	model: Model<B>,
}

impl<B: Backend> FaceRecognizer<B> {
	pub fn new(device: &B::Device) -> Self {
		Self {
			model: Model::from_file(&get_weights_file("recognizer"), device),
			device: device.clone(),
		}
	}

	/// This is going to panic if the frame has a size other than `DETECTOR_INPUT_SIZE`
	pub fn forward(&self, face_image: &Frame) -> FaceRecognitionData {
		assert_eq!(
			face_image.width(),
			RECOGNIZER_INPUT_SIZE.x,
			"Face image width does not match model requirements!"
		);
		assert_eq!(
			face_image.height(),
			RECOGNIZER_INPUT_SIZE.y,
			"Face image height does not match model requirements!"
		);

		let input = self.normalize_input(face_image);
		let output = self.model.forward(input);
		Self::interpret_output(&output)
	}

	fn normalize_input(&self, face_image: &Frame) -> Tensor<B, 4> {
		// Shape of the image: height, width, channels
		let shape = [
			RECOGNIZER_INPUT_SIZE.y as usize,
			RECOGNIZER_INPUT_SIZE.x as usize,
			3_usize,
		];

		// Make into a tensor
		let tensor = Tensor::from_data(TensorData::new(face_image.to_vec(), shape), &self.device);

		// Normalize between [-1, 1]
		let normalized = (tensor - Tensor::full(shape, 127, &self.device)) / 128.0;

		// Reorder dimension to have: channels, height, width
		let permutated = normalized.permute([2, 0, 1]);

		// Make the tensor the correct shape: batch, channels, height, width
		permutated.unsqueeze::<4>()
	}

	fn interpret_output(output: &Tensor<B, 2>) -> FaceRecognitionData {
		let data = output
			.to_data()
			.to_vec::<f32>()
			.expect("Embedding has an unexpected shape!");
		let embedding_data =
			FaceEmbeddingData::try_from(data).expect("Embedding has an unexpected shape!");
		let embedding = FaceEmbedding::new(&embedding_data);

		FaceRecognitionData { embedding }
	}
}
