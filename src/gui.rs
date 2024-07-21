use crate::{
	camera::{Camera, ImageSize},
	processor::{ProcessState, Processor},
};
use eframe::{
	egui::{self, Color32, ColorImage, Rounding, Stroke},
	NativeOptions,
};

const FACE_RECTANGLE_STROKE: Stroke = Stroke {
	width: 4.0,
	color: Color32::from_rgb(255, 0, 0),
};

pub fn start(camera: Camera<'static>, processor: Processor) {
	eframe::run_native(
		"GDay",
		NativeOptions::default(),
		Box::new(|_| Box::new(GUI::new(camera, processor))),
	)
}

struct GUI<'a> {
	camera: Camera<'a>,
	processor: Processor,
}

impl<'a> GUI<'a> {
	pub fn new(camera: Camera<'a>, processor: Processor) -> Self {
		Self { camera, processor }
	}
}

impl eframe::App for GUI<'_> {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		let image = match self.camera.get_frame() {
			Ok(b) => b,
			Err(e) => {
				println!("Failed to get frame: {e}");
				return;
			}
		};
		let egui_image =
			ColorImage::from_rgba_unmultiplied(image.get_size_array(), &image.clone().into_raw());
		let state = self.processor.process_frame(&image);

		match state.process_state {
			ProcessState::Scan(s) => {
				if let Some(_data) = s.data {
					println!("Scanned")
				}
			}
			ProcessState::Auth(s) => {
				if s.authenticated {
					println!("Authenticated")
				}
			}
		}

		egui::CentralPanel::default().show(ctx, |ui| {
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
