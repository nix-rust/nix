use nix::syslog::{openlog, syslog, Facility, LogFlags, Severity};
use std::ffi::CStr;

#[test]
fn test_syslog_hello_world() {
    let flags = LogFlags::LOG_PID;
    openlog(None, flags, Facility::LOG_USER).unwrap();
    syslog(Severity::LOG_EMERG, "Hello, nix!").unwrap();

    let name = "syslog";
    syslog(Severity::LOG_NOTICE, &format!("Hello, {name}!")).unwrap();
}

#[test]
fn test_openlog_with_ident() {
    const IDENT: &CStr = unsafe {
        CStr::from_bytes_with_nul_unchecked(b"test_openlog_with_ident\0")
    };

    let flags = LogFlags::LOG_PID;
    openlog(Some(IDENT), flags, Facility::LOG_USER).unwrap();
    syslog(Severity::LOG_EMERG, "Hello, ident!").unwrap();
}
