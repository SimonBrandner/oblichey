mod camera;
mod gui;
mod model;
mod no_gui;
mod processor;
mod utils;
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
		Err(e) => panic!("Failed cam {e}"),
	};
	let processor = Processor::new();

	if args.no_gui {
		no_gui::start(camera, processor);
	} else {
		gui::start(camera, processor);
	}
}
