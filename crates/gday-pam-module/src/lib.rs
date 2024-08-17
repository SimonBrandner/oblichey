use pam::constants::{PamFlag, PamResultCode};
use pam::module::{PamHandle, PamHooks};
use std::ffi::CStr;
use std::io::{self, Write};
use std::process::Command;

const EXECUTABLE_PATH: &str = "/home/simon/GIT/Rust/gday/result/bin/gday-cli";

struct GdayPamModule;

pam::pam_hooks!(GdayPamModule);

impl PamHooks for GdayPamModule {
	fn sm_authenticate(_pamh: &mut PamHandle, _args: Vec<&CStr>, _flags: PamFlag) -> PamResultCode {
		println!("Starting face recognition");

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
