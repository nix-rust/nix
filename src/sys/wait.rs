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

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum WaitStatus {
    Exited(Pid, i8),
    Signaled(Pid, Signal, bool),
    Stopped(Pid, Signal),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    PtraceEvent(Pid, Signal, c_int),
    Continued(Pid),
    StillAlive
}

#[cfg(any(target_os = "linux",
          target_os = "android"))]
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
                    if status_additional == 0 {
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

/// Counterpart of the POSIX `waitpid` function
/// It's best to use `nix::unistd::Pid` for passing the PID to this function
///
/// # Examples
/// ```
/// use std::process::Command;
/// use nix::unistd::Pid;
/// use nix::sys::wait::{waitpid, WaitStatus};
/// let child = Command::new("ls").spawn().unwrap();
/// let pid = Pid::from_raw(child.id() as i32);
/// match waitpid(pid, None) {
///     Ok(WaitStatus::Exited(_, code)) => println!("Child exited with code {}", code),
///     Ok(_) => println!("Other process exited, but not normally"),
///     Err(_) => panic!("There was an error")
/// }
/// ```
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
