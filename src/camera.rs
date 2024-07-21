use crate::utils::convert_yuyv_to_rgba;
use image::{ImageBuffer, ImageError, Rgba};
use std::fmt::Display;
use std::{io, u32};
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

const VIDEO_WIDTH: u32 = 1280;
const VIDEO_HEIGHT: u32 = 720;

pub enum Error {
	Io(io::Error),
	Format(String),
	Image(ImageError),
}

pub trait ImageSize {
	fn get_size_array(&self) -> [usize; 2];
}

impl ImageSize for ImageBuffer<Rgba<u8>, Vec<u8>> {
	fn get_size_array(&self) -> [usize; 2] {
		[VIDEO_WIDTH as _, VIDEO_HEIGHT as _]
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
		desired_format.fourcc = FourCC::new(b"YUYV");

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

	pub fn get_frame(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Error> {
		let (yuyv_frame_buffer, _) = self.stream.next()?;
		let rgba_frame_buffer = convert_yuyv_to_rgba(yuyv_frame_buffer, VIDEO_WIDTH, VIDEO_HEIGHT);
		Ok(rgba_frame_buffer)
	}
}
