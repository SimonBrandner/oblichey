mod utils;

use crate::geometry::Vec2D;
use image::{ImageBuffer, ImageError, Rgb};
use log::{error, trace};
use std::fmt::Display;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use utils::{brightness, convert_grey_to_rgb, convert_yuyv_to_rgb, reshape_frame};
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

/// Some (at least mine) IR cameras occasionally produce very dark frames which we ignore
const MAX_BRIGHTNESS_DECREASE: f32 = 24.0;
/// How many times we are allowed to fail getting a frame before we panic
const MAX_FAILED_FRAMES_IN_ROW: u8 = 10;

/// The type of a frame coming from the camera
pub type Frame = ImageBuffer<Rgb<u8>, Vec<u8>>;

#[derive(Clone, Copy, Debug, EnumIter, PartialEq, Eq)]
pub enum SupportedPixelFormat {
	Yuyv,
	Gray,
}

impl SupportedPixelFormat {
	pub fn to_fourcc(self) -> FourCC {
		let bytes = match self {
			Self::Gray => b"GREY",
			Self::Yuyv => b"YUYV",
		};
		FourCC::new(bytes)
	}
}

pub enum Error {
	Io(io::Error),
	Image(ImageError),
	CannotSetFormat,
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
			Self::CannotSetFormat => write!(f, "Failed to set desired format"),
			Self::Image(e) => write!(f, "Image error: {e}"),
		}
	}
}

pub struct Camera {
	stream: Stream<'static>,
	pixel_format: SupportedPixelFormat,
	frame_size: Vec2D<u32>,
}

impl Camera {
	/// Creates a new Camera which can be used to get frames from the given device.
	///
	/// This is going to panic if a supported output pixel format cannot be found
	pub fn new(camera_path: &str) -> Result<Self, Error> {
		trace!("Creating Camera");

		let device = Device::with_path(camera_path)?;
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
		let Some(pixel_format) = chosen_pixel_format else {
			return Err(Error::CannotSetFormat);
		};

		trace!("Chosen camera format: {format:?}");

		#[cfg(not(feature = "rgb-webcam"))]
		assert_eq!(
			pixel_format,
			SupportedPixelFormat::Gray,
			"Your camera does not appear to be support IR!"
		);

		Ok(Self {
			stream: Stream::with_buffers(&device, Type::VideoCapture, 4)?,
			pixel_format,
			frame_size,
		})
	}

	pub fn get_frame(&mut self) -> Result<Vec<u8>, Error> {
		let (frame_buffer, _) = self.stream.next()?;
		Ok(frame_buffer.to_vec())
	}
}

/// Starts the camera loop
pub fn start(frame: &Arc<Mutex<Option<Frame>>>, finished: &Arc<AtomicBool>, camera_path: &str) {
	let mut camera = match Camera::new(camera_path) {
		Ok(c) => c,
		Err(e) => panic!("Failed construct camera: {e}"),
	};

	let mut failed_frames_in_row = 0;
	let mut last_brightness = 255.0;
	loop {
		if finished.load(Ordering::SeqCst) {
			return;
		}

		let new_frame = match camera.get_frame() {
			Ok(f) => f,
			Err(e) => {
				error!("Failed to get frame: {e}");

				failed_frames_in_row += 1;
				assert!(
					failed_frames_in_row < MAX_FAILED_FRAMES_IN_ROW,
					"Failed to get {MAX_FAILED_FRAMES_IN_ROW} frames in row: {e}"
				);
				continue;
			}
		};

		let rgb_frame = match camera.pixel_format {
			SupportedPixelFormat::Yuyv => convert_yuyv_to_rgb(&new_frame, camera.frame_size),
			SupportedPixelFormat::Gray => {
				// We need to ignore very dark frames in some way. It's difficult to pick a single
				// threshold for "too dark", so we instead measure the brightness decrease
				let brightness = brightness(&new_frame, camera.frame_size);
				let brightness_decrease = last_brightness - brightness;
				last_brightness = brightness;
				if brightness_decrease > MAX_BRIGHTNESS_DECREASE {
					continue;
				}

				convert_grey_to_rgb(&new_frame, camera.frame_size)
			}
		};

		let reshaped_frame = reshape_frame(&rgb_frame, camera.frame_size);

		let mut frame_lock = match frame.lock() {
			Ok(l) => l,
			Err(e) => panic!("Failed to get lock: {e}"),
		};
		*frame_lock = Some(reshaped_frame);
		drop(frame_lock);
	}
}
