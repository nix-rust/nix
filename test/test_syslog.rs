use nix::syslog::{openlog, syslog, Facility, LogFlags, Priority, Severity};

#[test]
fn test_syslog_hello_world() {
    let name = "test_syslog_hello_world";
    let priority = Priority::from_severity(Severity::LOG_EMERG);
    openlog(name, LogFlags::LOG_PID, Facility::LOG_USER).unwrap();
    syslog(priority, "Hello, nix!").unwrap();
}
