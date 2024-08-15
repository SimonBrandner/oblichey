use pam::constants::{PamFlag, PamResultCode};
use pam::module::{PamHandle, PamHooks};
use std::ffi::CStr;

struct GdayPamModule;
pam::pam_hooks!(GdayPamModule);

impl PamHooks for GdayPamModule {
	fn sm_authenticate(_pamh: &mut PamHandle, _args: Vec<&CStr>, _flags: PamFlag) -> PamResultCode {
		PamResultCode::PAM_AUTH_ERR
	}

	fn sm_setcred(_pamh: &mut PamHandle, _args: Vec<&CStr>, _flags: PamFlag) -> PamResultCode {
		PamResultCode::PAM_SUCCESS
	}

	fn acct_mgmt(_pamh: &mut PamHandle, _args: Vec<&CStr>, _flags: PamFlag) -> PamResultCode {
		PamResultCode::PAM_SUCCESS
	}
}
