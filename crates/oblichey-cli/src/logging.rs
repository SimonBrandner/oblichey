#[macro_export]
macro_rules! log_and_print_error {
	($($arg:tt)*) => {
		let text = format!($($arg)*);
		log::error!("{}", text);
		eprintln!("{}", text);
	};
}

#[macro_export]
macro_rules! log_and_print_warn {
	($($arg:tt)*) => {
		let text = format!($($arg)*);
		log::warn!("{}", text);
		println!("{}", text);
	};
}
