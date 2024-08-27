use burn_import::onnx::ModelGen;
use merkle_hash::{bytes_to_hex, Algorithm, MerkleTree};
use std::{
	env,
	fs::{self, create_dir},
	io::ErrorKind,
	path::Path,
};

const MODEL_NAMES: [&str; 2] = ["detector", "recognizer"];
const MODELS_OUT_DIR: &str = "src/models/imported";
const ONNX_DIR: &str = "models";
const ONNX_DIR_HASH: &str = "89bccae035e26f635866d12e0b5c030cfa2e7bf7a8889812043c0c08cf7e6126";
const WEIGHTS_DIR: &str = "weights";
const TARGET_DIR_ERROR: &str =
	"Failed to get OUT_DIR parent when trying to get to target directory!";
const TRY_UNZIPPING_MODELS: &str = "Try running `oblichey/scripts/unzip_models.sh`";

fn main() {
	println!("cargo::rerun-if-changed=models");
	let tree = match MerkleTree::builder(ONNX_DIR)
		.algorithm(Algorithm::Blake3)
		.hash_names(false)
		.build()
	{
		Ok(t) => t,
		Err(e) => panic!("Failed to compute tree for directory hash. {TRY_UNZIPPING_MODELS}: {e}"),
	};
	assert_eq!(
		bytes_to_hex(tree.root.item.hash),
		ONNX_DIR_HASH,
		"Directory hashes do not match. {TRY_UNZIPPING_MODELS}"
	);

	let out_dir_str = env::var("OUT_DIR").expect("OUT_DIR not defined");
	let out_dir_path = Path::new(&out_dir_str);
	let target_dir_path = out_dir_path
		.parent()
		.expect(TARGET_DIR_ERROR)
		.parent()
		.expect(TARGET_DIR_ERROR)
		.parent()
		.expect(TARGET_DIR_ERROR);

	let source_weights_dir = out_dir_path.join(MODELS_OUT_DIR);
	let new_weights_dir = target_dir_path.join(WEIGHTS_DIR);

	// Create the weights directory unless it already exists
	if let Err(e) = create_dir(new_weights_dir.clone()) {
		assert_eq!(
			e.kind(),
			ErrorKind::AlreadyExists,
			"Failed to create weights directory: {e}"
		);
	};

	for model_name in MODEL_NAMES {
		import_onnx_model(model_name);
		copy_weights_next_to_executable(model_name, &source_weights_dir, &new_weights_dir);
	}
}

fn get_onnx_file_path(model_name: &str) -> String {
	format!("{ONNX_DIR}/{model_name}.onnx")
}

fn import_onnx_model(model_name: &str) {
	ModelGen::new()
		.input(&get_onnx_file_path(model_name))
		.out_dir(MODELS_OUT_DIR)
		.run_from_script();
}

fn copy_weights_next_to_executable(
	model_name: &str,
	source_weights_dir: &Path,
	new_weights_dir: &Path,
) {
	let weights_file_name = format!("{model_name}.mpk",);
	let source_path = source_weights_dir.join(weights_file_name.clone());
	let destination_path = new_weights_dir.join(weights_file_name);

	fs::copy(source_path, destination_path)
		.expect("Failed to copy weights to the weights directory!");
}
