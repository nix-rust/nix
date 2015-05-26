//! Linux epoll binding for Rust.
//!
//! Example:
//!
//!     extern crate nix;
//!     use nix::sys::epoll;
//!     use nix::sys::epoll::EpollOp;
//!     use nix::sys::socket;
//!     use std::ascii::AsciiExt;
//!     use std::net;
//!
//!     fn main() {
//!         let addrs = vec!(
//!             socket::InetAddr::from_std(&net::SocketAddr::V4(
//!                     net::SocketAddrV4::new(net::Ipv4Addr::new(173,194,122,196), 80))),
//!             socket::InetAddr::from_std(&net::SocketAddr::V4(
//!                     net::SocketAddrV4::new(net::Ipv4Addr::new(173,194,122,195), 80))),
//!         );
//!         let epollfd = match epoll::epoll_create() {
//!             Ok(fd) => fd,
//!             Err(e) => {
//!                 println!("Errro calling epoll_create(): {:?}", e);
//!                 return;
//!             }
//!         };
//!         for i in 0..addrs.len() {
//!             let sock = match socket::socket(socket::AddressFamily::Inet,
//!                                            socket::SockType::Stream,
//!                                            socket::SOCK_NONBLOCK) {
//!                 Ok(fd) => fd,
//!                 Err(e) => {
//!                     println!("Error calling socket. {:?}", e);
//!                     continue;
//!                 }
//!             };
//!             match socket::connect(sock, &socket::SockAddr::Inet(addrs[i])) {
//!                 Ok(()) => (),
//!                 Err(e) if e == nix::Error::Sys(nix::errno::Errno::EINPROGRESS) => (),
//!                 Err(e) => {
//!                     println!("Error connecting: {:?}", e);
//!                     continue;
//!                 }
//!             }
//!             let event = epoll::EpollEvent {
//!                 events: epoll::EPOLLOUT
//!                       | epoll::EPOLLERR
//!                       | epoll::EPOLLHUP,
//!                 data: sock as u64
//!             };
//!             match epoll::epoll_ctl(epollfd, EpollOp::EpollCtlAdd, sock, &event) {
//!                 Ok(()) => (),
//!                 Err(e) => {
//!                     println!("Error calling epoll_ctl: {:?}", e);
//!                     continue;
//!                 }
//!             }
//!         }
//!         loop {
//!             let mut events = [epoll::EpollEvent {events: epoll::EPOLLERR, data: 0}; 2];
//!             let nfds = match epoll::epoll_wait(epollfd, &mut events, 1000) {
//!                 Ok(n) if n == 0 => {
//!                     println!("Timeout.");
//!                     continue;
//!                 }
//!                 Ok(n) => n,
//!                 Err(e) => {
//!                     println!("Error calling epoll_wait: {:?}", e);
//!                     break;
//!                 }
//!             };
//!             for i in 0..nfds {
//!                 let fd = events[i].data as nix::fcntl::Fd;
//!                 if (events[i].events & epoll::EPOLLERR) == epoll::EPOLLERR
//!                 || (events[i].events & epoll::EPOLLHUP) == epoll::EPOLLHUP {
//!                     match epoll::epoll_ctl(epollfd, EpollOp::EpollCtlDel, fd, &events[i]) {
//!                         Ok(()) => (),
//!                         Err(e) => {
//!                             println!("Error calling epoll_ctl: {:?}", e);
//!                             continue;
//!                         }
//!                     }
//!                     println!("Error on socket.");
//!                 }
//!                 if (events[i].events & epoll::EPOLLOUT) == epoll::EPOLLOUT{
//!                     let data =
//!                         b"GET / HTTP/1.1\r\n\
//!                         Host: http://google.com\r\n\
//!                         User-Agent: Mozilla/5.0 (Windows NT 6.1)\r\n\r\n";
//!                     match nix::unistd::write(events[i].data as nix::fcntl::Fd, data) {
//!                         Ok(n) if n == data.len() => {
//!                             println!("Request sent.");
//!                             let event = epoll::EpollEvent {
//!                                 events: epoll::EPOLLIN
//!                                       | epoll::EPOLLPRI
//!                                       | epoll::EPOLLERR
//!                                       | epoll::EPOLLHUP,
//!                                 data: fd as u64
//!                             };
//!                             match epoll::epoll_ctl(epollfd, EpollOp::EpollCtlMod, fd, &event) {
//!                                 Ok(()) => (),
//!                                 Err(e) => {
//!                                     println!("Error calling epoll_ctl: {:?}", e);
//!                                     continue;
//!                                 }
//!                             }
//!                             continue;
//!                         }
//!                         _ => {
//!                             println!("Error calling write.");
//!                             match epoll::epoll_ctl(epollfd, EpollOp::EpollCtlDel, fd, &events[i]){
//!                                 Ok(()) => (),
//!                                 Err(e) => {
//!                                     println!("Error calling epoll_ctl: {:?}", e);
//!                                     continue;
//!                                 }
//!                             }
//!                             continue;
//!                         }
//!                     }
//!                 }
//!                 if (events[i].events & epoll::EPOLLIN) == epoll::EPOLLIN {
//!                     let mut data: [u8; 2048] = [0; 2048];
//!                     match nix::unistd::read(fd, &mut data) {
//!                         Ok(n) => {
//!                             let mut out: String = String::new();
//!                             for i in 0..n {
//!                                 if data[i].is_ascii() {
//!                                     out.push(data[i] as char);
//!                                 }
//!                             }
//!                             println!("Response: {}", out);
//!                         },
//!                         Err(e) => {
//!                             println!("Error calling read: {:?}", e);
//!                             continue;
//!                         }
//!                     }
//!                     match epoll::epoll_ctl(epollfd, EpollOp::EpollCtlDel, fd, &events[i]) {
//!                         Ok(()) => (),
//!                         Err(e) => {
//!                             println!("Error calling epoll_ctl: {:?}", e);
//!                             continue;
//!                         }
//!                     }
//!                 }
//!             }
//!         }
//!     }



