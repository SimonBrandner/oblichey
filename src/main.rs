mod camera;
mod model;
mod processor;

use camera::Camera;
use clap::Parser;
use core::panic;
use processor::Processor;

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
}

fn main() {
	let args = Args::parse();
	let command = args.command;

	if command != Command::Test {
		panic!("Not implemented!");
	}

	let mut camera = match Camera::new() {
		Ok(c) => c,
		Err(e) => panic!("Failed cam {e}"),
	};
	let processor = Processor::new();

	loop {
		let frame_buffer = match camera.get_frame() {
			Ok(b) => b,
			Err(e) => {
				println!("Failed to get frame: {e}");
				continue;
			}
		};
		let _state = processor.process_frame(&frame_buffer);
	}
}
