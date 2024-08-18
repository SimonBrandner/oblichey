use serde::Deserialize;
use std::{fmt::Display, fs, io};

const CONFIG_PATH: &str = "/etc/oblichey.toml";

pub enum Error {
	TomlDeserialize(toml::de::Error),
	Io(io::Error),
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::TomlDeserialize(e) => write!(f, "TOML deserialization failed: {e}"),
			Self::Io(e) => write!(f, "IO error: {e}"),
		}
	}
}

impl From<toml::de::Error> for Error {
	fn from(value: toml::de::Error) -> Self {
		Self::TomlDeserialize(value)
	}
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

#[derive(Deserialize, Clone)]
pub struct CameraConfig {
	pub path: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
	pub camera: CameraConfig,
}

impl Config {
	pub fn load() -> Result<Self, Error> {
		let serialized = fs::read_to_string(CONFIG_PATH)?;
		Ok(toml::from_str(&serialized)?)
	}
}
