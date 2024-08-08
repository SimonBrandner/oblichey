use super::face::{FaceEmbedding, FaceEmbeddingData, FaceForProcessing, FaceRecognitionData};
use crate::{
	camera::Frame,
	geometry::{Rectangle, Vec2D},
	models::{detector, recognizer},
	processors::face::FaceRecognitionError,
};
use burn::backend::{wgpu::WgpuDevice, Wgpu};
use burn::tensor::{Tensor, TensorData};
use image::imageops::{crop, resize, FilterType};

const WEIGHTS_DIRECTORY_NAME: &str = "weights";
const INTERSECTION_OVER_UNION_THRESHOLD: f32 = 0.5;
const CONFIDENCE_THRESHOLD: f32 = 0.95;
/// The size of the image the detector model takes as input
pub const DETECTOR_INPUT_SIZE: Vec2D<u32> = Vec2D { x: 640, y: 480 };
/// The size of the image the recognizer model takes as input
pub const RECOGNIZER_INPUT_SIZE: Vec2D<u32> = Vec2D { x: 128, y: 128 };

fn rectangle_large_enough_for_recognition(rectangle: &Rectangle<u32>) -> bool {
	let size = rectangle
		.size()
		.expect("Failed to calculate size of Rectangle!");

	if size.x < RECOGNIZER_INPUT_SIZE.x {
		return false;
	}
	if size.y < RECOGNIZER_INPUT_SIZE.y {
		return false;
	}

	true
}

fn get_weights_file(model_name: &str) -> String {
	let path_to_executable = std::env::current_exe().expect("Could not get path to executable!");
	let path_to_dir_with_executable = path_to_executable
		.parent()
		.expect("Could not get parent directory!");

	path_to_dir_with_executable
		.join(WEIGHTS_DIRECTORY_NAME)
		.join(String::new() + model_name + ".mpk")
		.to_str()
		.expect("Failed to get valid UTF-8 string from path!")
		.to_owned()
}

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
		let detector: detector::Model<Backend> =
			detector::Model::from_file(&get_weights_file("detector"), &device);
		let recognizer: recognizer::Model<Backend> =
			recognizer::Model::from_file(&get_weights_file("recognizer"), &device);

		Self {
			device,
			detector,
			recognizer,
		}
	}

	pub fn process_frame(&self, frame: &Frame) -> Vec<FaceForProcessing> {
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

		let mut detected_faces = Vec::new();
		let mut frame = frame.clone();
		for rectangle in face_rectangles {
			detected_faces.push(FaceForProcessing {
				rectangle,
				face_data: if rectangle_large_enough_for_recognition(&rectangle) {
					let recognizer_input = self.normalize_recognizer_input(&mut frame, &rectangle);
					let recognizer_output = self.recognizer.forward(recognizer_input);
					let face = self.interpret_recognizer_output(&recognizer_output);
					Ok(face)
				} else {
					Err(FaceRecognitionError::TooSmall)
				},
			})
		}

		detected_faces
	}

	fn interpret_recognizer_output(&self, output: &Tensor<Backend, 2>) -> FaceRecognitionData {
		let data = output
			.to_data()
			.to_vec::<f32>()
			.expect("Embedding has an unexpected shape!");
		let embedding_data =
			FaceEmbeddingData::try_from(data).expect("Embedding has an unexpected shape!");
		let embedding = FaceEmbedding::new(embedding_data);

		FaceRecognitionData { embedding }
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
				if face_rectangles[i]
					.intersection_over_union(&face_rectangles[j])
					.expect("Failed to calculate intersection over union")
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

	fn normalize_recognizer_input(
		&self,
		frame: &mut Frame,
		face_rectangle: &Rectangle<u32>,
	) -> Tensor<Backend, 4> {
		let cropped = crop(
			frame,
			face_rectangle.min.x,
			face_rectangle.min.y,
			face_rectangle.max.x + face_rectangle.min.x,
			face_rectangle.max.y + face_rectangle.min.y,
		);

		let resized = resize(
			&cropped.to_image(),
			RECOGNIZER_INPUT_SIZE.x,
			RECOGNIZER_INPUT_SIZE.y,
			FilterType::CatmullRom,
		);

		// Shape of the image: height, width, channels
		let shape = [
			RECOGNIZER_INPUT_SIZE.y as usize,
			RECOGNIZER_INPUT_SIZE.x as usize,
			3 as usize,
		];

		// Make into a tensor
		let tensor = Tensor::from_data(TensorData::new(resized.to_vec(), shape), &self.device);

		// Normalize between [-1, 1]
		let normalized = (tensor - Tensor::full(shape, 127, &self.device)) / 128.0;

		// Reorder dimension to have: channels, height, width
		let permutated = normalized.permute([2, 0, 1]);

		// Make the tensor the correct shape: batch, channels, height, width
		let unsqueezed = permutated.unsqueeze::<4>();

		unsqueezed
	}
}
