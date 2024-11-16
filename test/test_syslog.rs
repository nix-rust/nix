use nix::syslog::{openlog, syslog, Facility, LogFlags, Severity};

#[test]
fn test_syslog_hello_world() {
    openlog(None, LogFlags::LOG_PID, Facility::LOG_USER).unwrap();
    syslog(Severity::LOG_EMERG, "Hello, nix!").unwrap();
}
