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
use std::sync::atomic::{AtomicBool, Ordering};
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
				println!("{name}",);
			}
		}
		Command::Test => {
			let face_embeddings = load_face_embeddings();
			let auth_processor = Arc::new(Mutex::new(AuthProcessor::new(face_embeddings, true)));

			start_threads(auth_processor, true);
		}
		Command::Auth => {
			let face_embeddings = load_face_embeddings();
			let auth_processor = Arc::new(Mutex::new(AuthProcessor::new(face_embeddings, false)));

			start_threads(auth_processor.clone(), false);

			let auth_processor_lock = match auth_processor.lock() {
				Ok(l) => l,
				Err(e) => panic!("{}", e),
			};
			let Some(result) = auth_processor_lock.get_result() else {
				unreachable!()
			};
			drop(auth_processor_lock);
			if result.authenticated {
				println!("Authenticated!");
			} else {
				println!("Authentication failed!");
			}
		}
		Command::Scan { name } => {
			let scan_processor = Arc::new(Mutex::new(ScanProcessor::new()));
			start_threads(scan_processor.clone(), true);
			let scan_processor_lock = match scan_processor.lock() {
				Ok(l) => l,
				Err(e) => panic!("{}", e),
			};
			let Some(result) = scan_processor_lock.get_result() else {
				panic!("Scanning ended with no result!")
			};

			drop(scan_processor_lock);
			save_face_embedding(&name, &result.face_embedding);
		}
	}
}

/// This starts multiple threads for: reading from camera, processing frames and running the models
/// on them and the GUI
fn start_threads(face_processor: Arc<Mutex<dyn FaceProcessor + Send + Sync>>, gui: bool) {
	let frame: Arc<Mutex<Option<Frame>>> = Arc::new(Mutex::new(None));
	let faces_for_gui: Arc<Mutex<Vec<FaceForGUI>>> = Arc::new(Mutex::new(Vec::new()));
	let finished: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

	let frame_clone = frame.clone();
	let finished_clone = finished.clone();
	let camera_thread = thread::spawn(move || camera::start(&frame_clone, &finished_clone));

	let faces_for_gui_clone = faces_for_gui.clone();
	let finished_clone = finished.clone();
	let frame_clone = frame.clone();
	let processing_thread = thread::spawn(move || {
		processors::start(
			&frame_clone,
			&faces_for_gui_clone,
			&finished_clone,
			&face_processor,
		);
	});

	let gui_thread = gui.then(|| {
		let finished_clone = finished.clone();
		thread::spawn(move || {
			gui::start(frame, faces_for_gui, finished_clone);
		})
	});

	loop {
		let camera_finished = camera_thread.is_finished();
		let processing_finished = processing_thread.is_finished();
		let gui_finished = gui_thread
			.as_ref()
			.map_or(true, thread::JoinHandle::is_finished);

		if camera_finished || processing_finished || (gui_finished && gui_thread.is_some()) {
			finished.store(true, Ordering::SeqCst);
		}
		if camera_finished && processing_finished && gui_finished {
			break;
		}
	}
}
