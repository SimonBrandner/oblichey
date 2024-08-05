mod camera;
mod geometry;
mod gui;
mod models;
mod processors;

use camera::Frame;
use clap::Parser;
use core::panic;
use processors::FaceForGUI;
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
}

fn main() {
	let args = Args::parse();
	let command = args.command;

	if command != Command::Test {
		panic!("Not implemented!");
	}

	let frame: Arc<Mutex<Option<Frame>>> = Arc::new(Mutex::new(None));
	let faces_for_gui: Arc<Mutex<Vec<FaceForGUI>>> = Arc::new(Mutex::new(Vec::new()));
	let finished: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

	let frame_clone = frame.clone();
	let finished_clone = finished.clone();
	thread::spawn(move || camera::start(frame_clone, finished_clone));

	let faces_for_gui_clone = faces_for_gui.clone();
	let frame_clone = frame.clone();
	let finished_clone = finished.clone();
	thread::spawn(move || gui::start(frame_clone, faces_for_gui_clone, finished_clone));

	let _ = thread::spawn(move || processors::start(frame, faces_for_gui, finished)).join();
}
