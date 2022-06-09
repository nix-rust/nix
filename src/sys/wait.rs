//! This module contains the [`wait`] and [`waitpid`] functions.
//!
//! These are used to wait on and obtain status information from child
//! processes, which provide more granular control over child management than
//! the primitives provided by Rust's standard library, and are critical in the
//! creation of shells and managing jobs on *nix platforms.
//!
//! # Manual Pages
//!
//! - [`waitpid(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/waitpid.html)
//! - [`wait(2)`](http://pubs.opengroup.org/onlinepubs/007908799/xsh/wait.html)
//! - [`waitid(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/waitid.html)
//!
//! # Examples
//!
//! The following is an adaptation of the example program in the
//! [waitpid(2)](https://pubs.opengroup.org/onlinepubs/9699919799/functions/waitpid.html)
//! documentation.
//!
//! ```no_run
//! use nix::unistd::{fork,ForkResult};
//! use nix::unistd::Pid;
//!
//! let cpid = unsafe { fork() }.unwrap();
//! match cpid {
//!     ForkResult::Child => child_process(),
//!     ForkResult::Parent{ child } => parent_process(child),
//! }
//!
//! fn child_process() {
//!     use nix::unistd::{getpid,pause};
//!     use std::{write,io::{stdout,Write}};
//!
//!     let cpid = getpid();
//!     writeln!(stdout(), "Child PID is {}. Send it signals. E.g.:", cpid);
//!     writeln!(stdout(), "    kill -s SIGSTOP {}", cpid);
//!     nix::unistd::pause();
//!     unsafe { libc::_exit(0) };
//! }
//!
//! fn parent_process(cpid: Pid) {
//!     use nix::sys::wait::{WaitPidFlag,WaitStatus,waitpid};
//!     loop {
//!         match waitpid(cpid, WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED) {
//!             Err(err) => panic!("waitpid failed: {}", err.desc()),
//!             Ok(status) => {
//!                 match status {
//!                     WaitStatus::Exited(pid, code) => {
//!                         println!("pid {}, exited, status={}", pid, code);
//!                         break;
//!                     },
//!
//!                     WaitStatus::Signaled(pid, signal, core) => {
//!                         let comment = if core { " (core dumped)"} else { "" };
//!                         println!("pid {} killed by signal {}{}", pid, signal, comment);
//!                         break;
//!                     },
//!
//!                     WaitStatus::Stopped(pid, signal) =>
//!                         println!("pid {} stopped by signal {}", pid, signal),
//!
//!                     WaitStatus::Continued(pid) =>
//!                         println!("pid {} continued", pid),
//!
//!                     WaitStatus::StillAlive =>
//!                         println!("child pid {} is still alive", cpid),
//!
//!                     // Additional statuses are platform-specific
//!                     _ => panic!("Unexpected WaitStatus {:?}", status),
//!                 }
//!             },
//!         }
//!     }
//! }
//! ```
use crate::errno::Errno;
use crate::sys::signal::Signal;
use crate::unistd::Pid;
use crate::Result;
use cfg_if::cfg_if;
use libc::{self, c_int};
use std::convert::TryFrom;
#[cfg(any(
    target_os = "android",
    all(target_os = "linux", not(target_env = "uclibc")),
))]
use std::os::unix::io::RawFd;

