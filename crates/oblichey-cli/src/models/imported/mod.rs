#![allow(clippy::all)]
#![allow(clippy::nursery)]
#![allow(clippy::pedantic)]

pub mod detector {
	include!(concat!(env!("OUT_DIR"), "/src/models/imported/detector.rs"));
}

pub mod recognizer {
	include!(concat!(
		env!("OUT_DIR"),
		"/src/models/imported/recognizer.rs"
	));
}
