mod camera;
mod geometry;
mod gui;
mod models;
mod processors;
mod store;

use crate::store::{load_face_embeddings, remove_face_embedding};
use camera::Frame;
use clap::Parser;
use core::panic;
use processors::face::FaceForGUI;
use processors::face_processor::{AuthProcessor, FaceProcessor, ScanProcessor};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread::{self};
use store::save_face_embedding;

#[derive(PartialEq, Eq, Debug, Clone, clap::Subcommand)]
enum Command {
	Scan { name: String },
	Remove { name: String },
	Auth,
	Test,
	List,
}

#[derive(clap::Parser, Debug)]
struct Args {
	#[command(subcommand)]
	command: Command,
}

fn main() {
	let args = Args::parse();
	let command = args.command;

	match command {
		Command::Remove { name } => {
			remove_face_embedding(&name);
		}
		Command::List => {
			let face_embeddings = load_face_embeddings();
			for (name, _) in face_embeddings {
				println!("{}", name);
			}
		}
		Command::Test => {
			let face_embeddings = load_face_embeddings();
			let auth_processor = Arc::new(Mutex::new(AuthProcessor::new(face_embeddings, true)));

			start_threads(auth_processor);
		}
		Command::Auth => {
			let face_embeddings = load_face_embeddings();
			let auth_processor = Arc::new(Mutex::new(AuthProcessor::new(face_embeddings, false)));

			start_threads(auth_processor.clone());

			let auth_processor_lock = match auth_processor.lock() {
				Ok(l) => l,
				Err(e) => panic!("{}", e),
			};
			let result = match auth_processor_lock.get_result() {
				Some(r) => r,
				None => unreachable!(),
			};
			if result.authenticated {
				println!("Authenticated!");
			} else {
				println!("Authentication failed!");
			}
		}
		Command::Scan { name } => {
			let scan_processor = Arc::new(Mutex::new(ScanProcessor::new()));
			start_threads(scan_processor.clone());
			let scan_processor_lock = match scan_processor.lock() {
				Ok(l) => l,
				Err(e) => panic!("{}", e),
			};
			let result = match scan_processor_lock.get_result() {
				Some(r) => r,
				None => panic!("Scanning ended with no result!"),
			};
			save_face_embedding(&name, &result.face_embedding);
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
