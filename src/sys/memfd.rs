use libc;
use std::os::unix::io::RawFd;
use {Errno, Result};
use std::ffi::CStr;

bitflags!(
    pub struct MemFdCreateFlag: libc::c_uint {
        const MFD_CLOEXEC       = 0x0001;
        const MFD_ALLOW_SEALING = 0x0002;
    }
);

pub fn memfd_create(name: &CStr, flags: MemFdCreateFlag) -> Result<RawFd> {
    let res = unsafe {
        libc::syscall(libc::SYS_memfd_create, name.as_ptr(), flags.bits())
    };

    Errno::result(res).map(|r| r as RawFd)
}
