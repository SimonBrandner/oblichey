use eframe::{egui, NativeOptions};

pub fn main() {
	eframe::run_native(
		"GDay",
		NativeOptions::default(),
		Box::new(|_| Box::new(GUI::default())),
	)
}

#[derive(Default)]
struct GUI;

impl eframe::App for GUI {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {});
	}
}
