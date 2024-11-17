//! Interfaces for controlling system log.

use crate::{NixPath, Result};
use std::ffi::OsStr;
use std::ptr;

/// Logging options of subsequent [`syslog`] calls can be set by calling [`openlog`].
///
/// The parameter `ident` is a string that will be prepended to every message. The `logopt`
/// argument specifies logging options. The `facility` parameter encodes a default facility to be
/// assigned to all messages that do not have an explicit facility encoded.
//
// On Linux, the `ident` argument needs to have static lifetime according to the
// man page:
//
// The argument ident in the call of openlog() is probably stored as-is. Thus,
// if the string it points to is changed, syslog() may start prepending the changed
// string, and if the string it points to ceases to exist, the results are
// undefined.  Most portable is to use a string constant.
#[cfg(target_os = "linux")]
pub fn openlog(
    ident: Option<&'static std::ffi::CStr>,
    logopt: LogFlags,
    facility: Facility,
) -> Result<()> {
    let logopt = logopt.bits();
    let facility = facility as libc::c_int;
    match ident {
        None => unsafe {
            libc::openlog(ptr::null(), logopt, facility);
        },
        Some(ident) => ident.with_nix_path(|ident| unsafe {
            libc::openlog(ident.as_ptr(), logopt, facility);
        })?,
    }

    Ok(())
}

/// Logging options of subsequent [`syslog`] calls can be set by calling [`openlog`].
///
/// The parameter `ident` is a string that will be prepended to every message. The `logopt`
/// argument specifies logging options. The `facility` parameter encodes a default facility to be
/// assigned to all messages that do not have an explicit facility encoded.
#[cfg(not(target_os = "linux"))]
pub fn openlog<S: AsRef<OsStr> + ?Sized>(
    ident: Option<&S>,
    logopt: LogFlags,
    facility: Facility,
) -> Result<()> {
    let logopt = logopt.bits();
    let facility = facility as libc::c_int;
    match ident.map(OsStr::new) {
        None => unsafe {
            libc::openlog(ptr::null(), logopt, facility);
        },
        Some(ident) => ident.with_nix_path(|ident| unsafe {
            libc::openlog(ident.as_ptr(), logopt, facility);
        })?,
    }

    Ok(())
}

/// Writes message to the system message logger.
///
/// The message is then written to the system console, log files, logged-in users, or forwarded
/// to other machines as appropriate.
///
/// # Examples
///
/// ```rust
/// use nix::syslog::{openlog, syslog, Facility, LogFlags, Severity, Priority};
///
/// let priority = Priority::new(Severity::LOG_EMERG, Facility::LOG_USER);
/// syslog(priority, "Hello, nix!").unwrap();
///
/// // use `format!` to format the message
/// let name = "syslog";
/// syslog(priority, &format!("Hello, {name}!")).unwrap();
/// ```
pub fn syslog<P, S>(priority: P, message: &S) -> Result<()>
where
    P: Into<Priority>,
    S: AsRef<OsStr> + ?Sized,
{
    let priority = priority.into();
    let formatter = OsStr::new("%s");
    let message = OsStr::new(message);
    formatter.with_nix_path(|formatter| {
        message.with_nix_path(|message| unsafe {
            libc::syslog(priority.0, formatter.as_ptr(), message.as_ptr())
        })
    })??;
    Ok(())
}

/// Closes the log file.
pub fn closelog() {
    unsafe { libc::closelog() }
}

/// The priority for a log message.
#[derive(Debug, Clone, Copy)]
pub struct Priority(libc::c_int);

impl Priority {
    /// Create a new priority from a facility and severity level.
    pub fn new(severity: Severity, facility: Facility) -> Self {
        let priority = (facility as libc::c_int) | (severity as libc::c_int);
        Priority(priority)
    }
}

impl From<Severity> for Priority {
    fn from(severity: Severity) -> Self {
        let priority = severity as libc::c_int;
        Priority(priority)
    }
}

libc_bitflags! {
    /// Options for system logging.
    pub struct LogFlags: libc::c_int {
        /// Log the process id with each message: useful for identifying instantiations of
        /// daemons.
        LOG_PID;
        /// If syslog() cannot pass the message to syslogd(8) it will attempt to write the
        /// message to the console ("/dev/console").
        LOG_CONS;
        /// The converse of [`LOG_NDELAY`][LogFlags::LOG_NDELAY]; opening of the connection is
        /// delayed until `syslog` is called.
        ///
        /// This is the default, and need not be specified.
        LOG_ODELAY;
        /// Open the connection to syslogd(8) immediately. Normally the open is delayed until
        /// the first message is logged. Useful for programs that need to manage the order in
        /// which file descriptors are allocated.
        LOG_NDELAY;
        /// Write the message to standard error output as well to the system log.
        #[cfg(not(any(target_os = "redox", target_os = "illumos")))]
        LOG_PERROR;
    }
}

libc_enum! {
    /// Severity levels for log messages.
    #[repr(i32)]
    #[non_exhaustive]
    pub enum Severity {
        /// A panic condition.
        ///
        /// This is normally broadcast to all users.
        LOG_EMERG,
        /// A condition that should be corrected immediately, such as a corrupted system database.
        LOG_ALERT,
        /// Critical conditions, e.g., hard device errors.
        LOG_CRIT,
        /// Errors.
        LOG_ERR,
        /// Warning messages.
        LOG_WARNING,
        /// Conditions that are not error conditions, but should possibly be handled specially.
        LOG_NOTICE,
        /// Informational messages.
        LOG_INFO,
        /// Messages that contain information normally of use only when debugging a program.
        LOG_DEBUG,
    }
}

libc_enum! {
    /// Facilities for log messages.
    #[repr(i32)]
    #[non_exhaustive]
    pub enum Facility {
        /// Messages generated by the kernel.
        ///
        /// These cannot be generated by any user processes.
        LOG_KERN,
        /// Messages generated by random user processes.
        ///
        /// This is the default facility identifier if none is specified.
        LOG_USER,
        /// The mail system.
        LOG_MAIL,
        /// System daemons, such as routed(8), that are not provided for explicitly by other facilities.
        LOG_DAEMON,
        /// The authorization system: login(1), su(1), getty(8), etc.
        LOG_AUTH,
        /// Messages generated internally by syslogd(8).
        LOG_SYSLOG,
        /// The line printer spooling system: cups-lpd(8), cupsd(8), etc.
        LOG_LPR,
        /// The network news system.
        LOG_NEWS,
        /// The uucp system.
        LOG_UUCP,
        /// Reserved for local use.
        LOG_LOCAL0,
        /// Reserved for local use.
        LOG_LOCAL1,
        /// Reserved for local use.
        LOG_LOCAL2,
        /// Reserved for local use.
        LOG_LOCAL3,
        /// Reserved for local use.
        LOG_LOCAL4,
        /// Reserved for local use.
        LOG_LOCAL5,
        /// Reserved for local use.
        LOG_LOCAL6,
        /// Reserved for local use.
        LOG_LOCAL7,
    }
}
