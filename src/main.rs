mod camera;
mod gui;
mod model;
mod processor;

use camera::Camera;
use clap::Parser;
use core::panic;
//use chrono::Utc;
//use image::GenericImageView;
//use processor::Processor;
//use std::usize;

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

	#[arg(short, long, default_value = "true")]
	gui: bool,
}

fn main() {
	let args = Args::parse();
	let command = args.command;

	if command != Command::Test {
		panic!("Not implemented!");
	}

	let camera = match Camera::new() {
		Ok(c) => c,
		Err(e) => panic!("Failed cam {e}"),
	};

	if args.gui {
		gui::main(camera);
	}

	//let processor = Processor::new();

	//loop {
	//	let image = match camera.get_frame() {
	//		Ok(b) => b,
	//		Err(e) => {
	//			println!("Failed to get frame: {e}");
	//			return;
	//		}
	//	};
	//
	//	let _state = processor.process_frame(&image);
	//}
}
