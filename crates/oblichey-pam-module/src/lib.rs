use pam::constants::{PamFlag, PamResultCode};
use pam::module::{PamHandle, PamHooks};
use std::ffi::CStr;
use std::io::{self, Write};
use std::process::Command;

const EXECUTABLE_PATH: &str = "oblichey-cli";

struct OblicheyPamModule;

pam::pam_hooks!(OblicheyPamModule);

impl PamHooks for OblicheyPamModule {
	fn sm_authenticate(_pamh: &mut PamHandle, _args: Vec<&CStr>, _flags: PamFlag) -> PamResultCode {
		println!("Starting face recognition");

		// This is one of the ugliest things I have done recently and there really ought to be a
		// way to do this other than calling another executable. Ideally, we would make the core
		// code into some kind of a shared library.
		match Command::new(EXECUTABLE_PATH).arg("auth").output() {
			Ok(o) => {
				if o.status.success() {
					println!("Face recognition successful");
					PamResultCode::PAM_SUCCESS
				} else {
					if let Err(e) = io::stdout().write_all(&o.stdout) {
						println!("Failed to print stdout: {e}");
					};
					if let Err(e) = io::stderr().write_all(&o.stderr) {
						println!("Failed to print stderr: {e}");
					};

					println!("Face recognition unsuccessful");
					PamResultCode::PAM_AUTH_ERR
				}
			}
			Err(e) => {
				println!("Running face recognition failed: {e}");
				PamResultCode::PAM_AUTH_ERR
			}
		}
	}
}
