mod camera;
mod gui;
mod model;
mod no_gui;
mod processor;
mod utils;
use camera::Camera;
use clap::Parser;
use core::panic;
use processor::{Processor, ProcessorResult};
use std::rc::Rc;

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
	let processor = Rc::new(Processor::new_test());

	if args.no_gui {
		no_gui::start(camera, processor.clone());
	} else {
		gui::start(camera, processor.clone());
	}

	let state = match processor.get_result() {
		Some(s) => s,
		None => panic!("Result is None"),
	};
	match state {
		ProcessorResult::Scan(_s) => {
			todo!("Save data")
		}
		ProcessorResult::Auth(_s) => {
			todo!("Do something with PAM")
		}
	}
}
