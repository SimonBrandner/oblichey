use std::rc::Rc;

use crate::{
	camera::{Camera, ImageSize},
	processor::Processor,
};
use eframe::{
	egui::{self, Color32, ColorImage, Rounding, Stroke},
	NativeOptions,
};

const FACE_RECTANGLE_STROKE: Stroke = Stroke {
	width: 4.0,
	color: Color32::from_rgb(255, 0, 0),
};

pub fn start(camera: Camera<'static>, processor: Rc<Processor>) {
	eframe::run_native(
		"GDay",
		NativeOptions::default(),
		Box::new(|_| Box::new(GUI::new(camera, processor))),
	)
}

struct GUI<'a> {
	camera: Camera<'a>,
	processor: Rc<Processor>,
}

impl<'a> GUI<'a> {
	pub fn new(camera: Camera<'a>, processor: Rc<Processor>) -> Self {
		Self { camera, processor }
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

		self.processor.process_frame(&image);
		let state = self.processor.get_state();
		if let Some(_) = self.processor.get_result() {
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
			for face_coordinates in state.face_coordinates {
				ui.painter().rect_stroke(
					face_coordinates.to_rect(),
					Rounding::default(),
					FACE_RECTANGLE_STROKE,
				);
			}
		});
		ctx.request_repaint();
	}
}
