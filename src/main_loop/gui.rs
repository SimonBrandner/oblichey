use crate::{
	camera::{Camera, ImageSize},
	processors::{embedding_processor::EmbeddingProcessor, frame_processor::FrameProcessor},
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
	embedding_processor: Box<dyn EmbeddingProcessor>,
) {
	eframe::run_native(
		"Gday",
		NativeOptions::default(),
		Box::new(|_| Box::new(GUI::new(camera, frame_processor, embedding_processor))),
	)
}

struct GUI<'a> {
	camera: Camera<'a>,
	frame_processor: FrameProcessor,
	embedding_processor: Box<dyn EmbeddingProcessor>,
}

impl<'a> GUI<'a> {
	pub fn new(
		camera: Camera<'a>,
		frame_processor: FrameProcessor,
		embedding_processor: Box<dyn EmbeddingProcessor>,
	) -> Self {
		Self {
			camera,
			frame_processor,
			embedding_processor,
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
		self.embedding_processor
			.process_embeddings(&frame_processor_state.faces);
		if self.embedding_processor.is_finished() {
			frame.close();
		}

		egui::CentralPanel::default().show(ctx, |ui| {
			let egui_image = ColorImage::from_rgba_unmultiplied(
				image.get_size_array(),
				&image.clone().into_raw(),
			);
			let texture =
				ui.ctx()
					.load_texture("Camera", egui_image, egui::TextureOptions::default());

			ui.image(&texture, ui.available_size());
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