libc_bitflags!(
    /// Controls the behavior of [`waitpid`].
    pub struct WaitPidFlag: c_int {
        /// Do not block if the status is not immediately available for one
        /// of the child processes specified by pid.
        WNOHANG;
        /// Report the status of selected processes which are stopped due to a
        /// [`SIGTTIN`](crate::sys::signal::Signal::SIGTTIN),
        /// [`SIGTTOU`](crate::sys::signal::Signal::SIGTTOU),
        /// [`SIGTSTP`](crate::sys::signal::Signal::SIGTSTP), or
        /// [`SIGSTOP`](crate::sys::signal::Signal::SIGSTOP) signal.
        WUNTRACED;
        /// Report the status of selected processes which have terminated.
        #[cfg(any(target_os = "android",
                  target_os = "freebsd",
                  target_os = "haiku",
                  target_os = "ios",
                  target_os = "linux",
                  target_os = "redox",
                  target_os = "macos",
                  target_os = "netbsd"))]
        #[cfg_attr(docsrs, doc(cfg(all())))]
        WEXITED;
        /// Report the status of selected processes that have continued from a
        /// job control stop by receiving a
        /// [`SIGCONT`](crate::sys::signal::Signal::SIGCONT) signal.
        WCONTINUED;
        /// An alias for [`WUNTRACED`](Self::WUNTRACED).
        #[cfg(any(target_os = "android",
                  target_os = "freebsd",
                  target_os = "haiku",
                  target_os = "ios",
                  target_os = "linux",
                  target_os = "redox",
                  target_os = "macos",
                  target_os = "netbsd"))]
        #[cfg_attr(docsrs, doc(cfg(all())))]
        WSTOPPED;
        /// Don't wait, just poll status. Leaves the child in a waitable
        /// state; a later wait call can be used to again retrieve the
        /// child status information.
        #[cfg(any(target_os = "android",
                  target_os = "freebsd",
                  target_os = "haiku",
                  target_os = "ios",
                  target_os = "linux",
                  target_os = "redox",
                  target_os = "macos",
                  target_os = "netbsd"))]
        #[cfg_attr(docsrs, doc(cfg(all())))]
        WNOWAIT;
        /// Don't wait on children of other threads in this group
        #[cfg(any(target_os = "android", target_os = "linux", target_os = "redox"))]
        #[cfg_attr(docsrs, doc(cfg(all())))]
        __WNOTHREAD;
        /// Wait for all children, regardless of type (clone or non-clone)
        #[cfg(any(target_os = "android", target_os = "linux", target_os = "redox"))]
        #[cfg_attr(docsrs, doc(cfg(all())))]
        __WALL;
        /// Wait for "clone" children only. If omitted then wait for
        /// "non-clone" children only. (A "clone" child is one which delivers
        /// no signal, or a signal other than [`SIGCHLD`] to its parent upon
        /// termination.) This option is ignored if [`__WALL`] is also
        /// specified.
        ///
        /// [`SIGCHLD`]: crate::sys::signal::Signal::SIGCHLD
        /// [`__WALL`]: Self::__WALL
        #[cfg(any(target_os = "android", target_os = "linux", target_os = "redox"))]
        #[cfg_attr(docsrs, doc(cfg(all())))]
        __WCLONE;
    }
);

/// Possible return values from [`wait`] or [`waitpid`].
///
/// Each status (other than [`StillAlive`](WaitStatus::StillAlive))
/// describes a state transition in a child process
/// [`Pid`](crate::unistd::Pid), such as the process exiting or stopping,
/// plus additional data about the transition if any.
///
/// Note that there are two Linux-specific enum variants,
/// [`PtraceEvent`](WaitStatus) and [`PtraceSyscall`](WaitStatus).
/// Portable code should avoid exhaustively matching on [`WaitStatus`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WaitStatus {
    /// The process exited normally (as with `exit()` or returning from
    /// `main`) with the given exit code. This case matches the C macro
    /// [`WIFEXITED(status)`]; the second field is [`WEXITSTATUS(status)`].
    ///
    /// [`WIFEXITED(status)`]: libc::WIFEXITED
    /// [`WEXITSTATUS(status)`]: libc::WEXITSTATUS
    Exited(Pid, i32),
    /// The process was killed by the given signal. The third field
    /// indicates whether the signal generated a core dump. This case
    /// matches the C macro [`WIFSIGNALED(status)`]; the last two
    /// fields correspond to [`WTERMSIG(status)`] and
    /// [`WCOREDUMP(status)`].
    ///
    /// [`WIFSIGNALED(status)`]: libc::WIFSIGNALED
    /// [`WTERMSIG(status)`]: libc::WTERMSIG
    /// [`WCOREDUMP(status)`]: libc::WCOREDUMP
    Signaled(Pid, Signal, bool),
    /// The process is alive, but was stopped by the given signal. This
    /// is only reported if [`WUNTRACED`] was passed. This case matches
    /// the C macro [`WIFSTOPPED(status)`]; the second field is
    /// [`WSTOPSIG(status)`].
    ///
    /// [`WUNTRACED`]: self::WaitPidFlag::WUNTRACED
    /// [`WIFSTOPPED(status)`]: libc::WIFSTOPPED
    /// [`WSTOPSIG(status)`]: libc::WSTOPSIG
    Stopped(Pid, Signal),
    /// The traced process was stopped by a [`ptrace`] event. All
    /// currently defined events use [`SIGTRAP`] as the signal; the third
    /// field is the [`Event`] value.
    ///
    /// See [`ptrace(2)`] for more information.
    ///
    /// [`ptrace`]: crate::sys::ptrace
    /// [`Event`]: crate::sys::ptrace::Event
    /// [`SIGTRAP`]: crate::sys::signal::Signal::SIGTRAP
    /// [`ptrace(2)`]: https://man7.org/linux/man-pages/man2/ptrace.2.html
    #[cfg(any(target_os = "linux", target_os = "android"))]
    #[cfg_attr(docsrs, doc(cfg(all())))]
    PtraceEvent(Pid, Signal, c_int),
    /// The traced process was stopped by execution of a system call,
    /// and [`PTRACE_O_TRACESYSGOOD`] is in effect.
    ///
    /// See [`ptrace(2)`] for more information.
    ///
    /// [`PTRACE_O_TRACESYSGOOD`]: crate::sys::ptrace::Options::PTRACE_O_TRACESYSGOOD
    /// [`ptrace(2)`]: https://man7.org/linux/man-pages/man2/ptrace.2.html
    #[cfg(any(target_os = "linux", target_os = "android"))]
    #[cfg_attr(docsrs, doc(cfg(all())))]
    PtraceSyscall(Pid),
    /// The process was previously stopped but has resumed execution
    /// after receiving a [`SIGCONT`] signal. This is only reported if
    /// [`WCONTINUED`] was passed. This case matches the C
    /// macro [`WIFCONTINUED(status)`].
    ///
    /// [`SIGCONT`]: crate::sys::signal::Signal::SIGCONT
    /// [`WCONTINUED`]: self::WaitPidFlag::WCONTINUED
    /// [`WIFCONTINUED(status)`]: libc::WIFCONTINUED
    Continued(Pid),
    /// There are currently no state changes to report in any awaited
    /// child process. This is only returned if [`WNOHANG`]
    /// was used (otherwise [`wait`] or [`waitpid`] would block until
    /// there was something to report).
    ///
    /// [`WNOHANG`]: self::WaitPidFlag::WNOHANG
    StillAlive,
}

