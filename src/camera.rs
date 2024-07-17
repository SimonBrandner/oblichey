use std::fmt::Display;
use std::io;
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::Device;
use v4l::FourCC;

pub enum Error {
	Io(io::Error),
	Format(String),
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Io(e) => write!(f, "IO error: {e}"),
			Self::Format(e) => write!(f, "Format error: {e}"),
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
		desired_format.fourcc = FourCC::new(b"YUYV");

		let actual_format = device.set_format(&desired_format)?;
		if actual_format.to_string() != desired_format.to_string() {
			return Err(Error::Format(String::from(
				"Failed to set the desired format",
			)));
		}

		Ok(Self {
			stream: Stream::with_buffers(&mut device, Type::VideoCapture, 4)?,
		})
	}

	pub fn get_frame(&mut self) -> Result<Vec<u8>, Error> {
		let (buf, _) = self.stream.next()?;
		Ok(buf.to_vec())
	}
}
