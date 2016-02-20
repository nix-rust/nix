use nix::net::if_::*;
use std::ffi::CStr;

#[test]
fn test_if_nametoindex() {
    #[cfg(target_os = "linux")]
    fn loopback_name() -> &'static CStr { cstr!("lo") }

    #[cfg(not(target_os = "linux"))]
    fn loopback_name() -> &'static CStr { cstr!("lo0") }

    assert!(if_nametoindex(loopback_name()).is_ok());
}
