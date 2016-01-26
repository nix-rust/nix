// Silence invalid warnings due to rust-lang/rust#16719
#![allow(improper_ctypes)]

use {Errno, Result};
use libc::{c_int, c_void, size_t, off_t};
use std::marker::PhantomData;
use std::os::unix::io::RawFd;

mod ffi {
    use super::IoVec;
    use libc::{ssize_t, c_int, size_t, off_t, c_void};
    use std::os::unix::io::RawFd;

    extern {
        // vectorized version of write
        // doc: http://man7.org/linux/man-pages/man2/writev.2.html
        pub fn writev(fd: RawFd, iov: *const IoVec<&[u8]>, iovcnt: c_int) -> ssize_t;

        // vectorized version of read
        // doc: http://man7.org/linux/man-pages/man2/readv.2.html
        pub fn readv(fd: RawFd, iov: *const IoVec<&mut [u8]>, iovcnt: c_int) -> ssize_t;

        // vectorized write at a specified offset
        // doc: http://man7.org/linux/man-pages/man2/pwritev.2.html
        #[cfg(feature = "preadv_pwritev")]
        pub fn pwritev(fd: RawFd, iov: *const IoVec<&[u8]>, iovcnt: c_int,
                       offset: off_t) -> ssize_t;

        // vectorized read at a specified offset
        // doc: http://man7.org/linux/man-pages/man2/preadv.2.html
        #[cfg(feature = "preadv_pwritev")]
        pub fn preadv(fd: RawFd, iov: *const IoVec<&mut [u8]>, iovcnt: c_int,
                      offset: off_t) -> ssize_t;

        // write to a file at a specified offset
        // doc: http://man7.org/linux/man-pages/man2/pwrite.2.html
        pub fn pwrite(fd: RawFd, buf: *const c_void, nbyte: size_t,
                      offset: off_t) -> ssize_t;

        // read from a file at a specified offset
        // doc: http://man7.org/linux/man-pages/man2/pread.2.html
        pub fn pread(fd: RawFd, buf: *mut c_void, nbyte: size_t, offset: off_t)
                     -> ssize_t;
    }
}

pub fn writev(fd: RawFd, iov: &[IoVec<&[u8]>]) -> Result<usize> {
    let res = unsafe { ffi::writev(fd, iov.as_ptr(), iov.len() as c_int) };

    Errno::result(res).map(|r| r as usize)
}

pub fn readv(fd: RawFd, iov: &mut [IoVec<&mut [u8]>]) -> Result<usize> {
    let res = unsafe { ffi::readv(fd, iov.as_ptr(), iov.len() as c_int) };

    Errno::result(res).map(|r| r as usize)
}

#[cfg(feature = "preadv_pwritev")]
pub fn pwritev(fd: RawFd, iov: &[IoVec<&[u8]>],
               offset: off_t) -> Result<usize> {
    let res = unsafe {
        ffi::pwritev(fd, iov.as_ptr(), iov.len() as c_int, offset)
    };

    Errno::result(res).map(|r| r as usize)
}

#[cfg(feature = "preadv_pwritev")]
pub fn preadv(fd: RawFd, iov: &mut [IoVec<&mut [u8]>],
              offset: off_t) -> Result<usize> {
    let res = unsafe {
        ffi::preadv(fd, iov.as_ptr(), iov.len() as c_int, offset)
    };

    Errno::result(res).map(|r| r as usize)
}

pub fn pwrite(fd: RawFd, buf: &[u8], offset: off_t) -> Result<usize> {
    let res = unsafe {
        ffi::pwrite(fd, buf.as_ptr() as *const c_void, buf.len() as size_t,
                    offset)
    };

    Errno::result(res).map(|r| r as usize)
}

pub fn pread(fd: RawFd, buf: &mut [u8], offset: off_t) -> Result<usize>{
    let res = unsafe {
        ffi::pread(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t,
                   offset)
    };

    Errno::result(res).map(|r| r as usize)
}

#[repr(C)]
pub struct IoVec<T> {
    iov_base: *mut c_void,
    iov_len: size_t,
    phantom: PhantomData<T>
}

impl<T> IoVec<T> {
    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [u8] {
        use std::slice;

        unsafe {
            slice::from_raw_parts(
                self.iov_base as *const u8,
                self.iov_len as usize)
        }
    }
}

impl<'a> IoVec<&'a [u8]> {
    pub fn from_slice(buf: &'a [u8]) -> IoVec<&'a [u8]> {
        IoVec {
            iov_base: buf.as_ptr() as *mut c_void,
            iov_len: buf.len() as size_t,
            phantom: PhantomData
        }
    }
}

impl<'a> IoVec<&'a mut [u8]> {
    pub fn from_mut_slice(buf: &'a mut [u8]) -> IoVec<&'a mut [u8]> {
        IoVec {
            iov_base: buf.as_ptr() as *mut c_void,
            iov_len: buf.len() as size_t,
            phantom: PhantomData
        }
    }
}

#[test]
pub fn test_size_of_io_vec() {
    use nixtest;
    nixtest::assert_size_of::<IoVec<&[u8]>>("iovec");
}
