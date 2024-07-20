use std::time::SystemTime;

use crate::camera::Camera;
use chrono::Utc;
use eframe::{
	egui::{self, ColorImage},
	NativeOptions,
};

pub fn start(camera: Camera<'static>) {
	eframe::run_native(
		"GDay",
		NativeOptions::default(),
		Box::new(|_| Box::new(GUI::new(camera))),
	)
}

struct GUI<'a> {
	camera: Camera<'a>,
}

impl<'a> GUI<'a> {
	pub fn new(camera: Camera<'a>) -> Self {
		Self { camera }
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
		let size = [image.width() as _, image.height() as _];
		let egui_image = ColorImage::from_rgba_unmultiplied(size, &image.into_raw());

		egui::CentralPanel::default().show(ctx, |ui| {
			let texture =
				ui.ctx()
					.load_texture("frame", egui_image, egui::TextureOptions::default());
			ui.image(&texture, ui.available_size());
		});
		ctx.request_repaint();
	}
}