impl WaitStatus {
    /// Extracts the PID from the [`WaitStatus`] unless it's
    /// [`StillAlive`](self::WaitStatus::StillAlive).
    pub fn pid(&self) -> Option<Pid> {
        use self::WaitStatus::*;
        match *self {
            Exited(p, _) | Signaled(p, _, _) | Stopped(p, _) | Continued(p) => Some(p),
            StillAlive => None,
            #[cfg(any(target_os = "android", target_os = "linux"))]
            PtraceEvent(p, _, _) | PtraceSyscall(p) => Some(p),
        }
    }
}

fn exited(status: i32) -> bool {
    libc::WIFEXITED(status)
}

fn exit_status(status: i32) -> i32 {
    libc::WEXITSTATUS(status)
}

fn signaled(status: i32) -> bool {
    libc::WIFSIGNALED(status)
}

fn term_signal(status: i32) -> Result<Signal> {
    Signal::try_from(libc::WTERMSIG(status))
}

fn dumped_core(status: i32) -> bool {
    libc::WCOREDUMP(status)
}

fn stopped(status: i32) -> bool {
    libc::WIFSTOPPED(status)
}

fn stop_signal(status: i32) -> Result<Signal> {
    Signal::try_from(libc::WSTOPSIG(status))
}

#[cfg(any(target_os = "android", target_os = "linux"))]
fn syscall_stop(status: i32) -> bool {
    // From ptrace(2), setting PTRACE_O_TRACESYSGOOD has the effect
    // of delivering SIGTRAP | 0x80 as the signal number for syscall
    // stops. This allows easily distinguishing syscall stops from
    // genuine SIGTRAP signals.
    libc::WSTOPSIG(status) == libc::SIGTRAP | 0x80
}

#[cfg(any(target_os = "android", target_os = "linux"))]
fn stop_additional(status: i32) -> c_int {
    (status >> 16) as c_int
}

fn continued(status: i32) -> bool {
    libc::WIFCONTINUED(status)
}

