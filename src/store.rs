use crate::processors::face::FaceEmbedding;
use bincode::{deserialize, serialize};
use core::panic;
use std::{
	collections::HashMap,
	fs::{self, read_dir, remove_file},
	path::PathBuf,
};

fn get_embeddings_directory() -> PathBuf {
	PathBuf::from("./embeddings")
}

fn get_face_embedding_file_path(name: &str) -> PathBuf {
	get_embeddings_directory().join(name)
}

pub fn save_face_embedding(name: &str, face_embedding: &FaceEmbedding) {
	let path = get_face_embedding_file_path(name);
	let serialized: Vec<u8> = serialize(&face_embedding).unwrap();
	fs::write(&path, serialized).unwrap();
}

pub fn remove_face_embedding(name: &str) {
	let path = get_face_embedding_file_path(name);
	remove_file(path).unwrap();
}

pub fn load_face_embeddings() -> HashMap<String, FaceEmbedding> {
	let path = get_embeddings_directory();
	let files = match read_dir(path) {
		Ok(f) => f,
		Err(e) => panic!("Failed to read directory: {e}"),
	};

	let mut face_embeddings = HashMap::new();
	for file in files {
		let file = match file {
			Ok(f) => f,
			Err(e) => panic!("Failed to get file: {e}"),
		};
		let serialized = fs::read(file.path()).unwrap();
		let face_embedding = deserialize(&serialized).unwrap();
		face_embeddings.insert(
			file.file_name().to_str().unwrap().to_owned(),
			face_embedding,
		);
	}
	face_embeddings
}
