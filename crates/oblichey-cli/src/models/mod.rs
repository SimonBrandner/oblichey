mod imported;

pub mod detector;
pub mod recognizer;

const WEIGHTS_DIRECTORY_NAME: &str = "weights";

fn get_weights_file(model_name: &str) -> String {
	const PARENT_DIR_ERROR: &str = "Could not get parent directory!";

	let mut path = std::env::current_exe().expect("Could not get path to executable!");
	path = path.parent().expect(PARENT_DIR_ERROR).to_path_buf();

	// In tests we need to go further up for some reason
	#[cfg(test)]
	{
		path = path.parent().expect(PARENT_DIR_ERROR).to_path_buf();
	}

	path.join(WEIGHTS_DIRECTORY_NAME)
		.join(String::new() + model_name + ".mpk")
		.to_str()
		.expect("Failed to get valid UTF-8 string from path!")
		.to_owned()
}
