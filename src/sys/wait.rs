use libc::{pid_t, c_int};
use {Errno, Result};

use sys::signal;

mod ffi {
    use libc::{pid_t, c_int};

    extern {
        pub fn waitpid(pid: pid_t, status: *mut c_int, options: c_int) -> pid_t;
    }
}

#[cfg(not(any(target_os = "linux",
              target_os = "android")))]
bitflags!(
    flags WaitPidFlag: c_int {
        const WNOHANG     = 0x00000001,
    }
);

#[cfg(any(target_os = "linux",
          target_os = "android"))]
bitflags!(
    flags WaitPidFlag: c_int {
        const WNOHANG     = 0x00000001,
        const WUNTRACED   = 0x00000002,
        const WEXITED     = 0x00000004,
        const WCONTINUED  = 0x00000008,
        const WNOWAIT     = 0x01000000, // Don't reap, just poll status.
        const __WNOTHREAD = 0x20000000, // Don't wait on children of other threads in this group
        const __WALL      = 0x40000000, // Wait on all children, regardless of type
        // const __WCLONE    = 0x80000000,
    }
);

#[cfg(any(target_os = "linux",
          target_os = "android"))]
const WSTOPPED: WaitPidFlag = WUNTRACED;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum WaitStatus {
    Exited(pid_t, i8),
    Signaled(pid_t, signal::SigNum, bool),
    Stopped(pid_t, signal::SigNum),
    Continued(pid_t),
    StillAlive
}

#[cfg(any(target_os = "linux",
          target_os = "android"))]
mod status {
    use sys::signal;

    pub fn exited(status: i32) -> bool {
        (status & 0x7F) == 0
    }

    pub fn exit_status(status: i32) -> i8 {
        ((status & 0xFF00) >> 8) as i8
    }

    pub fn signaled(status: i32) -> bool {
        ((((status & 0x7f) + 1) as i8) >> 1) > 0
    }

    pub fn term_signal(status: i32) -> signal::SigNum {
        (status & 0x7f) as signal::SigNum
    }

    pub fn dumped_core(status: i32) -> bool {
        (status & 0x80) != 0
    }

    pub fn stopped(status: i32) -> bool {
        (status & 0xff) == 0x7f
    }

    pub fn stop_signal(status: i32) -> signal::SigNum {
        ((status & 0xFF00) >> 8) as signal::SigNum
    }

    pub fn continued(status: i32) -> bool {
        status == 0xFFFF
    }
}

#[cfg(any(target_os = "macos",
          target_os = "ios"))]
mod status {
    use sys::signal;

    const WCOREFLAG: i32 = 0x80;
    const WSTOPPED: i32 = 0x7f;

    fn wstatus(status: i32) -> i32 {
        status & 0x7F
    }

    pub fn exit_status(status: i32) -> i8 {
        ((status >> 8) & 0xFF) as i8
    }

    pub fn stop_signal(status: i32) -> signal::SigNum {
        (status >> 8) as signal::SigNum
    }

    pub fn continued(status: i32) -> bool {
        wstatus(status) == WSTOPPED && stop_signal(status) == 0x13
    }

    pub fn stopped(status: i32) -> bool {
        wstatus(status) == WSTOPPED && stop_signal(status) != 0x13
    }

    pub fn exited(status: i32) -> bool {
        wstatus(status) == 0
    }

    pub fn signaled(status: i32) -> bool {
        wstatus(status) != WSTOPPED && wstatus(status) != 0
    }

    pub fn term_signal(status: i32) -> signal::SigNum {
        wstatus(status) as signal::SigNum
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
    use sys::signal;

    const WCOREFLAG: i32 = 0x80;
    const WSTOPPED: i32 = 0x7f;

    fn wstatus(status: i32) -> i32 {
        status & 0x7F
    }

    pub fn stopped(status: i32) -> bool {
        wstatus(status) == WSTOPPED
    }

    pub fn stop_signal(status: i32) -> signal::SigNum {
        (status >> 8) as signal::SigNum
    }

    pub fn signaled(status: i32) -> bool {
        wstatus(status) != WSTOPPED && wstatus(status) != 0 && status != 0x13
    }

    pub fn term_signal(status: i32) -> signal::SigNum {
        wstatus(status) as signal::SigNum
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

fn decode(pid : pid_t, status: i32) -> WaitStatus {
    if status::exited(status) {
        WaitStatus::Exited(pid, status::exit_status(status))
    } else if status::signaled(status) {
        WaitStatus::Signaled(pid, status::term_signal(status), status::dumped_core(status))
    } else if status::stopped(status) {
        WaitStatus::Stopped(pid, status::stop_signal(status))
    } else {
        assert!(status::continued(status));
        WaitStatus::Continued(pid)
    }
}

pub fn waitpid(pid: pid_t, options: Option<WaitPidFlag>) -> Result<WaitStatus> {
    use self::WaitStatus::*;

    let mut status: i32 = 0;

    let option_bits = match options {
        Some(bits) => bits.bits(),
        None => 0
    };

    let res = unsafe { ffi::waitpid(pid as pid_t, &mut status as *mut c_int, option_bits) };

    Ok(match try!(Errno::result(res)) {
        0 => StillAlive,
        res => decode(res, status),
    })
}

pub fn wait() -> Result<WaitStatus> {
    waitpid(-1, None)
}
