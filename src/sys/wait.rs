//! This module contains the `wait()` and `waitpid()` functions.
//!
//! These are used to wait on and obtain status information from child processes, which provide
//! more granular control over child management than the primitives provided by Rust's standard
//! library, and are critical in the creation of shells and managing jobs on *nix platforms.
//!
//! Manual Page: http://pubs.opengroup.org/onlinepubs/007908799/xsh/wait.html
//!
//! # Examples
//!
//! ```rust
//! use nix::sys::wait::*;
//! use nix::unistd::Pid;
//!
//! let pid = Pid::from_raw(17563);
//! loop {
//!     match waitpid(PidGroup::ProcessGroupID(pid), WUNTRACED) {
//!         Ok(WaitStatus::Exited(pid, status)) => {
//!             println!("Process '{}' exited with status '{}'", pid, status);
//!             break
//!         },
//!         Ok(WaitStatus::Stopped(pid, signal)) => {
//!             println!("Process '{}' stopped with signal '{:?}'", pid, signal);
//!         },
//!         Ok(WaitStatus::Continued(pid)) => {
//!             println!("Process '{}' continued", pid);
//!         },
//!         Ok(_) => (),
//!         Err(why) => {
//!             println!("waitpid returned an error code: {}", why);
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
        #[cfg(any(target_os = "android",
                  target_os = "freebsd",
                  target_os = "linux",
                  target_os = "netbsd"))]
        WEXITED,
        /// Report the status of any continued child process specified by pid whose status has not
        /// been reported since it continued from a job control stop.
        WCONTINUED,
        /// Leave the child in a waitable state; a later wait call can be used to again retrieve
        /// the child status information.
        #[cfg(any(target_os = "android",
                  target_os = "freebsd",
                  target_os = "linux",
                  target_os = "netbsd"))]
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

#[cfg(any(target_os = "linux",
          target_os = "android"))]
const WSTOPPED: WaitPidFlag = WUNTRACED;

/// Possible return values from `wait()` or `waitpid()`.
///
/// Each status (other than `StillAlive`) describes a state transition
/// in a child process `Pid`, such as the process exiting or stopping,
/// plus additional data about the transition if any.
///
/// Note that there are two Linux-specific enum variants, `PtraceEvent`
/// and `PtraceSyscall`. Portable code should avoid exhaustively
/// matching on `WaitStatus`.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum WaitStatus {
    /// The process exited normally (as with `exit()` or returning from
    /// `main`) with the given exit code. This case matches the C macro
    /// `WIFEXITED(status)`; the second field is `WEXITSTATUS(status)`.
    Exited(Pid, i8),
    /// The process was killed by the given signal. The third field
    /// indicates whether the signal generated a core dump. This case
    /// matches the C macro `WIFSIGNALED(status)`; the last two fields
    /// correspond to `WTERMSIG(status)` and `WCOREDUMP(status)`.
    Signaled(Pid, Signal, bool),
    /// The process is alive, but was stopped by the given signal. This
    /// is only reported if `WaitPidFlag::WUNTRACED` was passed. This
    /// case matches the C macro `WIFSTOPPED(status)`; the second field
    /// is `WSTOPSIG(status)`.
    Stopped(Pid, Signal),
    /// The traced process was stopped by a `PTRACE_EVENT_*` event. See
    /// [`nix::sys::ptrace`] and [`ptrace`(2)] for more information. All
    /// currently-defined events use `SIGTRAP` as the signal; the third
    /// field is the `PTRACE_EVENT_*` value of the event.
    ///
    /// [`nix::sys::ptrace`]: ../ptrace/index.html
    /// [`ptrace`(2)]: http://man7.org/linux/man-pages/man2/ptrace.2.html
    #[cfg(any(target_os = "linux", target_os = "android"))]
    PtraceEvent(Pid, Signal, c_int),
    /// The traced process was stopped by execution of a system call,
    /// and `PTRACE_O_TRACESYSGOOD` is in effect. See [`ptrace`(2)] for
    /// more information.
    ///
    /// [`ptrace`(2)]: http://man7.org/linux/man-pages/man2/ptrace.2.html
    #[cfg(any(target_os = "linux", target_os = "android"))]
    PtraceSyscall(Pid),
    /// The process was previously stopped but has resumed execution
    /// after receiving a `SIGCONT` signal. This is only reported if
    /// `WaitPidFlag::WCONTINUED` was passed. This case matches the C
    /// macro `WIFCONTINUED(status)`.
    Continued(Pid),
    /// There are currently no state changes to report in any awaited
    /// child process. This is only returned if `WaitPidFlag::WNOHANG`
    /// was used (otherwise `wait()` or `waitpid()` would block until
    /// there was something to report).
    StillAlive
}

#[cfg(any(target_os = "android", target_os = "linux"))]
mod status {
    use sys::signal::Signal;
    use libc::c_int;
    use libc::SIGTRAP;

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
        // Keep only 7 bits of the signal: the high bit
        // is used to indicate syscall stops, below.
        Signal::from_c_int((status & 0x7F00) >> 8).unwrap()
    }

    pub fn syscall_stop(status: i32) -> bool {
        // From ptrace(2), setting PTRACE_O_TRACESYSGOOD has the effect
        // of delivering SIGTRAP | 0x80 as the signal number for syscall
        // stops. This allows easily distinguishing syscall stops from
        // genuine SIGTRAP signals.
        ((status & 0xFF00) >> 8) == SIGTRAP | 0x80
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
                    let status_additional = status::stop_additional(status);
                    if status::syscall_stop(status) {
                        WaitStatus::PtraceSyscall(pid)
                    } else if status_additional == 0 {
                        WaitStatus::Stopped(pid, status::stop_signal(status))
                    } else {
                        WaitStatus::PtraceEvent(pid, status::stop_signal(status), status)
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

/// Waits for and returns events that are received, with additional options.
///
/// The `pid` value may indicate either a process group ID, or process ID. The options
/// parameter controls the behavior of the function.
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
/// # Errors
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

/// Waits for and returns events from any child of the current process.
///
/// While waiting on the child, this function will return on events that indicate that the status
/// of that child has changed. It is directly equivalent to `waitpid(PidGroup::AnyChild, None)`.
///
/// # Errors
///
/// - **ECHILD**: The process does not exist or is not a child of the current process.
///   - This may also happen if a child process has the `SIGCHLD` signal masked or set to
///     `SIG_IGN`.
pub fn wait() -> Result<WaitStatus> {
    waitpid(PidGroup::AnyChild, None)
}
