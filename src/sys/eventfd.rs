use libc;
use std::os::unix::io::RawFd;
use {Errno, Result};

libc_bitflags! {
    flags EfdFlags: libc::c_int {
        const EFD_CLOEXEC, // Since Linux 2.6.27
        const EFD_NONBLOCK, // Since Linux 2.6.27
        const EFD_SEMAPHORE, // Since Linux 2.6.30
    }
}

pub fn eventfd(initval: libc::c_uint, flags: EfdFlags) -> Result<RawFd> {
    let res = unsafe { libc::eventfd(initval, flags.bits()) };

    Errno::result(res).map(|r| r as RawFd)
}
