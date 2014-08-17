#![cfg(target_os = "linux")]

use std::path::Path;
use std::io::FilePermission;
use libc::c_int;
use errno::{SysResult, SysError, from_ffi};

pub type Fd = c_int;

bitflags!(
    flags OFlag: c_int {
        static O_ACCMODE   = 0o00000003,
        static O_RDONLY    = 0o00000000,
        static O_WRONLY    = 0o00000001,
        static O_RDWR      = 0o00000002,
        static O_CREAT     = 0o00000100,
        static O_EXCL      = 0o00000200,
        static O_NOCTTY    = 0o00000400,
        static O_TRUNC     = 0o00001000,
        static O_APPEND    = 0o00002000,
        static O_NONBLOCK  = 0o00004000,
        static O_DSYNC     = 0o00010000,
        static O_DIRECT    = 0o00040000,
        static O_LARGEFILE = 0o00100000,
        static O_DIRECTORY = 0o00200000,
        static O_NOFOLLOW  = 0o00400000,
        static O_NOATIME   = 0o01000000,
        static O_CLOEXEC   = 0o02000000,
        static O_SYNC      = 0o04000000,
        static O_PATH      = 0o10000000,
        static O_TMPFILE   = 0o20000000,
        static O_NDELAY    = O_NONBLOCK.bits
    }
)

mod ffi {
    pub use libc::{open, close};
}

pub fn open(path: &Path, oflag: OFlag, mode: FilePermission) -> SysResult<Fd> {
    let fd = unsafe { ffi::open(path.to_c_str().as_ptr(), oflag.bits, mode.bits()) };

    if fd < 0 {
        return Err(SysError::last());
    }

    Ok(fd)
}

pub fn close(fd: Fd) -> SysResult<()> {
    let res = unsafe { ffi::close(fd) };
    from_ffi(res)
}
