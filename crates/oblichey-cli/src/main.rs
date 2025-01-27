mod camera;
mod config;
mod geometry;
mod gui;
mod logging;
mod models;
mod processors;
mod store;

use camera::Frame;
use clap::Parser;
use config::Config;
use flexi_logger::{FileSpec, Logger};
use log::trace;
use processors::auth_processor::AuthProcessor;
use processors::face::{FaceEmbedding, FaceForGUI};
use processors::face_processor::FaceProcessor;
use processors::scan_processor::ScanProcessor;
use std::collections::HashMap;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self};
use store::{get_log_directory, load_face_embeddings, remove_face_embedding, save_face_embedding};

const LOG_LEVEL: &str = "trace";
const LOG_FILE_BASE_NAME: &str = "oblichey";

#[derive(PartialEq, Eq, Debug, Clone, clap::Subcommand)]
enum Command {
	/// Scan a new face
	Scan {
		/// Name of the newly scanned face
		name: String,
	},
	/// Remove an existing scanned face
	Remove {
		/// Name of the scanned face to remove
		name: String,
	},
	/// List scanned faces
	List,
	/// Opens a window with the camera's annotated output which can be used for testing if everything is
	/// working as expected
	Test,
	/// This subcommand is mostly meant to be used by the PAM module. It authenticates the user.
	/// It will return 0, if authentication was successful, and a non-zero error code, if it failed
	Auth,
}

#[derive(clap::Parser, Debug)]
struct Args {
	#[command(subcommand)]
	command: Command,
}

fn main() -> ExitCode {
	let log_directory = get_log_directory().expect("Failed to get log directory");
	let log_spec = match Logger::try_with_str(LOG_LEVEL) {
		Ok(s) => s,
		Err(e) => {
			eprintln!("Failed to create log spec: {e}");
			return ExitCode::FAILURE;
		}
	};
	if let Err(e) = log_spec
		.log_to_file(
			FileSpec::default()
				.basename(LOG_FILE_BASE_NAME)
				.directory(log_directory),
		)
		.start()
	{
		eprintln!("Failed to start logger: {e}");
		return ExitCode::FAILURE;
	};
	log_panics::init();

	let args = Args::parse();
	let config = match Config::load() {
		Ok(c) => c,
		Err(e) => {
			log_and_print_error!("Failed to load config: {e}");
			return ExitCode::FAILURE;
		}
	};
	let face_embeddings = match load_face_embeddings() {
		Ok(e) => e,
		Err(e) => {
			log_and_print_error!("Failed to load face scans: {e}");
			return ExitCode::FAILURE;
		}
	};

	handle_command(args.command, &config, face_embeddings)
}

fn handle_command(
	command: Command,
	config: &Config,
	face_embeddings: HashMap<String, FaceEmbedding>,
) -> ExitCode {
	trace!("Handling command: {command:?}");

	match command {
		Command::Remove { name } => {
			if !face_embeddings.contains_key(&name) {
				log_and_print_error!("Face scan of this name does not exist.");
				return ExitCode::FAILURE;
			}
			if let Err(e) = remove_face_embedding(&name) {
				log_and_print_error!("Failed remove face scan: {e}");
				return ExitCode::FAILURE;
			}
		}
		Command::List => {
			for (name, _) in face_embeddings {
				println!("{name}",);
			}
		}
		Command::Test => {
			let auth_processor = Arc::new(Mutex::new(AuthProcessor::new(face_embeddings, true)));
			start_threads(auth_processor, config, true);
		}
		Command::Auth => {
			if face_embeddings.is_empty() {
				log_and_print_error!("No faces have been scanned yet");
				return ExitCode::FAILURE;
			}
			let auth_processor = Arc::new(Mutex::new(AuthProcessor::new(face_embeddings, false)));

			start_threads(auth_processor.clone(), config, false);

			let auth_processor_lock = match auth_processor.lock() {
				Ok(l) => l,
				Err(e) => {
					log_and_print_error!("Failed to get lock: {e}");
					return ExitCode::FAILURE;
				}
			};
			let Some(result) = auth_processor_lock.get_result() else {
				log_and_print_error!("Getting auth result failed!");
				return ExitCode::FAILURE;
			};
			drop(auth_processor_lock);
			if result.authenticated {
				println!("Authenticated!");
			} else {
				println!("Authentication failed!");
				return ExitCode::FAILURE;
			}
		}
		Command::Scan { name } => {
			if face_embeddings.contains_key(&name) {
				log_and_print_error!("Face of this name already exists. Either pick a different name or remove the existing face.");
				return ExitCode::FAILURE;
			}

			let scan_processor = Arc::new(Mutex::new(ScanProcessor::new()));
			start_threads(scan_processor.clone(), config, true);
			let scan_processor_lock = match scan_processor.lock() {
				Ok(l) => l,
				Err(e) => {
					log_and_print_error!("Failed to get lock: {e}");
					return ExitCode::FAILURE;
				}
			};
			let Some(result) = scan_processor_lock.get_result() else {
				log_and_print_error!("Getting auth result failed - this should never happen!");
				return ExitCode::FAILURE;
			};

			drop(scan_processor_lock);
			if let Err(e) = save_face_embedding(&name, &result.face_embedding) {
				log_and_print_error!("Failed to save face scan: {e}");
				return ExitCode::FAILURE;
			};
			println!("Face scan was successful!");
		}
	}

	ExitCode::SUCCESS
}

/// This starts multiple threads for: reading from camera, processing frames and running the models
/// on them and the GUI
fn start_threads(
	face_processor: Arc<Mutex<dyn FaceProcessor + Send + Sync>>,
	config: &Config,
	gui: bool,
) {
	trace!("Starting threads");

	let mut thread_handles = Vec::new();

	let frame: Arc<Mutex<Option<Frame>>> = Arc::new(Mutex::new(None));
	let faces_for_gui: Arc<Mutex<Vec<FaceForGUI>>> = Arc::new(Mutex::new(Vec::new()));
	let finished: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

	let frame_clone = frame.clone();
	let finished_clone = finished.clone();
	let camera_path_clone = config.camera.path.clone();
	thread_handles.push(thread::spawn(move || {
		camera::start(&frame_clone, &finished_clone, &camera_path_clone)
	}));

	let faces_for_gui_clone = faces_for_gui.clone();
	let finished_clone = finished.clone();
	let frame_clone = frame.clone();
	thread_handles.push(thread::spawn(move || {
		processors::start(
			&frame_clone,
			&faces_for_gui_clone,
			&finished_clone,
			&face_processor,
		)
	}));

	if gui {
		let finished_clone = finished.clone();
		thread_handles.push(thread::spawn(move || {
			gui::start(frame, faces_for_gui, finished_clone)
		}));
	}

	// Wait for any thread to finish
	loop {
		if thread_handles
			.iter()
			.any(std::thread::JoinHandle::is_finished)
		{
			finished.store(true, Ordering::SeqCst);
			break;
		}
	}

	// Join all threads and print any errors
	for thread_handle in thread_handles {
		match thread_handle.join() {
			Ok(Ok(())) => (),
			Ok(Err(e)) => {
				log_and_print_error!("Thread returned an error: {e}");
			}
			Err(e) => {
				if let Some(panic_msg) = e.downcast_ref::<String>() {
					log_and_print_error!("Thread panicked: {panic_msg}");
				} else {
					log_and_print_error!("Thread panicked but with an unknown error");
				}
			}
		}
	}
}
