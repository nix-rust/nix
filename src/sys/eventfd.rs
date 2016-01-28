use libc;
use std::os::unix::io::RawFd;
use {Errno, Result};

bitflags!(
    flags EventFdFlag: libc::c_int {
        const EFD_CLOEXEC   = 0o2000000, // Since Linux 2.6.27
        const EFD_NONBLOCK  = 0o0004000, // Since Linux 2.6.27
        const EFD_SEMAPHORE = 0o0000001, // Since Linux 2.6.30
    }
);

mod ffi {
    use libc;

    extern {
        pub fn eventfd(initval: libc::c_uint, flags: libc::c_int) -> libc::c_int;
    }
}

pub fn eventfd(initval: usize, flags: EventFdFlag) -> Result<RawFd> {
    unsafe {
        let res = ffi::eventfd(initval as libc::c_uint, flags.bits());

        Errno::result(res).map(|r| r as RawFd)
    }
}
