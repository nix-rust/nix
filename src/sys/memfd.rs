use libc;
use std::os::unix::io::RawFd;
use {Error, Result};
use std::ffi::CStr;

bitflags!(
    flags MemFdCreateFlag: libc::c_uint {
        const MFD_CLOEXEC       = 0x0001,
        const MFD_ALLOW_SEALING = 0x0002,
    }
);

pub fn memfd_create(name: &CStr, flags: MemFdCreateFlag) -> Result<RawFd> {
    use sys::syscall::{syscall, MEMFD_CREATE};
    let res = unsafe { syscall(MEMFD_CREATE, name.as_ptr(), flags.bits()) };
    if res == -1 { Err(Error::last()) }
    else { Ok(res as RawFd) }
}
