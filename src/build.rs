use burn_import::onnx::ModelGen;
use std::{
	env,
	fs::{self, create_dir},
	path::Path,
};

const MODEL_NAMES: [&str; 2] = ["detector", "recognizer"];
const MODELS_OUT_DIR: &str = "src/models";
const ONNX_DIR: &str = "models";
const WEIGHTS_DIR: &str = "weights";

fn main() {
	println!("cargo::rerun-if-changed=models");

	let out_dir = env::var("OUT_DIR").expect("OUT_DIR not defined");
	let profile = env::var("PROFILE").expect("PROFILE not defined");

	let source_weights_dir = Path::new(&out_dir).join(MODELS_OUT_DIR);
	let new_weights_dir = Path::new("target").join(profile).join(WEIGHTS_DIR);

	// Create the weights directory unless it already exists
	let _ = create_dir(new_weights_dir.clone());

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