impl WaitStatus {
    /// Convert a `wstatus` obtained from [`libc::waitpid`] into a
    /// [`WaitStatus`]:
    ///
    /// # Errors
    ///
    /// - [`EINVAL`](crate::errno::Errno::EINVAL): The supplied options were
    ///   invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use nix::sys::wait::WaitStatus;
    /// use nix::sys::signal::Signal;
    /// let pid = nix::unistd::Pid::from_raw(1);
    /// let status = WaitStatus::from_raw(pid, 0x0002);
    /// assert_eq!(status, Ok(WaitStatus::Signaled(pid, Signal::SIGINT, false)));
    /// ```
    pub fn from_raw(pid: Pid, status: i32) -> Result<WaitStatus> {
        Ok(if exited(status) {
            WaitStatus::Exited(pid, exit_status(status))
        } else if signaled(status) {
            WaitStatus::Signaled(pid, term_signal(status)?, dumped_core(status))
        } else if stopped(status) {
            cfg_if! {
                if #[cfg(any(target_os = "android", target_os = "linux"))] {
                    fn decode_stopped(pid: Pid, status: i32) -> Result<WaitStatus> {
                        let status_additional = stop_additional(status);
                        Ok(if syscall_stop(status) {
                            WaitStatus::PtraceSyscall(pid)
                        } else if status_additional == 0 {
                            WaitStatus::Stopped(pid, stop_signal(status)?)
                        } else {
                            WaitStatus::PtraceEvent(pid, stop_signal(status)?,
                                                    stop_additional(status))
                        })
                    }
                } else {
                    fn decode_stopped(pid: Pid, status: i32) -> Result<WaitStatus> {
                        Ok(WaitStatus::Stopped(pid, stop_signal(status)?))
                    }
                }
            }
            return decode_stopped(pid, status);
        } else {
            assert!(continued(status));
            WaitStatus::Continued(pid)
        })
    }

    /// Convert a [`siginfo_t`] value obtained from [`libc::waitid`] to a
    /// [`WaitStatus`]
    ///
    /// # Errors
    ///
    /// - [`EINVAL`](crate::errno::Errno::EINVAL): The supplied options were
    ///   invalid.
    ///
    /// # Safety
    ///
    /// Because [`siginfo_t`] is implemented as a union, not all fields may
    /// be initialized. The functions [`si_pid()`] and [`si_status()`] must be
    /// valid to call on the passed reference.
    ///
    /// [`siginfo_t`]: libc::siginfo_t
    /// [`si_pid()`]: libc::unix::linux_like::linux::gnu::siginfo_t
    /// [`si_status()`]: libc::unix::linux_like::linux::gnu::siginfo_t
    /// [`libc::waitid`]: libc::unix::linux_like::waitid
    #[cfg(any(
        target_os = "android",
        target_os = "freebsd",
        target_os = "haiku",
        all(target_os = "linux", not(target_env = "uclibc")),
    ))]
    unsafe fn from_siginfo(siginfo: &libc::siginfo_t) -> Result<WaitStatus> {
        let si_pid = siginfo.si_pid();
        if si_pid == 0 {
            return Ok(WaitStatus::StillAlive);
        }

        assert_eq!(siginfo.si_signo, libc::SIGCHLD);

        let pid = Pid::from_raw(si_pid);
        let si_status = siginfo.si_status();

        let status = match siginfo.si_code {
            libc::CLD_EXITED => WaitStatus::Exited(pid, si_status),
            libc::CLD_KILLED | libc::CLD_DUMPED => WaitStatus::Signaled(
                pid,
                Signal::try_from(si_status)?,
                siginfo.si_code == libc::CLD_DUMPED,
            ),
            libc::CLD_STOPPED => WaitStatus::Stopped(pid, Signal::try_from(si_status)?),
            libc::CLD_CONTINUED => WaitStatus::Continued(pid),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            libc::CLD_TRAPPED => {
                if si_status == libc::SIGTRAP | 0x80 {
                    WaitStatus::PtraceSyscall(pid)
                } else {
                    WaitStatus::PtraceEvent(
                        pid,
                        Signal::try_from(si_status & 0xff)?,
                        (si_status >> 8) as c_int,
                    )
                }
            }
            _ => return Err(Errno::EINVAL),
        };

        Ok(status)
    }
}

