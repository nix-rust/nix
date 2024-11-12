use std::ffi::CString;

#[cfg(target_os = "macos")]
pub fn openlog(ident: &str, logopt: LogFlags, facility: Facility) {
    let ident = CString::new(ident).expect("TODO: handle error");
    unsafe {
        libc::openlog(ident.as_ptr(), logopt.bits(), facility as libc::c_int)
    }
}

#[cfg(target_os = "macos")]
pub fn syslog(priority: libc::c_int, message: &str) {
    let formatter = CString::new("%s").expect("TODO: handle error");
    let message = CString::new(message).expect("TODO: handle error");
    unsafe { libc::syslog(priority, formatter.as_ptr(), message.as_ptr()) }
}

pub use self::consts::*;

#[cfg(target_os = "macos")]
mod consts {
    libc_bitflags! {
        pub struct LogFlags: libc::c_int {
            /// Log the process id with each message: useful for identifying instantiations of
            /// daemons.
            LOG_PID;
            /// If syslog() cannot pass the message to syslogd(8) it will attempt to write the
            /// message to the console ("/dev/console").
            LOG_CONS;
            /// Open the connection to syslogd(8) immediately. Normally the open is delayed until
            /// the first message is logged. Useful for programs that need to manage the order in
            /// which file descriptors are allocated.
            LOG_NDELAY;
            /// Write the message to standard error output as well to the system log.
            LOG_PERROR;
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(i32)]
    #[non_exhaustive]
    pub enum Severity {
        LOG_EMERG = libc::LOG_EMERG,
        LOG_ALERT = libc::LOG_ALERT,
        LOG_CRIT = libc::LOG_CRIT,
        LOG_ERR = libc::LOG_ERR,
        LOG_WARNING = libc::LOG_WARNING,
        LOG_NOTICE = libc::LOG_NOTICE,
        LOG_INFO = libc::LOG_INFO,
        LOG_DEBUG = libc::LOG_DEBUG,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(i32)]
    #[non_exhaustive]
    pub enum Facility {
        LOG_KERN = libc::LOG_KERN,
        LOG_USER = libc::LOG_USER,
        LOG_MAIL = libc::LOG_MAIL,
        LOG_DAEMON = libc::LOG_DAEMON,
        LOG_AUTH = libc::LOG_AUTH,
        LOG_SYSLOG = libc::LOG_SYSLOG,
        LOG_LPR = libc::LOG_LPR,
        LOG_NEWS = libc::LOG_NEWS,
        LOG_UUCP = libc::LOG_UUCP,
        LOG_LOCAL0 = libc::LOG_LOCAL0,
        LOG_LOCAL1 = libc::LOG_LOCAL1,
        LOG_LOCAL2 = libc::LOG_LOCAL2,
        LOG_LOCAL3 = libc::LOG_LOCAL3,
        LOG_LOCAL4 = libc::LOG_LOCAL4,
        LOG_LOCAL5 = libc::LOG_LOCAL5,
        LOG_LOCAL6 = libc::LOG_LOCAL6,
        LOG_LOCAL7 = libc::LOG_LOCAL7,
    }
}
