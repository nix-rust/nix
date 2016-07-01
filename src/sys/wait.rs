use libc::{self, pid_t, c_int};
use {Errno, Result};

use sys::signal::Signal;

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
        const WNOHANG     = libc::WNOHANG,
        const WUNTRACED   = libc::WUNTRACED,
    }
);

#[cfg(any(target_os = "linux",
          target_os = "android"))]
bitflags!(
    flags WaitPidFlag: c_int {
        const WNOHANG     = libc::WNOHANG,
        const WUNTRACED   = libc::WUNTRACED,
        const WEXITED     = libc::WEXITED,
        const WCONTINUED  = libc::WCONTINUED,
        const WNOWAIT     = libc::WNOWAIT, // Don't reap, just poll status.
        const __WNOTHREAD = libc::__WNOTHREAD, // Don't wait on children of other threads in this group
        const __WALL      = libc::__WALL, // Wait on all children, regardless of type
        const __WCLONE    = libc::__WCLONE,
    }
);

#[cfg(any(target_os = "linux",
          target_os = "android"))]
const WSTOPPED: WaitPidFlag = WUNTRACED;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum WaitStatus {
    Exited(pid_t, i8),
    Signaled(pid_t, Signal, bool),
    Stopped(pid_t, Signal),
    Continued(pid_t),
    StillAlive
}

#[cfg(any(target_os = "linux",
          target_os = "android"))]
mod status {
    use sys::signal::Signal;

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
