pub mod detector {
	include!(concat!(env!("OUT_DIR"), "/src/models/detector.rs"));
}

pub mod recognizer {
	include!(concat!(env!("OUT_DIR"), "/src/models/recognizer.rs"));
}
