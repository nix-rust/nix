use {Errno, Result};
use libc::{self, c_int};
use std::os::unix::io::RawFd;
use std::ptr;
use std::mem;
use ::Error;

bitflags!(
    #[repr(C)]
    pub flags EpollFlags: u32 {
        const EPOLLIN = libc::EPOLLIN as u32,
        const EPOLLPRI = libc::EPOLLPRI as u32,
        const EPOLLOUT = libc::EPOLLOUT as u32,
        const EPOLLRDNORM = libc::EPOLLRDNORM as u32,
        const EPOLLRDBAND = libc::EPOLLRDBAND as u32,
        const EPOLLWRNORM = libc::EPOLLWRNORM as u32,
        const EPOLLWRBAND = libc::EPOLLWRBAND as u32,
        const EPOLLMSG = libc::EPOLLMSG as u32,
        const EPOLLERR = libc::EPOLLERR as u32,
        const EPOLLHUP = libc::EPOLLHUP as u32,
        const EPOLLRDHUP = libc::EPOLLRDHUP as u32,
        const EPOLLEXCLUSIVE = 1 << 28,
        const EPOLLWAKEUP = libc::EPOLLWAKEUP as u32,
        const EPOLLONESHOT = libc::EPOLLONESHOT as u32,
        const EPOLLET = libc::EPOLLET as u32,
    }
);

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub enum EpollOp {
    EpollCtlAdd = 1,
    EpollCtlDel = 2,
    EpollCtlMod = 3
}

libc_bitflags!{
    pub flags EpollCreateFlags: c_int {
        EPOLL_CLOEXEC,
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EpollEvent {
    event: libc::epoll_event,
}

impl EpollEvent {
    pub fn new(events: EpollFlags, data: u64) -> Self {
        EpollEvent { event: libc::epoll_event { events: events.bits(), u64: data } }
    }

    pub fn empty() -> Self {
        unsafe { mem::zeroed::<EpollEvent>() }
    }

    pub fn events(&self) -> EpollFlags {
        EpollFlags::from_bits(self.event.events).unwrap()
    }

    pub fn data(&self) -> u64 {
        self.event.u64
    }
}

impl<'a> Into<&'a mut EpollEvent> for Option<&'a mut EpollEvent> {
    #[inline]
    fn into(self) -> &'a mut EpollEvent {
        match self {
            Some(epoll_event) => epoll_event,
            None => unsafe { &mut *ptr::null_mut::<EpollEvent>() }
        }
    }
}

#[inline]
pub fn epoll_create() -> Result<RawFd> {
    let res = unsafe { libc::epoll_create(1024) };

    Errno::result(res)
}

#[inline]
pub fn epoll_create1(flags: EpollCreateFlags) -> Result<RawFd> {
    let res = unsafe { libc::epoll_create1(flags.bits()) };

    Errno::result(res)
}

#[inline]
pub fn epoll_ctl<'a, T>(epfd: RawFd, op: EpollOp, fd: RawFd, event: T) -> Result<()>
    where T: Into<&'a mut EpollEvent>
{
    let event: &mut EpollEvent = event.into();
    if event as *const EpollEvent == ptr::null() && op != EpollOp::EpollCtlDel {
        Err(Error::Sys(Errno::EINVAL))
    } else {
        let res = unsafe { libc::epoll_ctl(epfd, op as c_int, fd, &mut event.event) };
        Errno::result(res).map(drop)
    }
}

#[inline]
pub fn epoll_wait(epfd: RawFd, events: &mut [EpollEvent], timeout_ms: isize) -> Result<usize> {
    let res = unsafe {
        libc::epoll_wait(epfd, events.as_mut_ptr() as *mut libc::epoll_event, events.len() as c_int, timeout_ms as c_int)
    };

    Errno::result(res).map(|r| r as usize)
}
