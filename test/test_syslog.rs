#[cfg(target_os = "macos")]
#[test]
fn test_syslog_hello_world() {
    use nix::syslog::{openlog, syslog, Facility, LogFlags, Severity};

    openlog(
        "test_syslog_hello_world",
        LogFlags::LOG_PID,
        Facility::LOG_USER,
    );
    syslog(Severity::LOG_ERR as libc::c_int, "Hello, nix!");
}
