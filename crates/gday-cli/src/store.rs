use crate::processors::face::FaceEmbedding;
use bincode::{deserialize, serialize};
use std::{
	collections::HashMap,
	fs::{self, read_dir, remove_file},
	path::PathBuf,
};

fn get_embeddings_directory() -> PathBuf {
	PathBuf::from("/home/simon/Downloads/embeddings")
}

fn get_face_embedding_file_path(name: &str) -> PathBuf {
	get_embeddings_directory().join(name)
}

pub fn save_face_embedding(name: &str, face_embedding: &FaceEmbedding) {
	let path = get_face_embedding_file_path(name);
	let serialized: Vec<u8> = serialize(&face_embedding).expect("Failed to serialize!");
	fs::write(path, serialized).expect("Failed to write embedding!");
}

pub fn remove_face_embedding(name: &str) {
	let path = get_face_embedding_file_path(name);
	remove_file(path).expect("Failed to remove embedding file!");
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
		let serialized = fs::read(file.path()).expect("Failed to read embedding file!");
		let face_embedding =
			deserialize(&serialized).expect("Failed to deserialize embedding file!");
		face_embeddings.insert(
			file.file_name()
				.to_str()
				.expect("Failed to convert embedding file name to String!")
				.to_owned(),
			face_embedding,
		);
	}
	face_embeddings
}
