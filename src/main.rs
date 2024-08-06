mod camera;
mod geometry;
mod gui;
mod models;
mod processors;

use camera::Frame;
use clap::Parser;
use core::panic;
use processors::face_processor::{AuthProcessor, FaceProcessor, ScanProcessor};
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

	match command {
		Command::Remove => {
			panic!("Not implemented!")
		}
		Command::List => {
			panic!("Not implemented!")
		}
		Command::Test => {
			let auth_processor = Arc::new(Mutex::new(AuthProcessor::new(false)));
			start_threads(auth_processor);
		}
		Command::Auth => {
			let auth_processor = Arc::new(Mutex::new(AuthProcessor::new(true)));
			start_threads(auth_processor.clone());
			let auth_processor_lock = match auth_processor.lock() {
				Ok(l) => l,
				Err(e) => panic!("{}", e),
			};
			let result = auth_processor_lock.get_result();
			println!("Scanning result: {:?}", result);
		}
		Command::Scan => {
			let scan_processor = Arc::new(Mutex::new(ScanProcessor::new()));
			start_threads(scan_processor.clone());
			let scan_processor_lock = match scan_processor.lock() {
				Ok(l) => l,
				Err(e) => panic!("{}", e),
			};
			let result = scan_processor_lock.get_result();
			println!("Scanning result: {:?}", result);
		}
	}
}

/// This starts multiple threads for: reading from camera, processing frames and running the models
/// on them and the GUI
fn start_threads(face_processor: Arc<Mutex<dyn FaceProcessor + Send + Sync>>) {
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

	let _ =
		thread::spawn(move || processors::start(frame, faces_for_gui, finished, face_processor))
			.join();
}
