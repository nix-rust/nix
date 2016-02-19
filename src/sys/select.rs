use std::ptr::{null, null_mut};
use std::os::unix::io::RawFd;
use libc::{c_int, timespec};
use {Error, Errno, Result};
use sys::time::TimeVal;
use sys::signal::sigset_t;

pub const FD_SETSIZE: RawFd = 1024;

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[repr(C)]
pub struct FdSet {
    bits: [i32; FD_SETSIZE as usize / 32]
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
const BITS: usize = 32;

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
#[repr(C)]
#[derive(Clone)]
pub struct FdSet {
    bits: [u64; FD_SETSIZE as usize / 64]
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
const BITS: usize = 64;

impl FdSet {
    pub fn new() -> FdSet {
        FdSet {
            bits: [0; FD_SETSIZE as usize / BITS]
        }
    }

    pub fn insert(&mut self, fd: RawFd) {
        let fd = fd as usize;
        self.bits[fd / BITS] |= 1 << (fd % BITS);
    }

    pub fn remove(&mut self, fd: RawFd) {
        let fd = fd as usize;
        self.bits[fd / BITS] &= !(1 << (fd % BITS));
    }

    pub fn contains(&mut self, fd: RawFd) -> bool {
        let fd = fd as usize;
        self.bits[fd / BITS] & (1 << (fd % BITS)) > 0
    }

    pub fn clear(&mut self) {
        for bits in &mut self.bits {
            *bits = 0
        }
    }
}

mod ffi {
    use libc::{c_int, timespec};
    use sys::time::TimeVal;
    use sys::signal::sigset_t;
    use super::FdSet;

    extern {
        pub fn select(nfds: c_int,
                      readfds: *mut FdSet,
                      writefds: *mut FdSet,
                      errorfds: *mut FdSet,
                      timeout: *mut TimeVal) -> c_int;

        pub fn pselect(nfds: c_int,
                       readfds: *mut FdSet,
                       writefds: *mut FdSet,
                       errorfds: *mut FdSet,
                       timeout: *const timespec,
                       sigmask: *const sigset_t) -> c_int;
    }
}

pub fn select(nfds: c_int,
              readfds: Option<&mut FdSet>,
              writefds: Option<&mut FdSet>,
              errorfds: Option<&mut FdSet>,
              timeout: Option<&mut TimeVal>) -> Result<c_int> {
    let readfds = readfds.map(|set| set as *mut FdSet).unwrap_or(null_mut());
    let writefds = writefds.map(|set| set as *mut FdSet).unwrap_or(null_mut());
    let errorfds = errorfds.map(|set| set as *mut FdSet).unwrap_or(null_mut());
    let timeout = timeout.map(|tv| tv as *mut TimeVal).unwrap_or(null_mut());

    let res = unsafe {
        ffi::select(nfds, readfds, writefds, errorfds, timeout)
    };

    Errno::result(res)
}

pub fn pselect(nfds: c_int,
               readfds: Option<&mut FdSet>,
               writefds: Option<&mut FdSet>,
               errorfds: Option<&mut FdSet>,
               timeout: Option<&timespec>,
               sigmask: Option<&sigset_t>) -> Result<c_int> {
    let readfds = readfds.map(|set| set as *mut FdSet).unwrap_or(null_mut());
    let writefds = writefds.map(|set| set as *mut FdSet).unwrap_or(null_mut());
    let errorfds = errorfds.map(|set| set as *mut FdSet).unwrap_or(null_mut());
    let timeout = timeout.map(|ts| ts as *const timespec).unwrap_or(null());
    let sigmask = sigmask.map(|sm| sm as *const sigset_t).unwrap_or(null());

    let res = unsafe {
        ffi::pselect(nfds, readfds, writefds, errorfds, timeout, sigmask)
    };

    Errno::result(res)
}
