use burn_import::onnx::ModelGen;

fn main() {
	ModelGen::new()
		.input("./models/detector.onnx")
		.out_dir("./src/models/")
		.run_from_script();

	ModelGen::new()
		.input("./models/recognizer.onnx")
		.out_dir("./src/models/")
		.run_from_script();
}
