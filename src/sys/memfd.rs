use errno::Errno;
use libc;
use std::ffi::CStr;
use std::os::unix::io::RawFd;
use Result;

libc_bitflags!(
    pub struct MemFdCreateFlag: libc::c_uint {
        MFD_CLOEXEC;
        MFD_ALLOW_SEALING;
    }
);

pub fn memfd_create(name: &CStr, flags: MemFdCreateFlag) -> Result<RawFd> {
    let res = unsafe { libc::syscall(libc::SYS_memfd_create, name.as_ptr(), flags.bits()) };

    Errno::result(res).map(|r| r as RawFd)
}
