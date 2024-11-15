use nix::syslog::Priority;

#[cfg(target_os = "macos")]
#[test]
fn test_syslog_hello_world() {
    use nix::syslog::{openlog, syslog, Facility, LogFlags, Severity};

    let name = "test_syslog_hello_world";
    openlog(name, LogFlags::LOG_PID, Facility::LOG_USER).unwrap();
    syslog(Priority::from_severity(Severity::LOG_EMERG), "Hello, nix!");
}
