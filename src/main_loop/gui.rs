use crate::{
	camera::{self, Camera, ImageSize},
	processors::{face_processor::FaceProcessor, frame_processor::FrameProcessor},
};
use eframe::{
	egui::{self, Color32, ColorImage, Rounding, Stroke, Vec2},
	NativeOptions,
};
use std::fmt::Display;

const FACE_RECTANGLE_STROKE: Stroke = Stroke {
	width: 4.0,
	color: Color32::from_rgb(255, 0, 0),
};

pub enum Error {
	Camera(camera::Error),
}

impl From<camera::Error> for Error {
	fn from(value: camera::Error) -> Self {
		Self::Camera(value)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Camera(e) => write!(f, "Camera error: {e}"),
		}
	}
}

pub fn start(
	camera: Camera<'static>,
	frame_processor: FrameProcessor,
	face_processor: Box<dyn FaceProcessor>,
) -> Result<(), Error> {
	let size = camera.get_output_size()?;

	eframe::run_native(
		"Gday",
		NativeOptions {
			resizable: false,
			initial_window_size: Some(Vec2::new(size.x as f32, size.y as f32)),
			..NativeOptions::default()
		},
		Box::new(|_| Box::new(GUI::new(camera, frame_processor, face_processor))),
	);
	Ok(())
}

struct GUI<'a> {
	camera: Camera<'a>,
	frame_processor: FrameProcessor,
	face_processor: Box<dyn FaceProcessor>,
}

impl<'a> GUI<'a> {
	pub fn new(
		camera: Camera<'a>,
		frame_processor: FrameProcessor,
		face_processor: Box<dyn FaceProcessor>,
	) -> Self {
		Self {
			camera,
			frame_processor,
			face_processor,
		}
	}
}

impl eframe::App for GUI<'_> {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		let image = match self.camera.get_frame() {
			Ok(b) => b,
			Err(e) => {
				println!("Failed to get frame: {e}");
				return;
			}
		};

		let frame_processor_state = self.frame_processor.process_frame(&image);
		self.face_processor
			.process_faces(&frame_processor_state.faces);
		if self.face_processor.is_finished() {
			frame.close();
		}

		egui::CentralPanel::default()
			.frame(egui::Frame::none().inner_margin(0.0).outer_margin(0.0))
			.show(ctx, |ui| {
				let image_size_vec2 = image.get_size_vec2();
				let image_size_array = image.get_size_array();
				let image_raw = &image.into_raw();
				let egui_image = ColorImage::from_rgb(image_size_array, image_raw);
				let image_texture =
					ui.ctx()
						.load_texture("Camera", egui_image, egui::TextureOptions::default());

				ui.image(&image_texture, image_size_vec2);
				for face in frame_processor_state.faces {
					ui.painter().rect_stroke(
						face.rectangle.to_rect(),
						Rounding::default(),
						FACE_RECTANGLE_STROKE,
					);
				}
			});
		ctx.request_repaint();
	}
}
