use libc::{pid_t, c_int};
use errno::Errno;
use {Error, Result};

mod ffi {
    use libc::{pid_t, c_int};

    extern {
        pub fn waitpid(pid: pid_t, status: *mut c_int, options: c_int) -> pid_t;
    }
}

bitflags!(
    flags WaitPidFlag: c_int {
        const WNOHANG = 0x00000001,
    }
);

#[derive(Copy)]
pub enum WaitStatus {
    Exited(pid_t),
    StillAlive
}

pub fn waitpid(pid: pid_t, options: Option<WaitPidFlag>) -> Result<WaitStatus> {
    use self::WaitStatus::*;

    let mut status: i32 = 0;

    let option_bits = match options {
        Some(bits) => bits.bits(),
        None => 0
    };

    let res = unsafe { ffi::waitpid(pid as pid_t, &mut status as *mut c_int, option_bits) };

    if res < 0 {
        Err(Error::Sys(Errno::last()))
    } else if res == 0 {
        Ok(StillAlive)
    } else {
        Ok(Exited(res))
    }
}
