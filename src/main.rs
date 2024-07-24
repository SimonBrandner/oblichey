mod camera;
mod main_loop;
mod model;
mod processors;
mod types;
mod utils;

use crate::main_loop::{gui, no_gui};
use crate::processors::{face_processor::AuthProcessor, frame_processor::FrameProcessor};
use camera::Camera;
use clap::Parser;
use core::panic;

#[derive(PartialEq, Eq, Debug, Clone, clap::ValueEnum)]
enum Command {
	Auth,
	Scan,
	Test,
	Remove,
	List,
}

#[derive(clap::Parser, Debug)]
struct Args {
	#[arg(value_enum)]
	command: Command,

	#[arg(short, long, default_value = "false")]
	no_gui: bool,
}

fn main() {
	let args = Args::parse();
	let command = args.command;

	if command != Command::Test {
		panic!("Not implemented!");
	}

	let camera = match Camera::new() {
		Ok(c) => c,
		Err(e) => panic!("Failed construct camera: {e}"),
	};

	let face_processor = Box::new(AuthProcessor::new());
	let frame_processor = FrameProcessor::new();

	if args.no_gui {
		no_gui::start(camera, frame_processor, face_processor);
	} else {
		if let Err(e) = gui::start(camera, frame_processor, face_processor) {
			panic!("Error during running GUI: {e}");
		}
	}
}
