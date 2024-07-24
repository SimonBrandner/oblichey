use crate::{
	camera::{Camera, ImageSize},
	processors::{face_processor::FaceProcessor, frame_processor::FrameProcessor},
};
use eframe::{
	egui::{self, Color32, ColorImage, Rounding, Stroke},
	NativeOptions,
};

const FACE_RECTANGLE_STROKE: Stroke = Stroke {
	width: 4.0,
	color: Color32::from_rgb(255, 0, 0),
};

pub fn start(
	camera: Camera<'static>,
	frame_processor: FrameProcessor,
	face_processor: Box<dyn FaceProcessor>,
) {
	eframe::run_native(
		"Gday",
		NativeOptions::default(),
		Box::new(|_| Box::new(GUI::new(camera, frame_processor, face_processor))),
	)
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

		egui::CentralPanel::default().show(ctx, |ui| {
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
