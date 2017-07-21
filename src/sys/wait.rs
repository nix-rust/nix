use libc::{self, c_int};
use {Errno, Result};
use unistd::Pid;

use sys::signal::Signal;

mod ffi {
    use libc::{pid_t, c_int};

    extern {
        pub fn waitpid(pid: pid_t, status: *mut c_int, options: c_int) -> pid_t;
    }
}

#[cfg(not(any(target_os = "linux",
              target_os = "android")))]
libc_bitflags!(
    pub flags WaitPidFlag: c_int {
        WNOHANG,
        WUNTRACED,
    }
);

#[cfg(any(target_os = "linux",
          target_os = "android"))]
libc_bitflags!(
    pub flags WaitPidFlag: c_int {
        WNOHANG,
        WUNTRACED,
        WEXITED,
        WCONTINUED,
        WNOWAIT, // Don't reap, just poll status.
        __WNOTHREAD, // Don't wait on children of other threads in this group
        __WALL, // Wait on all children, regardless of type
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

#[cfg(any(target_os = "linux",
          target_os = "android"))]
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

#[cfg(any(target_os = "macos",
          target_os = "ios"))]
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
                        WaitStatus::PtraceEvent(pid, status::stop_signal(status), status::stop_additional(status))
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

pub fn waitpid<P: Into<Option<Pid>>>(pid: P, options: Option<WaitPidFlag>) -> Result<WaitStatus> {
    use self::WaitStatus::*;

    let mut status: i32 = 0;

    let option_bits = match options {
        Some(bits) => bits.bits(),
        None => 0
    };

    let res = unsafe { ffi::waitpid(pid.into().unwrap_or(Pid::from_raw(-1)).into(), &mut status as *mut c_int, option_bits) };

    Ok(match try!(Errno::result(res)) {
        0 => StillAlive,
        res => decode(Pid::from_raw(res), status),
    })
}

pub fn wait() -> Result<WaitStatus> {
    waitpid(None, None)
}
