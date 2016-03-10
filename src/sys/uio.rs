// Silence invalid warnings due to rust-lang/rust#16719
#![allow(improper_ctypes)]

use {Errno, Result};
use libc::{self, c_int, c_void, size_t, off_t};
use std::marker::PhantomData;
use std::os::unix::io::RawFd;

pub fn writev(fd: RawFd, iov: &[IoVec<&[u8]>]) -> Result<usize> {
    let res = unsafe { libc::writev(fd, iov.as_ptr() as *const libc::iovec, iov.len() as c_int) };

    Errno::result(res).map(|r| r as usize)
}

pub fn readv(fd: RawFd, iov: &mut [IoVec<&mut [u8]>]) -> Result<usize> {
    let res = unsafe { libc::readv(fd, iov.as_ptr() as *const libc::iovec, iov.len() as c_int) };

    Errno::result(res).map(|r| r as usize)
}

#[cfg(feature = "preadv_pwritev")]
pub fn pwritev(fd: RawFd, iov: &[IoVec<&[u8]>],
               offset: off_t) -> Result<usize> {
    let res = unsafe {
        libc::pwritev(fd, iov.as_ptr() as *const libc::iovec, iov.len() as c_int, offset)
    };

    Errno::result(res).map(|r| r as usize)
}

#[cfg(feature = "preadv_pwritev")]
pub fn preadv(fd: RawFd, iov: &mut [IoVec<&mut [u8]>],
              offset: off_t) -> Result<usize> {
    let res = unsafe {
        libc::preadv(fd, iov.as_ptr() as *const libc::iovec, iov.len() as c_int, offset)
    };

    Errno::result(res).map(|r| r as usize)
}

pub fn pwrite(fd: RawFd, buf: &[u8], offset: off_t) -> Result<usize> {
    let res = unsafe {
        libc::pwrite(fd, buf.as_ptr() as *const c_void, buf.len() as size_t,
                    offset)
    };

    Errno::result(res).map(|r| r as usize)
}

pub fn pread(fd: RawFd, buf: &mut [u8], offset: off_t) -> Result<usize>{
    let res = unsafe {
        libc::pread(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t,
                   offset)
    };

    Errno::result(res).map(|r| r as usize)
}

#[repr(C)]
pub struct IoVec<T>(libc::iovec, PhantomData<T>); 

impl<T> IoVec<T> {
    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [u8] {
        use std::slice;

        unsafe {
            slice::from_raw_parts(
                self.0.iov_base as *const u8,
                self.0.iov_len as usize)
        }
    }
}

impl<'a> IoVec<&'a [u8]> {
    pub fn from_slice(buf: &'a [u8]) -> IoVec<&'a [u8]> {
        IoVec(libc::iovec {
            iov_base: buf.as_ptr() as *mut c_void,
            iov_len: buf.len() as size_t,
        }, PhantomData)
    }
}

impl<'a> IoVec<&'a mut [u8]> {
    pub fn from_mut_slice(buf: &'a mut [u8]) -> IoVec<&'a mut [u8]> {
        IoVec(libc::iovec {
            iov_base: buf.as_ptr() as *mut c_void,
            iov_len: buf.len() as size_t,
        }, PhantomData)
    }
}

#[test]
pub fn test_size_of_io_vec() {
    use nixtest;
    nixtest::assert_size_of::<IoVec<&[u8]>>("iovec");
}
