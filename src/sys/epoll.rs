use {Errno, Result};
use libc::c_int;
use std::os::unix::io::RawFd;

mod ffi {
    use libc::{c_int};
    use super::EpollEvent;

    extern {
        pub fn epoll_create(size: c_int) -> c_int;
        pub fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *const EpollEvent) -> c_int;
        pub fn epoll_wait(epfd: c_int, events: *mut EpollEvent, max_events: c_int, timeout: c_int) -> c_int;
    }
}

bitflags!(
    #[repr(C)]
    flags EpollEventKind: u32 {
        const EPOLLIN = 0x001,
        const EPOLLPRI = 0x002,
        const EPOLLOUT = 0x004,
        const EPOLLRDNORM = 0x040,
        const EPOLLRDBAND = 0x080,
        const EPOLLWRNORM = 0x100,
        const EPOLLWRBAND = 0x200,
        const EPOLLMSG = 0x400,
        const EPOLLERR = 0x008,
        const EPOLLHUP = 0x010,
        const EPOLLRDHUP = 0x2000,
        const EPOLLWAKEUP = 1 << 29,
        const EPOLLONESHOT = 1 << 30,
        const EPOLLET = 1 << 31
    }
);

#[derive(Clone, Copy)]
#[repr(C)]
pub enum EpollOp {
    EpollCtlAdd = 1,
    EpollCtlDel = 2,
    EpollCtlMod = 3
}

#[cfg(not(target_arch = "x86_64"))]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct EpollEvent {
    pub events: EpollEventKind,
    pub data: u64
}

#[cfg(target_arch = "x86_64")]
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct EpollEvent {
    pub events: EpollEventKind,
    pub data: u64
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[test]
fn test_epoll_event_size() {
    use std::mem::size_of;
    assert_eq!(size_of::<EpollEvent>(), 12);
}

#[cfg(target_arch = "arm")]
#[test]
fn test_epoll_event_size() {
    use std::mem::size_of;
    assert_eq!(size_of::<EpollEvent>(), 16);
}

#[inline]
pub fn epoll_create() -> Result<RawFd> {
    let res = unsafe { ffi::epoll_create(1024) };

    Errno::result(res)
}

#[inline]
pub fn epoll_ctl(epfd: RawFd, op: EpollOp, fd: RawFd, event: &EpollEvent) -> Result<()> {
    let res = unsafe { ffi::epoll_ctl(epfd, op as c_int, fd, event as *const EpollEvent) };

    Errno::result(res).map(drop)
}

#[inline]
pub fn epoll_wait(epfd: RawFd, events: &mut [EpollEvent], timeout_ms: isize) -> Result<usize> {
    let res = unsafe {
        ffi::epoll_wait(epfd, events.as_mut_ptr(), events.len() as c_int, timeout_ms as c_int)
    };

    Errno::result(res).map(|r| r as usize)
}
