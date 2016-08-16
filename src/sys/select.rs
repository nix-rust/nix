use std::ptr::{null, null_mut};
use std::os::unix::io::RawFd;
use std::mem;
use libc;
use libc::{fd_set, c_int, timespec, timeval, sigset_t};
use {Errno, Result};
use sys::time::{TimeVal, TimeSpec};
use sys::signal::SigSet;

pub struct FdSet {
    set: fd_set
}

impl AsRef<fd_set> for FdSet {
    fn as_ref(&self) -> &fd_set {
        &self.set
    }
}

pub const FD_SETSIZE: RawFd = libc::FD_SETSIZE as RawFd;

impl FdSet {
    pub fn new() -> FdSet {
        let mut set = FdSet {
            set: unsafe { mem::uninitialized() }
        };
        set.clear();
        set
    }

    pub fn insert(&mut self, fd: RawFd) {
        assert!(fd >= 0 && fd < FD_SETSIZE, "RawFd out of bounds");
        unsafe {
            libc::FD_SET(fd, &mut self.set as *mut _);
        }
    }

    pub fn remove(&mut self, fd: RawFd) {
        assert!(fd >= 0 && fd < FD_SETSIZE, "RawFd out of bounds");
        unsafe {
            libc::FD_CLR(fd, &mut self.set as *mut _);
        }
    }

    pub fn contains(&self, fd: RawFd) -> bool {
        assert!(fd >= 0 && fd < FD_SETSIZE, "RawFd out of bounds");
        unsafe {
            // We require `transmute` here because FD_ISSET wants a mutable pointer,
            // when in fact it doesn't mutate.
            libc::FD_ISSET(fd, mem::transmute(&self.set as *const fd_set))
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            libc::FD_ZERO(&mut self.set as *mut _);
        }
    }
}

pub fn select(nfds: c_int,
              readfds: Option<&mut FdSet>,
              writefds: Option<&mut FdSet>,
              errorfds: Option<&mut FdSet>,
              timeout: Option<&mut TimeVal>) -> Result<c_int> {
    let readfds = readfds.map(|set| &mut set.set as *mut fd_set).unwrap_or(null_mut());
    let writefds = writefds.map(|set| &mut set.set as *mut fd_set).unwrap_or(null_mut());
    let errorfds = errorfds.map(|set| &mut set.set as *mut fd_set).unwrap_or(null_mut());
    let timeout = timeout.map(|tv| tv.as_mut() as *mut timeval).unwrap_or(null_mut());

    let res = unsafe {
        libc::select(nfds, readfds, writefds, errorfds, timeout)
    };

    Errno::result(res)
}

pub fn pselect(nfds: c_int,
               readfds: Option<&mut FdSet>,
               writefds: Option<&mut FdSet>,
               errorfds: Option<&mut FdSet>,
               timeout: Option<&TimeSpec>,
               sigmask: Option<&SigSet>) -> Result<c_int> {
    let readfds = readfds.map(|set| &mut set.set as *mut fd_set).unwrap_or(null_mut());
    let writefds = writefds.map(|set| &mut set.set as *mut fd_set).unwrap_or(null_mut());
    let errorfds = errorfds.map(|set| &mut set.set as *mut fd_set).unwrap_or(null_mut());
    let timeout = timeout.map(|ts| ts.as_ref() as *const timespec).unwrap_or(null());
    let sigmask = sigmask.map(|sm| sm.as_ref() as *const sigset_t).unwrap_or(null());

    let res = unsafe {
        libc::pselect(nfds, readfds, writefds, errorfds, timeout, sigmask)
    };

    Errno::result(res)
}
