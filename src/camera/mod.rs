mod utils;

use crate::geometry::Vec2D;
use image::{ImageBuffer, ImageError, Rgb};
use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{io, usize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use utils::{brightness, convert_grey_to_rgb, convert_yuyv_to_rgb, reshape_frame};
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

// TODO: Make this user configurable
const CAMERA_PATH: &str = "/dev/video2";
/// Some (at least mine) IR cameras occasionally produce very dark frames which we ignore
const MAX_BRIGHTNESS_DECREASE: f32 = 24.0;

pub type Frame = ImageBuffer<Rgb<u8>, Vec<u8>>;

#[derive(Clone, Copy, Debug, EnumIter)]
pub enum SupportedPixelFormat {
	YUYV,
	GREY,
}

impl SupportedPixelFormat {
	pub fn to_fourcc(&self) -> FourCC {
		let bytes = match self {
			Self::GREY => b"GREY",
			Self::YUYV => b"YUYV",
		};
		FourCC::new(bytes)
	}
}

pub enum Error {
	Io(io::Error),
	Format(String),
	Image(ImageError),
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
	pixel_format: SupportedPixelFormat,
	frame_size: Vec2D<u32>,
}

impl<'a> Camera<'a> {
	pub fn new() -> Result<Self, Error> {
		let mut device = Device::with_path(CAMERA_PATH)?;
		let mut format = device.format()?;
		let frame_size = Vec2D::new(format.width, format.height);

		let mut chosen_pixel_format: Option<SupportedPixelFormat> = None;
		for pixel_format in SupportedPixelFormat::iter() {
			format.fourcc = pixel_format.to_fourcc();
			let new_format = device.set_format(&format)?;

			if new_format.fourcc == pixel_format.to_fourcc() {
				chosen_pixel_format = Some(pixel_format);
				break;
			}
		}
		let pixel_format = match chosen_pixel_format {
			Some(f) => f,
			None => {
				return Err(Error::Format(String::from(
					"Failed to set the desired format",
				)));
			}
		};

		Ok(Self {
			stream: Stream::with_buffers(&mut device, Type::VideoCapture, 4)?,
			pixel_format,
			frame_size,
		})
	}

	pub fn get_frame(&mut self) -> Result<Vec<u8>, Error> {
		let (frame_buffer, _) = self.stream.next()?;
		Ok(frame_buffer.to_vec())
	}

	pub fn get_pixel_format(&self) -> SupportedPixelFormat {
		self.pixel_format
	}

	pub fn get_frame_size(&self) -> Vec2D<u32> {
		self.frame_size
	}
}

pub fn start(frame: Arc<Mutex<Option<Frame>>>, finished: Arc<AtomicBool>) {
	let mut camera = match Camera::new() {
		Ok(c) => c,
		Err(e) => {
			finished.store(true, Ordering::SeqCst);
			panic!("Failed construct camera: {e}")
		}
	};
	let pixel_format = camera.get_pixel_format();
	let frame_size = camera.get_frame_size();

	let mut last_brightness = 255.0;
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

		let rgb_frame = match pixel_format {
			SupportedPixelFormat::YUYV => convert_yuyv_to_rgb(&new_frame, &frame_size),
			SupportedPixelFormat::GREY => {
				// We need to ignore very dark frames in some way. It's difficult to pick a single
				// threshold for "too dark", so we instead measure the brightness decrease
				let brightness = brightness(&new_frame, &frame_size);
				let brightness_decrease = last_brightness - brightness;
				last_brightness = brightness;
				if brightness_decrease > MAX_BRIGHTNESS_DECREASE {
					continue;
				}

				convert_grey_to_rgb(&new_frame, &frame_size)
			}
		};

		let reshaped_frame = reshape_frame(rgb_frame, &frame_size);

		let mut frame_lock = match frame.lock() {
			Ok(l) => l,
			Err(e) => {
				finished.store(true, Ordering::SeqCst);
				panic!("Failed to get frame lock: {e}");
			}
		};
		*frame_lock = Some(reshaped_frame);
		drop(frame_lock);
	}
}
