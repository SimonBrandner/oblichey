mod geometry;
mod poi;

use crate::{
	camera::{self, Frame},
	gui::{
		geometry::{RectangleToEgui, Vec2DToEgui},
		poi::draw_poi_square,
	},
	models::detector::DETECTOR_INPUT_SIZE,
	processors::face::{FaceForGUI, FaceForGUIAnnotation, FaceForGUIAnnotationWarning},
};
use eframe::{
	egui::{
		self, Align2, Color32, ColorImage, FontFamily, FontId, Rounding, Ui, Vec2, ViewportBuilder,
	},
	EventLoopBuilderHook, NativeOptions,
};
use log::{trace, warn};
use std::{
	fmt::Display,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc, Mutex,
	},
};
use winit::platform::wayland;
use winit::platform::x11;

const FACE_RECTANGLE_WHITE_COLOR: Color32 = Color32::from_rgb(255, 255, 255);
const FACE_RECTANGLE_GREY_COLOR: Color32 = Color32::from_rgb(192, 192, 192);
const FACE_RECTANGLE_YELLOW_COLOR: Color32 = Color32::from_rgb(255, 255, 0);
const LABEL_SHIFT: Vec2 = Vec2::new(10.0, 0.0);

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

/// Start the GUI loop
pub fn start(
	frame: Arc<Mutex<Option<Frame>>>,
	faces: Arc<Mutex<Vec<FaceForGUI>>>,
	finished: Arc<AtomicBool>,
) {
	trace!("Creating GUI");

	let event_loop_builder: Option<EventLoopBuilderHook> = Some(Box::new(|event_loop_builder| {
		wayland::EventLoopBuilderExtWayland::with_any_thread(event_loop_builder, true);
		x11::EventLoopBuilderExtX11::with_any_thread(event_loop_builder, true);
	}));
	let viewport = ViewportBuilder::default()
		.with_resizable(false)
		.with_inner_size(
			DETECTOR_INPUT_SIZE
				.to_vec2()
				.expect("Failed to convert Vec2D to Vec2!"),
		);

	if let Err(e) = eframe::run_native(
		"Oblichey",
		NativeOptions {
			viewport,
			event_loop_builder,
			..NativeOptions::default()
		},
		Box::new(|_| Ok(Box::new(Gui::new(frame, faces, finished)))),
	) {
		panic!("Running eframe failed: {e}");
	};
}

struct Gui {
	frame: Arc<Mutex<Option<Frame>>>,
	faces: Arc<Mutex<Vec<FaceForGUI>>>,
	finished: Arc<AtomicBool>,
}

impl Gui {
	pub fn new(
		frame: Arc<Mutex<Option<Frame>>>,
		faces: Arc<Mutex<Vec<FaceForGUI>>>,
		finished: Arc<AtomicBool>,
	) -> Self {
		Self {
			frame,
			faces,
			finished,
		}
	}
}

impl Gui {
	/// Draws the window contents
	fn draw(ctx: &egui::Context, frame: Frame, faces_for_gui: Vec<FaceForGUI>) {
		egui::CentralPanel::default()
			.frame(egui::Frame::none().inner_margin(0.0).outer_margin(0.0))
			.show(ctx, |ui| {
				Self::draw_frame(ui, frame);
				for face_for_gui in faces_for_gui {
					Self::draw_face(ui, face_for_gui);
				}
			});
	}

	/// Draws the frame
	fn draw_frame(ui: &mut Ui, frame: Frame) {
		let egui_image = ColorImage::from_rgb(
			[
				DETECTOR_INPUT_SIZE.x as usize,
				DETECTOR_INPUT_SIZE.y as usize,
			],
			&frame.into_raw(),
		);
		let image_texture =
			ui.ctx()
				.load_texture("Camera", egui_image, egui::TextureOptions::default());

		ui.image(&image_texture);
	}

	/// Draws a face
	fn draw_face(ui: &Ui, face_for_gui: FaceForGUI) {
		let (text, color) = match face_for_gui.annotation {
			FaceForGUIAnnotation::Name(n) => (n, FACE_RECTANGLE_YELLOW_COLOR),
			FaceForGUIAnnotation::Warning(w) => match w {
				FaceForGUIAnnotationWarning::TooSmall => {
					("Too small".to_owned(), FACE_RECTANGLE_GREY_COLOR)
				}
				FaceForGUIAnnotationWarning::NotRecognized => {
					("Not recognized".to_owned(), FACE_RECTANGLE_GREY_COLOR)
				}
				FaceForGUIAnnotationWarning::TooManyFaces => {
					("Too many faces".to_owned(), FACE_RECTANGLE_GREY_COLOR)
				}
			},
			FaceForGUIAnnotation::ScanningState {
				scanned_sample_count,
				required_sample_count,
			} => (
				format!("Scanning: {scanned_sample_count}/{required_sample_count}",),
				FACE_RECTANGLE_WHITE_COLOR,
			),
		};
		let (rectangles, top_right_position) = draw_poi_square(face_for_gui.rectangle);

		ui.painter().text(
			top_right_position
				.to_pos2()
				.expect("Failed to convert Vec2D to Pos2!")
				+ LABEL_SHIFT,
			Align2::LEFT_TOP,
			text,
			FontId::new(16.0, FontFamily::Monospace),
			FACE_RECTANGLE_WHITE_COLOR,
		);
		for rectangle in rectangles {
			ui.painter().rect_filled(
				rectangle
					.to_rect()
					.expect("Failed to convert Rectangle to Rect!"),
				Rounding::default(),
				color,
			);
		}
	}
}

impl eframe::App for Gui {
	fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
		if self.finished.load(Ordering::SeqCst) {
			trace!("Sending viewport command to close window");
			ctx.send_viewport_cmd(egui::ViewportCommand::Close);
			return;
		}

		let frame_lock = match self.frame.lock() {
			Ok(l) => l,
			Err(e) => panic!("Failed to get lock: {e}"),
		};
		let Some(image) = frame_lock.clone() else {
			warn!("Do not have a frame to render");
			ctx.request_repaint();
			return;
		};
		drop(frame_lock);

		assert_eq!(
			image.width(),
			DETECTOR_INPUT_SIZE.x,
			"Image width does not match network requirements!"
		);
		assert_eq!(
			image.height(),
			DETECTOR_INPUT_SIZE.y,
			"Image height does not match network requirements!"
		);

		let faces_for_gui_lock = match self.faces.lock() {
			Ok(l) => l,
			Err(e) => panic!("Failed to get lock: {e}"),
		};
		let faces_for_gui = faces_for_gui_lock.clone();
		drop(faces_for_gui_lock);

		Self::draw(ctx, image, faces_for_gui);
		ctx.request_repaint();
	}
}
