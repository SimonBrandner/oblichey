use image::codecs::jpeg::JpegDecoder;
use image::{DynamicImage, ImageError, RgbaImage};
use std::fmt::Display;
use std::io;
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

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
}

impl<'a> Camera<'a> {
	pub fn new() -> Result<Self, Error> {
		let mut device = Device::new(0)?;

		let mut desired_format = device.format()?;
		desired_format.width = 1280;
		desired_format.height = 720;
		desired_format.fourcc = FourCC::new(b"MJPG");

		let actual_format = device.set_format(&desired_format)?;
		if actual_format.width != desired_format.width
			|| actual_format.height != desired_format.height
			|| actual_format.fourcc != desired_format.fourcc
		{
			return Err(Error::Format(String::from(
				"Failed to set the desired format",
			)));
		}
		println!("{}", actual_format.fourcc);

		Ok(Self {
			stream: Stream::with_buffers(&mut device, Type::VideoCapture, 4)?,
		})
	}

	pub fn get_frame(&mut self) -> Result<RgbaImage, Error> {
		let (frame_buffer, _) = self.stream.next()?;
		let decoder = JpegDecoder::new(frame_buffer)?;
		Ok(DynamicImage::from_decoder(decoder)?.to_rgba8())
	}
}
