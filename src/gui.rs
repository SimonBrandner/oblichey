use crate::{
	camera::{self, Frame},
	geometry::{Rectangle, Vec2D},
	processors::{frame_processor::MODEL_INPUT_IMAGE_SIZE, DetectedFace},
};
use eframe::{
	egui::{self, Color32, ColorImage, Pos2, Rect, Rounding, Stroke, Vec2},
	EventLoopBuilderHook, NativeOptions,
};
use std::{
	fmt::Display,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc, Mutex,
	},
};
use winit::platform::unix::EventLoopBuilderExtUnix;

trait ToVec2 {
	fn to_pos2(&self) -> Pos2;
}

impl ToVec2 for Vec2D {
	fn to_pos2(&self) -> Pos2 {
		Pos2 {
			x: self.x as f32,
			y: self.y as f32,
		}
	}
}

trait ToRect {
	fn to_rect(&self) -> Rect;
}

impl ToRect for Rectangle {
	fn to_rect(&self) -> Rect {
		Rect {
			min: self.min.to_pos2(),
			max: self.max.to_pos2(),
		}
	}
}

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
	frame: Arc<Mutex<Option<Frame>>>,
	detected_faces: Arc<Mutex<Vec<DetectedFace>>>,
	finished: Arc<AtomicBool>,
) -> Result<(), Error> {
	let event_loop_builder: Option<EventLoopBuilderHook> = Some(Box::new(|event_loop_builder| {
		event_loop_builder.with_any_thread(true);
	}));

	eframe::run_native(
		"Gday",
		NativeOptions {
			event_loop_builder,
			resizable: false,
			initial_window_size: Some(Vec2::new(
				MODEL_INPUT_IMAGE_SIZE.x as f32,
				MODEL_INPUT_IMAGE_SIZE.y as f32,
			)),
			..NativeOptions::default()
		},
		Box::new(|_| Box::new(GUI::new(frame, detected_faces, finished))),
	);
	Ok(())
}

struct GUI {
	frame: Arc<Mutex<Option<Frame>>>,
	detected_faces: Arc<Mutex<Vec<DetectedFace>>>,
	finished: Arc<AtomicBool>,
}

impl GUI {
	pub fn new(
		frame: Arc<Mutex<Option<Frame>>>,
		detected_faces: Arc<Mutex<Vec<DetectedFace>>>,
		finished: Arc<AtomicBool>,
	) -> Self {
		Self {
			frame,
			detected_faces,
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
			image.width() as usize,
			MODEL_INPUT_IMAGE_SIZE.x,
			"Image width does not match network requirements!"
		);
		assert_eq!(
			image.height() as usize,
			MODEL_INPUT_IMAGE_SIZE.y,
			"Image height does not match network requirements!"
		);

		let detected_faces_lock = match self.detected_faces.lock() {
			Ok(l) => l,
			Err(e) => {
				self.finished.store(true, Ordering::SeqCst);
				panic!("Failed to get detected faces lock: {e}");
			}
		};
		let detected_faces = detected_faces_lock.clone();
		drop(detected_faces_lock);

		egui::CentralPanel::default()
			.frame(egui::Frame::none().inner_margin(0.0).outer_margin(0.0))
			.show(ctx, |ui| {
				let egui_image = ColorImage::from_rgb(
					[MODEL_INPUT_IMAGE_SIZE.x, MODEL_INPUT_IMAGE_SIZE.y],
					&image.into_raw(),
				);
				let image_texture =
					ui.ctx()
						.load_texture("Camera", egui_image, egui::TextureOptions::default());

				ui.image(
					&image_texture,
					Vec2::new(
						MODEL_INPUT_IMAGE_SIZE.x as f32,
						MODEL_INPUT_IMAGE_SIZE.y as f32,
					),
				);
				for face in detected_faces {
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
