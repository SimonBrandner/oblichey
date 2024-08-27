mod imported;

pub mod detector;
pub mod recognizer;

const WEIGHTS_DIRECTORY_NAME: &str = "weights";

fn get_weights_file(model_name: &str) -> String {
	let path_to_executable = std::env::current_exe().expect("Could not get path to executable!");
	let path_to_dir_with_executable = path_to_executable
		.parent()
		.expect("Could not get parent directory!");

	path_to_dir_with_executable
		.join(WEIGHTS_DIRECTORY_NAME)
		.join(String::new() + model_name + ".mpk")
		.to_str()
		.expect("Failed to get valid UTF-8 string from path!")
		.to_owned()
}
