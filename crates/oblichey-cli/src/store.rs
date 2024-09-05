use crate::processors::face::FaceEmbedding;
use bincode::{deserialize, serialize};
use std::{
	collections::HashMap,
	env,
	fmt::Display,
	fs::{self, create_dir, read_dir, remove_file},
	io,
	path::PathBuf,
};

const OBLICHEY_DIRECTORY_NAME: &str = "oblichey";

#[derive(Debug)]
pub enum Error {
	EnvVar(env::VarError),
	Io(io::Error),
	Bincode(bincode::Error),
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::EnvVar(e) => write!(f, "Environment variable error: {e}"),
			Self::Io(e) => write!(f, "IO error: {e}"),
			Self::Bincode(e) => write!(f, "Bincode error: {e}"),
		}
	}
}

impl From<env::VarError> for Error {
	fn from(value: env::VarError) -> Self {
		Self::EnvVar(value)
	}
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl From<bincode::Error> for Error {
	fn from(value: bincode::Error) -> Self {
		Self::Bincode(value)
	}
}

fn get_embeddings_directory() -> Result<PathBuf, Error> {
	let state_dir = env::var("XDG_STATE_HOME")?;
	let embeddings_dir_path = PathBuf::from(state_dir).join(OBLICHEY_DIRECTORY_NAME);

	if let Err(e) = create_dir(embeddings_dir_path.clone()) {
		if e.kind() != io::ErrorKind::AlreadyExists {
			return Err(Error::from(e));
		}
	};

	Ok(embeddings_dir_path)
}

fn get_face_embedding_file_path(name: &str) -> Result<PathBuf, Error> {
	Ok(get_embeddings_directory()?.join(name))
}

pub fn get_log_directory() -> Result<PathBuf, Error> {
	let cache_dir = env::var("XDG_CACHE_HOME")?;
	let log_dir_path = PathBuf::from(cache_dir).join(OBLICHEY_DIRECTORY_NAME);

	Ok(log_dir_path)
}

pub fn save_face_embedding(name: &str, face_embedding: &FaceEmbedding) -> Result<(), Error> {
	let path = get_face_embedding_file_path(name)?;
	let serialized: Vec<u8> = serialize(&face_embedding)?;
	fs::write(path, serialized)?;

	Ok(())
}

pub fn remove_face_embedding(name: &str) -> Result<(), Error> {
	let path = get_face_embedding_file_path(name)?;
	remove_file(path)?;

	Ok(())
}

pub fn load_face_embeddings() -> Result<HashMap<String, FaceEmbedding>, Error> {
	let path = get_embeddings_directory()?;
	let files = read_dir(path)?;

	let mut face_embeddings = HashMap::new();
	for file in files {
		let file = match file {
			Ok(f) => f,
			Err(e) => {
				println!("Failed to get file - skipping: {e}");
				continue;
			}
		};
		let Ok(name) = file.file_name().into_string() else {
			println!("File name contains invalid unicode - skipping");
			continue;
		};
		let serialized = fs::read(file.path())?;
		let face_embedding = deserialize(&serialized)?;

		face_embeddings.insert(name, face_embedding);
	}

	Ok(face_embeddings)
}
