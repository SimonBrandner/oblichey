pub mod detector {
	include!(concat!(env!("OUT_DIR"), "/models/detector.rs"));
}

pub mod recognizer {
	include!(concat!(env!("OUT_DIR"), "/models/recognizer.rs"));
}
