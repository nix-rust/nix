use std::os::unix::io::RawFd;
use std::ptr;

use libc::{self, off_t};

use {Errno, Result};

pub fn sendfile(out_fd: RawFd, in_fd: RawFd, offset: Option<&mut off_t>, count: usize) -> Result<usize> {
    let offset = offset.map(|offset| offset as *mut _).unwrap_or(ptr::null_mut());
    let ret = unsafe { libc::sendfile(out_fd, in_fd, offset, count) };
    Errno::result(ret).map(|r| r as usize)
}