/// Waits for and events from one or more child processes.
///
/// # Usage Notes
///
/// The value of `pid` changes the behavior of [`waitpid`]
///
/// - To wait on a specific child PID, pass it as an argument directly.
/// - To wait on any child process, pass [`None`] or
///   [`ANY_CHILD`](crate::unistd::ANY_CHILD).
/// - To wait on any child within a specific process group ID, pass
///   `some_pid.`[`as_wait_pgrp()`](crate::unistd::Pid::as_wait_pgrp).
/// - To wait on any child whose process group matches the current process pass
///   [`ANY_PGRP_CHILD`](crate::unistd::ANY_PGRP_CHILD).
///
/// # Errors
///
/// - [`ECHILD`](crate::errno::Errno::ECHILD): The process does not exist or is
///   not a child of the current process.This may also happen if a child
///   process has the [`SIGCHLD`](crate::sys::signal::Signal::SIGCHLD) signal
///   masked or set to [`SigIgn`](crate::sys::signal::SigHandler::SigIgn).
/// - [`EINTR`](crate::errno::Errno::EINTR):
///   [`WNOHANG`](self::WaitPidFlag::WNOHANG) was not set and either
///   an unblocked signal or a [`SIGCHLD`](crate::sys::signal::Signal::SIGCHLD)
///   was caught.
/// - [`EINVAL`](crate::errno::Errno::EINVAL): The supplied options were
///   invalid.
///
/// # See also
///
/// [`waitpid(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/waitpid.html)
pub fn waitpid<P, O>(pid: P, options: O) -> Result<WaitStatus>
    where
    P: Into<Option<Pid>>,
    O: Into<Option<WaitPidFlag>>
{
    use self::WaitStatus::*;
    let mut status = 0;

    let option_bits = match options.into() {
        Some(bits) => bits.bits(),
        None => 0,
    };

    let res = unsafe {
        libc::waitpid(
            pid.into().unwrap_or_else(|| Pid::from_raw(-1)).into(),
            &mut status as *mut c_int,
            option_bits,
        )
    };

    match Errno::result(res)? {
        0 => Ok(StillAlive),
        res => WaitStatus::from_raw(Pid::from_raw(res), status),
    }
}

/// Waits for and returns events from any child of the current process.
///
/// While waiting on the child, this function will return on events that
/// indicate that the status of that child has changed. It is directly
/// equivalent to [`waitpid(None, None)`](self::waitpid).
///
/// # Errors
///
/// - [`ECHILD`](crate::errno::Errno::ECHILD): The process does not exist or is
///    not a child of the current process.This may also happen if a child
///    process has the [`SIGCHLD`](crate::sys::signal::Signal::SIGCHLD) signal
///    masked or set to [`SigIgn`](crate::sys::signal::SigHandler::SigIgn).
///
/// # See also
///
/// [wait(2)](https://pubs.opengroup.org/onlinepubs/9699919799/functions/wait.html)
pub fn wait() -> Result<WaitStatus> {
    waitpid(None, None)
}

/// The ID argument for [`waitid`]
#[cfg(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "haiku",
    all(target_os = "linux", not(target_env = "uclibc")),
))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Id {
    /// Wait for any child
    All,
    /// Wait for the child whose process ID matches the given PID
    Pid(Pid),
    /// Wait for the child whose process group ID matches the given PID
    ///
    /// If the PID is zero, the caller's process group is used since Linux 5.4.
    PGid(Pid),
    /// Wait for the child referred to by the given PID file descriptor
    #[cfg(any(target_os = "android", target_os = "linux"))]
    PIDFd(RawFd),
}

/// Wait for a process to change status
///
/// This is the glibc and POSIX interface for [`waitpid`]which has
/// different semantics for expressing process group IDs.
///
/// # See also
///
/// [waitid(2)](https://pubs.opengroup.org/onlinepubs/9699919799/functions/waitid.html)
#[cfg(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "haiku",
    all(target_os = "linux", not(target_env = "uclibc")),
))]
pub fn waitid(id: Id, flags: WaitPidFlag) -> Result<WaitStatus> {
    let (idtype, idval) = match id {
        Id::All => (libc::P_ALL, 0),
        Id::Pid(pid) => (libc::P_PID, pid.as_raw() as libc::id_t),
        Id::PGid(pid) => (libc::P_PGID, pid.as_raw() as libc::id_t),
        #[cfg(any(target_os = "android", target_os = "linux"))]
        Id::PIDFd(fd) => (libc::P_PIDFD, fd as libc::id_t),
    };

    let siginfo = unsafe {
        // Memory is zeroed rather than uninitialized, as not all platforms
        // initialize the memory in the StillAlive case
        let mut siginfo: libc::siginfo_t = std::mem::zeroed();
        Errno::result(libc::waitid(idtype, idval, &mut siginfo, flags.bits()))?;
        siginfo
    };

    unsafe { WaitStatus::from_siginfo(&siginfo) }
}
