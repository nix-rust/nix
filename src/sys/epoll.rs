use libc::c_int;
use fcntl::Fd;
use errno::{SysResult, SysError, from_ffi};

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
    flags EpollEventKind: u32 {
        static EPOLLIN = 0x001,
        static EPOLLPRI = 0x002,
        static EPOLLOUT = 0x004,
        static EPOLLRDNORM = 0x040,
        static EPOLLRDBAND = 0x080,
        static EPOLLWRNORM = 0x100,
        static EPOLLWRBAND = 0x200,
        static EPOLLMSG = 0x400,
        static EPOLLERR = 0x008,
        static EPOLLHUP = 0x010,
        static EPOLLRDHUP = 0x2000,
        static EPOLLWAKEUP = 1 << 29,
        static EPOLLONESHOT = 1 << 30,
        static EPOLLET = 1 << 31
    }
)

#[repr(C)]
pub enum EpollOp {
    EpollCtlAdd = 1,
    EpollCtlDel = 2,
    EpollCtlMod = 3
}

pub struct EpollEvent {
    pub events: EpollEventKind,
    pub data: u64
}

#[inline]
pub fn epoll_create() -> SysResult<Fd> {
    let res = unsafe { ffi::epoll_create(1024) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
}

#[inline]
pub fn epoll_ctl(epfd: Fd, op: EpollOp, fd: Fd, event: &EpollEvent) -> SysResult<()> {
    let res = unsafe { ffi::epoll_ctl(epfd, op as c_int, fd, event as *const EpollEvent) };
    from_ffi(res)
}

#[inline]
pub fn epoll_wait(epfd: Fd, events: &mut [EpollEvent], timeout_ms: uint) -> SysResult<uint> {
    let res = unsafe {
        ffi::epoll_wait(epfd, events.as_mut_ptr(), events.len() as c_int, timeout_ms as c_int)
    };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res as uint)
}
