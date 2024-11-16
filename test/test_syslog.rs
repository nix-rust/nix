use nix::syslog::{openlog, syslog, Facility, LogFlags, Severity};

#[test]
fn test_syslog_hello_world() {
    #[cfg(not(target_os = "haiku"))]
    let flags = LogFlags::LOG_PID;
    #[cfg(target_os = "haiku")]
    let flags = LogFlags::empty();

    openlog(None::<&str>, flags, Facility::LOG_USER).unwrap();
    syslog(Severity::LOG_EMERG, "Hello, nix!").unwrap();

    let name = "syslog";
    syslog(Severity::LOG_NOTICE, &format!("Hello, {name}!")).unwrap();
}
