use burn_import::onnx::ModelGen;

fn main() {
	ModelGen::new()
		.input("src/face_onnx_models/detector.onnx")
		.out_dir("models/")
		.run_from_script();

	ModelGen::new()
		.input("src/face_onnx_models/recognizer.onnx")
		.out_dir("models/")
		.run_from_script();
}