use std::fmt;
use libc::c_int;
use errno::Errno;
use {Error, Result, from_ffi};
use fcntl::Fd;

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

impl fmt::Debug for EpollEventKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let variants = [
            (EPOLLIN,       "EPOLLIN"),
            (EPOLLPRI,      "EPOLLPRI"),
            (EPOLLOUT,      "EPOLLOUT"),
            (EPOLLRDNORM,   "EPOLLRDNORM"),
            (EPOLLRDBAND,   "EPOLLRDBAND"),
            (EPOLLWRNORM,   "EPOLLWRNORM"),
            (EPOLLWRBAND,   "EPOLLWRBAND"),
            (EPOLLMSG,      "EPOLLMSG"),
            (EPOLLERR,      "EPOLLERR"),
            (EPOLLHUP,      "EPOLLHUP"),
            (EPOLLRDHUP,    "EPOLLRDHUP"),
            (EPOLLWAKEUP,   "EPOLLWAKEUP"),
            (EPOLLONESHOT,  "EPOLLONESHOT"),
            (EPOLLET,       "EPOLLET")];

        let mut first = true;

        for &(val, name) in variants.iter() {
            if self.contains(val) {
                if first {
                    first = false;
                    try!(write!(fmt, "{}", name));
                } else {
                    try!(write!(fmt, "|{}", name));
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub enum EpollOp {
    EpollCtlAdd = 1,
    EpollCtlDel = 2,
    EpollCtlMod = 3
}

#[cfg(all(target_os = "android", not(target_arch = "x86_64")))]
#[derive(Copy)]
#[repr(C)]
pub struct EpollEvent {
    pub events: EpollEventKind,
    pub data: u64
}

#[cfg(all(target_os = "android", not(target_arch = "x86_64")))]
#[test]
fn test_epoll_event_size() {
    use std::mem::size_of;
    assert_eq!(size_of::<EpollEvent>(), 16);
}

#[cfg(any(not(target_os = "android"), target_arch = "x86_64"))]
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct EpollEvent {
    pub events: EpollEventKind,
    pub data: u64
}

#[inline]
pub fn epoll_create() -> Result<Fd> {
    let res = unsafe { ffi::epoll_create(1024) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(res)
}

#[inline]
pub fn epoll_ctl(epfd: Fd, op: EpollOp, fd: Fd, event: &EpollEvent) -> Result<()> {
    let res = unsafe { ffi::epoll_ctl(epfd, op as c_int, fd, event as *const EpollEvent) };
    from_ffi(res)
}

#[inline]
pub fn epoll_wait(epfd: Fd, events: &mut [EpollEvent], timeout_ms: usize) -> Result<usize> {
    let res = unsafe {
        ffi::epoll_wait(epfd, events.as_mut_ptr(), events.len() as c_int, timeout_ms as c_int)
    };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(res as usize)
}
