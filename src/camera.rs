use crate::geometry::Vec2D;
use crate::utils::convert_yuyv_to_rgb;
use eframe::egui::Vec2;
use image::{ImageBuffer, ImageError, Rgb};
use std::fmt::Display;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

pub type Frame = ImageBuffer<Rgb<u8>, Vec<u8>>;
pub const VIDEO_WIDTH: u32 = 640;
pub const VIDEO_HEIGHT: u32 = 480;
const FOUR_CC: &[u8; 4] = b"YUYV";

pub enum Error {
	Io(io::Error),
	Format(String),
	Image(ImageError),
}

pub trait ImageSize {
	#[allow(non_snake_case)]
	fn get_size_vec2D(&self) -> Vec2D;
	fn get_size_array(&self) -> [usize; 2];
	fn get_size_vec2(&self) -> Vec2;
}

impl ImageSize for Frame {
	fn get_size_array(&self) -> [usize; 2] {
		[VIDEO_WIDTH as _, VIDEO_HEIGHT as _]
	}
	fn get_size_vec2(&self) -> Vec2 {
		Vec2::new(VIDEO_WIDTH as f32, VIDEO_HEIGHT as f32)
	}
	fn get_size_vec2D(&self) -> Vec2D {
		Vec2D {
			x: VIDEO_WIDTH as usize,
			y: VIDEO_HEIGHT as usize,
		}
	}
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl From<ImageError> for Error {
	fn from(value: ImageError) -> Self {
		Self::Image(value)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Io(e) => write!(f, "IO error: {e}"),
			Self::Format(e) => write!(f, "Format error: {e}"),
			Self::Image(e) => write!(f, "Image error: {e}"),
		}
	}
}

pub struct Camera<'a> {
	stream: Stream<'a>,
}

impl<'a> Camera<'a> {
	pub fn new() -> Result<Self, Error> {
		let mut device = Device::new(0)?;

		let mut desired_format = device.format()?;
		desired_format.width = VIDEO_WIDTH;
		desired_format.height = VIDEO_HEIGHT;
		desired_format.fourcc = FourCC::new(FOUR_CC);

		let actual_format = device.set_format(&desired_format)?;
		if actual_format.width != desired_format.width
			|| actual_format.height != desired_format.height
			|| actual_format.fourcc != desired_format.fourcc
		{
			return Err(Error::Format(String::from(
				"Failed to set the desired format",
			)));
		}

		Ok(Self {
			stream: Stream::with_buffers(&mut device, Type::VideoCapture, 4)?,
		})
	}

	pub fn get_frame(&mut self) -> Result<Frame, Error> {
		let (yuyv_frame_buffer, _) = self.stream.next()?;
		let rgb_frame_buffer = convert_yuyv_to_rgb(yuyv_frame_buffer, VIDEO_WIDTH, VIDEO_HEIGHT);
		Ok(rgb_frame_buffer)
	}
}

pub fn start(frame: Arc<Mutex<Option<Frame>>>, finished: Arc<AtomicBool>) {
	let mut camera = match Camera::new() {
		Ok(c) => c,
		Err(e) => panic!("Failed construct camera: {e}"),
	};

	loop {
		if finished.load(Ordering::SeqCst) {
			return;
		}

		let new_frame = match camera.get_frame() {
			Ok(f) => f,
			Err(e) => {
				finished.store(true, Ordering::SeqCst);
				panic!("Failed to get frame: {e}");
			}
		};
		let mut frame_lock = match frame.lock() {
			Ok(l) => l,
			Err(e) => {
				finished.store(true, Ordering::SeqCst);
				panic!("Failed to get frame lock: {e}");
			}
		};
		*frame_lock = Some(new_frame);
		drop(frame_lock);
	}
}
