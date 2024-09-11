use super::get_weights_file;
use super::imported::detector::Model;
use crate::camera::Frame;
use crate::geometry::{Rectangle, Vec2D};
use burn::tensor::backend::Backend;
use burn::tensor::{Tensor, TensorData};
#[cfg(test)]
use mockall::automock;

/// The size of the image the detector model takes as input
pub const DETECTOR_INPUT_SIZE: Vec2D<u32> = Vec2D { x: 640, y: 480 };
const CONFIDENCE_THRESHOLD: f32 = 0.95;

#[derive(Debug)]
pub struct FaceDetector<B: Backend> {
	device: B::Device,
	model: Model<B>,
}

#[cfg_attr(test, automock)]
#[cfg_attr(test, allow(unused))]
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

#[cfg(test)]
mod tests {
	use super::FaceDetector;
	use crate::{
		camera::Frame,
		geometry::{Rectangle, Vec2D},
		models::detector::{CONFIDENCE_THRESHOLD, DETECTOR_INPUT_SIZE},
		processors::frame_processor::BurnBackend,
	};
	use burn::{
		backend::{ndarray::NdArrayDevice, NdArray},
		tensor::{Tensor, TensorData},
	};
	use image::ImageBuffer;

	const FRAME_CHANNEL_SIZE: usize = (DETECTOR_INPUT_SIZE.x * DETECTOR_INPUT_SIZE.y) as usize;
	const FRAME_VEC_SIZE: usize = FRAME_CHANNEL_SIZE * 3;

	fn get_device() -> NdArrayDevice {
		NdArrayDevice::default()
	}

	fn get_face_detector() -> FaceDetector<NdArray<f32>> {
		FaceDetector::new(&get_device())
	}

	fn get_frame(data: Vec<u8>) -> Frame {
		ImageBuffer::from_vec(DETECTOR_INPUT_SIZE.x, DETECTOR_INPUT_SIZE.y, data)
			.expect("Failed to construct frame")
	}

	fn check_channel(channel: &[f32], expected_value: f32) {
		let mut current_expected_value: f32 = 0.0;
		for (index, v) in channel.iter().enumerate() {
			// Change expectation at the end of a row
			if index as u32 % DETECTOR_INPUT_SIZE.x == 0 {
				current_expected_value =
					if (current_expected_value - expected_value).abs() < f32::EPSILON {
						0.0
					} else {
						expected_value
					}
			}

			println!("{v}");

			assert!((*v - current_expected_value).abs() < f32::EPSILON);
		}
	}

	#[test]
	fn normalizes_input_between_values() {
		let face_detector = get_face_detector();
		let test_cases = vec![
			(vec![255; FRAME_VEC_SIZE], 1.0),
			(vec![191; FRAME_VEC_SIZE], 0.5),
			(vec![63; FRAME_VEC_SIZE], -0.5),
		];

		for (frame_data, expected_result) in test_cases {
			let frame = get_frame(frame_data);

			let normalized = face_detector.normalize_input(&frame);
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
			for i in 0..DETECTOR_INPUT_SIZE.y {
				for _ in 0..DETECTOR_INPUT_SIZE.x {
					data.extend(if i % 2 == 0 {
						value_by_channel.clone()
					} else {
						zero_value_by_channel.clone()
					});
				}
			}
			data
		};
		let face_recognizer = get_face_detector();
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
		fn create_confidences_tensor(
			confidences: &[f32],
			device: &NdArrayDevice,
		) -> Tensor<BurnBackend, 3> {
			let tensor_data_vec: Vec<f32> = confidences
				.iter()
				.flat_map(|&confidence| [0.0, confidence])
				.collect();

			Tensor::from_data(
				TensorData::new(tensor_data_vec.clone(), [tensor_data_vec.len(), 1, 1]),
				device,
			)
		}
		fn create_rectangles_tensor(
			rectangles: &[Rectangle<u32>],
			device: &NdArrayDevice,
		) -> Tensor<BurnBackend, 3> {
			let tensor_data_vec: Vec<f32> = rectangles
				.iter()
				.flat_map(|rect| {
					vec![
						(rect.min.x as f32) / (DETECTOR_INPUT_SIZE.x as f32),
						(rect.min.y as f32) / (DETECTOR_INPUT_SIZE.y as f32),
						(rect.max.x as f32) / (DETECTOR_INPUT_SIZE.x as f32),
						(rect.max.y as f32) / (DETECTOR_INPUT_SIZE.y as f32),
					]
				})
				.collect();

			Tensor::from_data(
				TensorData::new(tensor_data_vec.clone(), [tensor_data_vec.len(), 1, 1]),
				device,
			)
		}

		let device = get_device();
		let test_cases = vec![
			(
				vec![0.0, 1.0],
				vec![
					Rectangle::new(Vec2D::new(0, 0), Vec2D::new(10, 10)),
					Rectangle::new(Vec2D::new(0, 0), Vec2D::new(10, 10)),
				],
			),
			(
				vec![0.0, 1.0, 0.0, 1.0],
				vec![
					Rectangle::new(Vec2D::new(0, 0), Vec2D::new(10, 10)),
					Rectangle::new(Vec2D::new(0, 0), Vec2D::new(10, 10)),
					Rectangle::new(Vec2D::new(0, 0), Vec2D::new(20, 20)),
					Rectangle::new(Vec2D::new(0, 0), Vec2D::new(20, 20)),
				],
			),
		];

		for (confidences, rectangles) in test_cases {
			let confidences_tensor = create_confidences_tensor(&confidences, &device);
			let rectangles_tensor = create_rectangles_tensor(&rectangles, &device);
			let valid_rectangle_count = confidences.iter().fold(0, |count, confidence| {
				if *confidence > CONFIDENCE_THRESHOLD {
					count + 1
				} else {
					count
				}
			});

			let interpreted =
				FaceDetector::interpret_output(&(confidences_tensor, rectangles_tensor));

			let mut valid_rectangles = vec![];
			for (index, rectangle) in rectangles.iter().enumerate() {
				if confidences[index] < CONFIDENCE_THRESHOLD {
					continue;
				}

				valid_rectangles.push(rectangle);
			}

			assert_eq!(interpreted.len(), valid_rectangle_count);
			for i in 0..interpreted.len() {
				assert_eq!(interpreted[i], *valid_rectangles[i]);
			}
		}
	}
}
