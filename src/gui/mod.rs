mod poi;

use crate::{
	camera::{self, Frame},
	geometry::{Rectangle, Vec2D, Vec2DNumber},
	gui::poi::draw_poi_square,
	processors::{frame_processor::DETECTOR_INPUT_SIZE, FaceForGUI, FaceForGUIAnnotation},
};
use eframe::{
	egui::{self, Align2, Color32, ColorImage, FontFamily, FontId, Pos2, Rect, Rounding, Vec2},
	EventLoopBuilderHook, NativeOptions,
};
use num::NumCast;
use std::{
	cmp::{max, min},
	fmt::Display,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc, Mutex,
	},
};
use winit::platform::unix::EventLoopBuilderExtUnix;

const FACE_RECTANGLE_WHITE_COLOR: Color32 = Color32::from_rgb(255, 255, 255);
const FACE_RECTANGLE_GREY_COLOR: Color32 = Color32::from_rgb(192, 192, 192);
const FACE_RECTANGLE_YELLOW_COLOR: Color32 = Color32::from_rgb(255, 255, 0);
const LABEL_SHIFT: Vec2 = Vec2::new(10.0, 0.0);

trait ToVec2 {
	fn to_pos2(&self) -> Pos2;
}

impl<T: Vec2DNumber> ToVec2 for Vec2D<T> {
	fn to_pos2(&self) -> Pos2 {
		Pos2 {
			x: <f32 as NumCast>::from(self.x.clone()).unwrap_or(0.0),
			y: <f32 as NumCast>::from(self.y.clone()).unwrap_or(0.0),
		}
	}
}

trait ToRect {
	fn to_rect(&self) -> Rect;
}

impl<T: Vec2DNumber> ToRect for Rectangle<T> {
	fn to_rect(&self) -> Rect {
		// EGUI requires min to be the top left corner and max to be the bottom right corner
		let min_position = Vec2D::new(min(self.min.x, self.max.x), min(self.min.y, self.max.y));
		let max_position = Vec2D::new(max(self.min.x, self.max.x), max(self.min.y, self.max.y));

		Rect {
			min: min_position.to_pos2(),
			max: max_position.to_pos2(),
		}
	}
}

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
	frame: Arc<Mutex<Option<Frame>>>,
	faces_for_gui: Arc<Mutex<Vec<FaceForGUI>>>,
	finished: Arc<AtomicBool>,
) {
	let event_loop_builder: Option<EventLoopBuilderHook> = Some(Box::new(|event_loop_builder| {
		event_loop_builder.with_any_thread(true);
	}));

	eframe::run_native(
		"Gday",
		NativeOptions {
			event_loop_builder,
			resizable: false,
			initial_window_size: Some(Vec2::new(
				DETECTOR_INPUT_SIZE.x as f32,
				DETECTOR_INPUT_SIZE.y as f32,
			)),
			..NativeOptions::default()
		},
		Box::new(|_| Box::new(GUI::new(frame, faces_for_gui, finished))),
	);
}

struct GUI {
	frame: Arc<Mutex<Option<Frame>>>,
	faces_for_gui: Arc<Mutex<Vec<FaceForGUI>>>,
	finished: Arc<AtomicBool>,
}

impl GUI {
	pub fn new(
		frame: Arc<Mutex<Option<Frame>>>,
		faces_for_gui: Arc<Mutex<Vec<FaceForGUI>>>,
		finished: Arc<AtomicBool>,
	) -> Self {
		Self {
			frame,
			faces_for_gui,
			finished,
		}
	}
}

impl eframe::App for GUI {
	fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
		self.finished.store(true, Ordering::SeqCst);
	}

	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		if self.finished.load(Ordering::SeqCst) {
			frame.close();
			return;
		}

		let frame_lock = match self.frame.lock() {
			Ok(l) => l,
			Err(e) => {
				self.finished.store(true, Ordering::SeqCst);
				panic!("Failed to get frame lock: {e}");
			}
		};
		let image = match frame_lock.clone() {
			Some(f) => f.clone(),
			None => {
				ctx.request_repaint();
				return;
			}
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

		let faces_for_gui_lock = match self.faces_for_gui.lock() {
			Ok(l) => l,
			Err(e) => {
				self.finished.store(true, Ordering::SeqCst);
				panic!("Failed to get detected faces lock: {e}");
			}
		};
		let faces_for_gui = faces_for_gui_lock.clone();
		drop(faces_for_gui_lock);

		egui::CentralPanel::default()
			.frame(egui::Frame::none().inner_margin(0.0).outer_margin(0.0))
			.show(ctx, |ui| {
				let egui_image = ColorImage::from_rgb(
					[
						DETECTOR_INPUT_SIZE.x as usize,
						DETECTOR_INPUT_SIZE.y as usize,
					],
					&image.into_raw(),
				);
				let image_texture =
					ui.ctx()
						.load_texture("Camera", egui_image, egui::TextureOptions::default());

				ui.image(
					&image_texture,
					Vec2::new(DETECTOR_INPUT_SIZE.x as f32, DETECTOR_INPUT_SIZE.y as f32),
				);
				for face_for_gui in faces_for_gui {
					let (text, color) = match face_for_gui.annotation {
						FaceForGUIAnnotation::Name(n) => (n, FACE_RECTANGLE_YELLOW_COLOR),
						FaceForGUIAnnotation::Warning(w) => (w, FACE_RECTANGLE_GREY_COLOR),
						FaceForGUIAnnotation::ScanningState {
							scanned_sample_count,
							required_sample_count,
						} => (
							format!(
								"Scanning: {}/{}",
								scanned_sample_count, required_sample_count
							),
							FACE_RECTANGLE_WHITE_COLOR,
						),
					};
					let (rectangles, top_right_position) = draw_poi_square(face_for_gui.rectangle);

					ui.painter().text(
						top_right_position.to_pos2() + LABEL_SHIFT,
						Align2::LEFT_TOP,
						text,
						FontId::new(16.0, FontFamily::Monospace),
						FACE_RECTANGLE_WHITE_COLOR,
					);
					for rectangle in rectangles {
						ui.painter()
							.rect_filled(rectangle.to_rect(), Rounding::default(), color);
					}
				}
			});
		ctx.request_repaint();
	}
}
