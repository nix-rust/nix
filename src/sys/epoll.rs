use {Errno, Result};
use libc::{self, c_int};
use std::os::unix::io::RawFd;

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
        const EPOLLEXCLUSIVE = 1 << 28,
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

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EpollEvent {
    event: libc::epoll_event,
}

impl EpollEvent {
    fn new(events: EpollEventKind, data: u64) -> EpollEvent {
        EpollEvent { event: libc::epoll_event { events: events.bits(), u64: data } }
    }
}

#[inline]
pub fn epoll_create() -> Result<RawFd> {
    let res = unsafe { libc::epoll_create(1024) };

    Errno::result(res)
}

#[inline]
pub fn epoll_create1(flags: c_int) -> Result<RawFd> {
    let res = unsafe { libc::epoll_create1(flags) };

    Errno::result(res)
}

#[inline]
pub fn epoll_ctl(epfd: RawFd, op: EpollOp, fd: RawFd, event: &mut EpollEvent) -> Result<()> {
    let res = unsafe { libc::epoll_ctl(epfd, op as c_int, fd, &mut event.event) };

    Errno::result(res).map(drop)
}

#[inline]
pub fn epoll_wait(epfd: RawFd, events: &mut [EpollEvent], timeout_ms: isize) -> Result<usize> {
    let res = unsafe {
        libc::epoll_wait(epfd, events.as_mut_ptr() as *mut libc::epoll_event, events.len() as c_int, timeout_ms as c_int)
    };

    Errno::result(res).map(|r| r as usize)
}
