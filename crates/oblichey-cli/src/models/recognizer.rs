use super::{get_weights_file, imported::recognizer::Model};
use crate::{
	camera::Frame,
	geometry::Vec2D,
	processors::face::{FaceEmbedding, FaceEmbeddingData, FaceRecognitionData},
};
use burn::tensor::{backend::Backend, Tensor, TensorData};
#[cfg(test)]
use mockall::automock;

/// The size of the image the recognizer model takes as input
pub const RECOGNIZER_INPUT_SIZE: Vec2D<u32> = Vec2D { x: 128, y: 128 };

#[derive(Debug)]
pub struct FaceRecognizer<B: Backend> {
	device: B::Device,
	model: Model<B>,
}

#[cfg_attr(test, automock)]
#[cfg_attr(test, allow(unused))]
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

#[cfg(test)]
mod tests {
	use super::{FaceRecognizer, RECOGNIZER_INPUT_SIZE};
	use crate::{
		camera::Frame,
		processors::{
			face::{FaceEmbedding, EMBEDDING_LENGTH},
			frame_processor::BurnBackend,
		},
	};
	use burn::{
		backend::{ndarray::NdArrayDevice, NdArray},
		tensor::{Tensor, TensorData},
	};
	use image::ImageBuffer;

	const FRAME_CHANNEL_SIZE: usize = (RECOGNIZER_INPUT_SIZE.x * RECOGNIZER_INPUT_SIZE.y) as usize;
	const FRAME_VEC_SIZE: usize = FRAME_CHANNEL_SIZE * 3;

	fn get_device() -> NdArrayDevice {
		NdArrayDevice::default()
	}

	fn get_face_recognizer() -> FaceRecognizer<NdArray<f32>> {
		FaceRecognizer::new(&get_device())
	}

	fn get_frame(data: Vec<u8>) -> Frame {
		ImageBuffer::from_vec(RECOGNIZER_INPUT_SIZE.x, RECOGNIZER_INPUT_SIZE.y, data)
			.expect("Failed to construct frame")
	}

	fn check_channel(channel: &[f32], expected_value: f32) {
		let mut current_expected_value: f32 = 0.0;
		for (index, v) in channel.iter().enumerate() {
			// Change expectation at the end of a row
			if index as u32 % RECOGNIZER_INPUT_SIZE.x == 0 {
				current_expected_value =
					if (current_expected_value - expected_value).abs() < f32::EPSILON {
						0.0
					} else {
						expected_value
					}
			}

			assert!((*v - current_expected_value).abs() < f32::EPSILON);
		}
	}

	#[test]
	fn normalizes_input_between_values() {
		let face_recognizer = get_face_recognizer();
		let test_cases = vec![
			(vec![255; FRAME_VEC_SIZE], 1.0),
			(vec![191; FRAME_VEC_SIZE], 0.5),
			(vec![63; FRAME_VEC_SIZE], -0.5),
		];

		for (frame_data, expected_result) in test_cases {
			let frame = get_frame(frame_data);

			let normalized = face_recognizer.normalize_input(&frame);
			let vector = normalized.to_data().to_vec::<f32>().expect("Failed");

			for element in vector {
				assert!((element - expected_result).abs() < f32::EPSILON);
			}
		}
	}

	#[test]
	fn permutes_dimensions_during_normalization() {
		#[allow(clippy::identity_op, clippy::erasing_op)]
		let zero = 0 * 128 + 127; // This will become 0 after normalization
		let zero_value_by_channel = vec![zero; 3]; // This will normalize to [0.0, 0.0, 0.0]
		let value_by_channel = vec![159, 191, 255]; // This will normalize to [0.25, 0.5, 1.0]
		let value_by_channel_normalized: Vec<f32> = value_by_channel
			.iter()
			.map(|&value| (value as f32 - 127.0) / 128.0)
			.collect();
		let frame_data = {
			let mut data = vec![];
			for i in 0..RECOGNIZER_INPUT_SIZE.y {
				for _ in 0..RECOGNIZER_INPUT_SIZE.x {
					data.extend(if i % 2 == 0 {
						value_by_channel.clone()
					} else {
						zero_value_by_channel.clone()
					});
				}
			}
			data
		};
		let face_recognizer = get_face_recognizer();
		let frame = get_frame(frame_data);
		let normalized = face_recognizer.normalize_input(&frame);
		let vector = normalized.to_data().to_vec::<f32>().expect("Failed");

		for (index, value) in value_by_channel_normalized.iter().enumerate() {
			let channel = &vector[FRAME_CHANNEL_SIZE * index..FRAME_CHANNEL_SIZE * (index + 1)];
			check_channel(channel, *value);
		}
	}

	#[test]
	fn interprets_output() {
		let device = get_device();
		let test_cases = vec![
			[0.0; EMBEDDING_LENGTH],
			[128.0; EMBEDDING_LENGTH],
			[255.0; EMBEDDING_LENGTH],
		];

		for test_case in test_cases {
			let expected_result = FaceEmbedding::new(&test_case);
			let output: Tensor<BurnBackend, 2> =
				Tensor::from_data(TensorData::new(test_case.to_vec(), [512, 1]), &device);
			let result = FaceRecognizer::interpret_output(&output).embedding;

			assert_eq!(result, expected_result);
		}
	}
}
