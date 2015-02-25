// Silence invalid warnings due to rust-lang/rust#16719
#![allow(improper_ctypes)]

use {NixResult, NixError};
use errno::Errno;
use fcntl::Fd;
use libc::{c_int, c_void, size_t};
use std::marker::PhantomData;

mod ffi {
    use super::IoVec;
    use libc::{ssize_t, c_int};
    use fcntl::Fd;

    extern {
        // vectorized version of write
        // doc: http://man7.org/linux/man-pages/man2/writev.2.html
        pub fn writev(fd: Fd, iov: *const IoVec<&[u8]>, iovcnt: c_int) -> ssize_t;

        // vectorized version of read
        // doc: http://man7.org/linux/man-pages/man2/readv.2.html
        pub fn readv(fd: Fd, iov: *const IoVec<&mut [u8]>, iovcnt: c_int) -> ssize_t;
    }
}

pub fn writev(fd: Fd, iov: &[IoVec<&[u8]>]) -> NixResult<usize> {
    let res = unsafe { ffi::writev(fd, iov.as_ptr(), iov.len() as c_int) };

    if res < 0 {
        return Err(NixError::Sys(Errno::last()));
    }

    return Ok(res as usize)
}

pub fn readv(fd: Fd, iov: &mut [IoVec<&mut [u8]>]) -> NixResult<usize> {
    let res = unsafe { ffi::readv(fd, iov.as_ptr(), iov.len() as c_int) };
    if res < 0 {
        return Err(NixError::Sys(Errno::last()));
    }

    return Ok(res as usize)
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
