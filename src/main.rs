mod camera;
mod geometry;
mod gui;
mod models;
mod processors;

use camera::Frame;
use clap::Parser;
use core::panic;
use processors::DetectedFace;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread::{self};

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

	let frame: Arc<Mutex<Option<Frame>>> = Arc::new(Mutex::new(None));
	let detected_faces: Arc<Mutex<Vec<DetectedFace>>> = Arc::new(Mutex::new(Vec::new()));
	let finished: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

	let frame_clone = frame.clone();
	let finished_clone = finished.clone();
	thread::spawn(move || camera::start(frame_clone, finished_clone));

	if !args.no_gui {
		let detected_faces_clone = detected_faces.clone();
		let frame_clone = frame.clone();
		let finished_clone = finished.clone();
		thread::spawn(move || gui::start(frame_clone, detected_faces_clone, finished_clone));
	}

	let _ = thread::spawn(move || processors::start(frame, detected_faces, finished)).join();
}
