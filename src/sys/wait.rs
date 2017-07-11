//! This module contains the `wait()` and `waitpid()` functions, which are used to wait on and
//! obtain status information from child processes. These provide more granular control over
//! child management than the primitives provided by Rust's standard library, and are critical
//! in the creation of shells and managing jobs on *nix platforms.
//!
//! # Example
//!
//! ```rust
//! use nix::sys::wait::*;
//! loop {
//!     match waitpid(PidGroup::ProcessGroupID(pid), WUNTRACED) {
//!         Ok(WaitStatus::Exited(pid, status)) => {
//!             println!("Process '{}' exited with status '{}'", pid, status);
//!             break
//!         },
//!         Ok(WaitStatus::Stopped(pid, signal)) => {
//!             println!("Process '{}' stopped with signal '{}'", pid, signal);
//!         },
//!         Ok(WaitStatus::Continued(pid)) => {
//!             println!("Process '{}' continued", pid);
//!         },
//!         Ok(_) => (),
//!         Err(why) => {
//!             println!("waitpid returned an error code: {}" why);
//!             break
//!         }
//!     }
//! }
//! ```

use libc::{self, c_int, pid_t};
use {Errno, Result};
use unistd::Pid;

use sys::signal::Signal;

libc_bitflags!(
    /// Defines optional flags for the `waitpid` function.
    pub flags WaitPidFlag: c_int {
        /// Do not suspend execution of the calling thread if the status is not immediately
        /// available for one of the child processes specified by pid.
        WNOHANG,
        /// The status of any child processes specified by pid that are stopped, and whose status
        /// has not yet been reported since they stopped, shall also be reported to the requesting
        /// process
        WUNTRACED,
        /// Waits for children that have terminated.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        WEXITED,
        /// Report the status of any continued child process specified by pid whose status has not
        /// been reported since it continued from a job control stop.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        WCONTINUED,
        /// Leave the child in a waitable state; a later wait call can be used to again retrieve
        /// the child status information.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        WNOWAIT,
        /// Don't wait on children of other threads in this group
        #[cfg(any(target_os = "android", target_os = "linux"))]
        __WNOTHREAD,
        /// Wait for all children, regardless of type (clone or non-clone)
        #[cfg(any(target_os = "android", target_os = "linux"))]
        __WALL,
        /// Wait for "clone" children only. If omitted then wait for "non-clone" children only.
        /// (A "clone" child is one which delivers no signal, or a signal other than `SIGCHLD` to
        /// its parent upon termination.) This option is ignored if `__WALL` is also specified.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        __WCLONE,

    }
);

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
/// Contains the status returned by the `wait` and `waitpid` functions.
pub enum WaitStatus {
    /// Signifies that the process has exited, providing the PID and associated exit status.
    Exited(Pid, i8),
    /// Signifies that the process was killed by a signal, providing the PID, the associated
    /// signal, and a boolean value that is set to `true` when a core dump was produced.
    Signaled(Pid, Signal, bool),
    /// Signifies that the process was stopped by a signal, providing the PID and associated
    /// signal.
    Stopped(Pid, Signal),
    /// Signifies that the process was stopped due to a ptrace event, providing the PID, the
    /// associated signal, and an integer that represents the status of the event.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    PtraceEvent(Pid, Signal, c_int),
    /// Signifies that the process received a `SIGCONT` signal, and thus continued.
    Continued(Pid),
    /// If `WNOHANG` was set, this value is returned when no children have changed state.
    StillAlive
}

#[cfg(any(target_os = "android", target_os = "linux"))]
mod status {
    use sys::signal::Signal;
    use libc::c_int;

    pub fn exited(status: i32) -> bool {
        (status & 0x7F) == 0
    }

    pub fn exit_status(status: i32) -> i8 {
        ((status & 0xFF00) >> 8) as i8
    }

    pub fn signaled(status: i32) -> bool {
        ((((status & 0x7f) + 1) as i8) >> 1) > 0
    }

    pub fn term_signal(status: i32) -> Signal {
        Signal::from_c_int(status & 0x7f).unwrap()
    }

    pub fn dumped_core(status: i32) -> bool {
        (status & 0x80) != 0
    }

    pub fn stopped(status: i32) -> bool {
        (status & 0xff) == 0x7f
    }

    pub fn stop_signal(status: i32) -> Signal {
        Signal::from_c_int((status & 0xFF00) >> 8).unwrap()
    }

    pub fn stop_additional(status: i32) -> c_int {
        (status >> 16) as c_int
    }

    pub fn continued(status: i32) -> bool {
        status == 0xFFFF
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod status {
    use sys::signal::{Signal,SIGCONT};

    const WCOREFLAG: i32 = 0x80;
    const WSTOPPED: i32 = 0x7f;

    fn wstatus(status: i32) -> i32 {
        status & 0x7F
    }

    pub fn exit_status(status: i32) -> i8 {
        ((status >> 8) & 0xFF) as i8
    }

    pub fn stop_signal(status: i32) -> Signal {
        Signal::from_c_int(status >> 8).unwrap()
    }

    pub fn continued(status: i32) -> bool {
        wstatus(status) == WSTOPPED && stop_signal(status) == SIGCONT
    }

    pub fn stopped(status: i32) -> bool {
        wstatus(status) == WSTOPPED && stop_signal(status) != SIGCONT
    }

    pub fn exited(status: i32) -> bool {
        wstatus(status) == 0
    }

    pub fn signaled(status: i32) -> bool {
        wstatus(status) != WSTOPPED && wstatus(status) != 0
    }

    pub fn term_signal(status: i32) -> Signal {
        Signal::from_c_int(wstatus(status)).unwrap()
    }

    pub fn dumped_core(status: i32) -> bool {
        (status & WCOREFLAG) != 0
    }
}

#[cfg(any(target_os = "freebsd",
          target_os = "openbsd",
          target_os = "dragonfly",
          target_os = "netbsd"))]
