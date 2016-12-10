use {Errno, Result};
use libc::{self, c_int};
use std::os::unix::io::RawFd;
use std::ptr;

bitflags!(
    #[repr(C)]
    flags EpollFlags: u32 {
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

libc_bitflags!{
    flags EpollCreateFlags: c_int {
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

    pub empty() -> Self {
        EpollEvent::new(EpollFlags::from_bits(0), 0);
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
    let res = unsafe { libc::epoll_ctl(epfd, op as c_int, fd, &mut event.into().event) };
    Errno::result(res).map(drop)
}

#[inline]
pub fn epoll_wait(epfd: RawFd, events: &mut [EpollEvent], timeout_ms: isize) -> Result<usize> {
    let res = unsafe {
        libc::epoll_wait(epfd, events.as_mut_ptr() as *mut libc::epoll_event, events.len() as c_int, timeout_ms as c_int)
    };

    Errno::result(res).map(|r| r as usize)
}
