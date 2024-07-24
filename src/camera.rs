use crate::utils::convert_yuyv_to_rgb;
use eframe::egui::Vec2;
use image::{ImageBuffer, ImageError, Rgb};
use std::fmt::Display;
use std::io;
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

const VIDEO_WIDTH: u32 = 640;
const VIDEO_HEIGHT: u32 = 480;
const FOUR_CC: &[u8; 4] = b"YUYV";

pub enum Error {
	Io(io::Error),
	Format(String),
	Image(ImageError),
}

pub trait ImageSize {
	fn get_size_array(&self) -> [usize; 2];
	fn get_size_vec2(&self) -> Vec2;
}

impl ImageSize for ImageBuffer<Rgb<u8>, Vec<u8>> {
	fn get_size_array(&self) -> [usize; 2] {
		[VIDEO_WIDTH as _, VIDEO_HEIGHT as _]
	}
	fn get_size_vec2(&self) -> Vec2 {
		Vec2::new(VIDEO_WIDTH as f32, VIDEO_HEIGHT as f32)
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
	device: Device,
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
			device,
		})
	}

	pub fn get_frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Error> {
		let (yuyv_frame_buffer, _) = self.stream.next()?;
		let rgb_frame_buffer = convert_yuyv_to_rgb(yuyv_frame_buffer, VIDEO_WIDTH, VIDEO_HEIGHT);
		Ok(rgb_frame_buffer)
	}

	pub fn get_output_size(&self) -> Result<Vec2D, Error> {
		let format = self.device.format()?;
		Ok(Vec2D {
			x: format.width as usize,
			y: format.height as usize,
		})
	}
}