mod status {
    use sys::signal::Signal;

    const WCOREFLAG: i32 = 0x80;
    const WSTOPPED: i32 = 0x7f;

    fn wstatus(status: i32) -> i32 {
        status & 0x7F
    }

    pub fn stopped(status: i32) -> bool {
        wstatus(status) == WSTOPPED
    }

    pub fn stop_signal(status: i32) -> Signal {
        Signal::from_c_int(status >> 8).unwrap()
    }

    pub fn signaled(status: i32) -> bool {
        wstatus(status) != WSTOPPED && wstatus(status) != 0 && status != 0x13
    }

    pub fn term_signal(status: i32) -> Signal {
        Signal::from_c_int(wstatus(status)).unwrap()
    }

    pub fn exited(status: i32) -> bool {
        wstatus(status) == 0
    }

    pub fn exit_status(status: i32) -> i8 {
        (status >> 8) as i8
    }

    pub fn continued(status: i32) -> bool {
        status == 0x13
    }

    pub fn dumped_core(status: i32) -> bool {
        (status & WCOREFLAG) != 0
    }
}

fn decode(pid : Pid, status: i32) -> WaitStatus {
    if status::exited(status) {
        WaitStatus::Exited(pid, status::exit_status(status))
    } else if status::signaled(status) {
        WaitStatus::Signaled(pid, status::term_signal(status), status::dumped_core(status))
    } else if status::stopped(status) {
        cfg_if! {
            if #[cfg(any(target_os = "linux", target_os = "android"))] {
                fn decode_stopped(pid: Pid, status: i32) -> WaitStatus {
                    let status = status::stop_additional(status);
                    let signal = status::stop_signal(status);
                    if status == 0 {
                        WaitStatus::Stopped(pid, signal)
                    } else {
                        WaitStatus::PtraceEvent(pid, signal, status)
                    }
                }
            } else {
                fn decode_stopped(pid: Pid, status: i32) -> WaitStatus {
                    WaitStatus::Stopped(pid, status::stop_signal(status))
                }
            }
        }
        decode_stopped(pid, status)
    } else {
        assert!(status::continued(status));
        WaitStatus::Continued(pid)
    }
}

/// Designates whether the supplied `Pid` value is a process ID, process group ID,
/// specifies any child of the current process's group ID, or any child of the current process.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PidGroup {
    /// Signifies that the `Pid` is supposed to represent a process ID.
    ProcessID(Pid),
    /// Signifies that the `Pid` is supposed to represent a process group ID.
    ProcessGroupID(Pid),
    /// Signifies that any child process that belongs to the process group of the current process
    /// should be selected.
    AnyGroupChild,
    /// Signifies that any child process that belongs to the current process should be selected.
    AnyChild,
}

impl From<i32> for PidGroup {
    fn from(pid: i32) -> PidGroup {
        if pid > 0 {
            PidGroup::ProcessID(Pid::from_raw(pid))
        } else if pid < -1 {
            PidGroup::ProcessGroupID(Pid::from_raw(pid))
        } else if pid == 0 {
            PidGroup::AnyGroupChild
        } else {
            PidGroup::AnyChild
        }
    }
}

/// Waits for and returns events that are received from the given supplied process or process group
/// ID, and associated options.
///
/// # Usage Notes
///
/// - If the value of the PID is `PidGroup::ProcessID(Pid)`, it will wait on the child
///   with that has the specified process ID.
/// - If the value of the PID is `PidGroup::ProcessGroupID(Pid)`, it will wait on any child that
///   belongs to the specified process group ID.
/// - If the value of the PID is `PidGroup::AnyGroupChild`, it will wait on any child process
///   that has the same group ID as the current process.
/// - If the value of the PID is `PidGroup::AnyChild`, it will wait on any child process of the
///   current process.
///
/// # Possible Error Values
///
/// If this function returns an error, the error value will be one of the following:
///
/// - **ECHILD**: The process does not exist or is not a child of the current process.
///   - This may also happen if a child process has the `SIGCHLD` signal masked or set to
///     `SIG_IGN`.
/// - **EINTR**: `WNOHANG` was not set and either an unblocked signal or a `SIGCHLD` was caught.
/// - **EINVAL**: The supplied options were invalid.
pub fn waitpid<O>(pid: PidGroup, options: O) -> Result<WaitStatus>
    where O: Into<Option<WaitPidFlag>>
{
    use self::WaitStatus::*;

    let mut status = 0;
    let options = options.into().map_or(0, |o| o.bits());

    let pid = match pid {
        PidGroup::ProcessID(pid) if pid < Pid::from_raw(-1)      => -pid_t::from(pid),
        PidGroup::ProcessGroupID(pid) if pid > Pid::from_raw(0) => -pid_t::from(pid),
        PidGroup::ProcessID(pid) | PidGroup::ProcessGroupID(pid) => pid_t::from(pid),
        PidGroup::AnyGroupChild => 0,
        PidGroup::AnyChild => -1,
    };

    let res = unsafe { libc::waitpid(pid.into(), &mut status as *mut c_int, options) };

    Errno::result(res).map(|res| match res {
        0 => StillAlive,
        res => decode(Pid::from_raw(res), status),
    })
}

/// Waits on any child of the current process, returning on events that change the status of
/// of that child. It is directly equivalent to `waitpid(PidGroup::AnyChild, None)`.
pub fn wait() -> Result<WaitStatus> {
    waitpid(PidGroup::AnyChild, None)
}
