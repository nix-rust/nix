#![cfg(target_os = "linux")]

use std::{mem, ptr};
use libc::{c_int, sockaddr, socklen_t};
use fcntl::Fd;
use errno::{SysResult, SysError, from_ffi};

pub use libc::{in_addr, sockaddr_in, sockaddr_in6, sockaddr_un, sa_family_t};

mod ffi {
    use libc::{c_int, sockaddr, socklen_t};
    pub use libc::{socket, listen, bind, accept, connect};

    extern {
        pub fn accept4(
            sockfd: c_int,
            addr: *mut sockaddr,
            addrlen: *mut socklen_t,
            flags: c_int) -> c_int;
    }
}

pub type AddressFamily = c_int;

pub static AF_UNIX: AddressFamily  = 1;
pub static AF_LOCAL: AddressFamily = AF_UNIX;
pub static AF_INET: AddressFamily  = 2;
pub static AF_INET6: AddressFamily = 10;

pub type SockType = c_int;

pub static SOCK_STREAM: SockType = 1;
pub static SOCK_DGRAM: SockType = 1;
pub static SOCK_SEQPACKET: SockType = 1;
pub static SOCK_RAW: SockType = 1;
pub static SOCK_RDM: SockType = 1;

// Extra flags - Linux 2.6.27
bitflags!(
    flags SockFlag: c_int {
        static SOCK_NONBLOCK = 0o0004000,
        static SOCK_CLOEXEC  = 0o2000000
    }
)

pub fn socket(domain: AddressFamily, ty: SockType, flags: SockFlag) -> SysResult<Fd> {
    // TODO: Check the kernel version
    let res = unsafe { ffi::socket(domain, ty | flags.bits(), 0) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
}

pub fn listen(sockfd: Fd, backlog: uint) -> SysResult<()> {
    let res = unsafe { ffi::listen(sockfd, backlog as c_int) };
    from_ffi(res)
}

pub enum SockAddr {
    SockIpV4(sockaddr_in),
    SockIpV6(sockaddr_in6),
    SockUnix(sockaddr_un)
}

pub fn bind(sockfd: Fd, addr: &SockAddr) -> SysResult<()> {
    let res = unsafe {
        match *addr {
            SockIpV4(ref addr) => ffi::bind(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in>() as socklen_t),
            SockIpV6(ref addr) => ffi::bind(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in6>() as socklen_t),
            SockUnix(ref addr) => ffi::bind(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_un>() as socklen_t)
        }
    };

    from_ffi(res)
}

pub fn accept(sockfd: Fd) -> SysResult<Fd> {
    let res = unsafe { ffi::accept(sockfd, ptr::mut_null(), ptr::mut_null()) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
}

pub fn accept4(sockfd: Fd, flags: SockFlag) -> SysResult<Fd> {
    // TODO: Check the kernel version
    let res = unsafe { ffi::accept4(sockfd, ptr::mut_null(), ptr::mut_null(), flags.bits) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
}

pub fn connect(sockfd: Fd, addr: &SockAddr) -> SysResult<()> {
    let res = unsafe {
        match *addr {
            SockIpV4(ref addr) => ffi::connect(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in>() as socklen_t),
            SockIpV6(ref addr) => ffi::connect(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in6>() as socklen_t),
            SockUnix(ref addr) => ffi::connect(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_un>() as socklen_t)
        }
    };

    from_ffi(res)
}

pub type SockOpt = c_int;

pub static SO_ACCEPTCONN: SockOpt = 30;
pub static SO_BINDTODEVICE: SockOpt = 25;
pub static SO_BROADCAST: SockOpt = 6;
pub static SO_BSDCOMPAT: SockOpt = 14;
pub static SO_DEBUG: SockOpt = 1;
pub static SO_DOMAIN: SockOpt = 39;
pub static SO_ERROR: SockOpt = 4;
pub static SO_DONTROUTE: SockOpt = 5;
pub static SO_KEEPALIVE: SockOpt = 9;
pub static SO_LINGER: SockOpt = 13;
pub static SO_MARK: SockOpt = 36;
pub static SO_OOBINLINE: SockOpt = 10;
pub static SO_PASSCRED: SockOpt = 16;
pub static SO_PEEK_OFF: SockOpt = 42;
pub static SO_PEERCRED: SockOpt = 17;
pub static SO_PRIORITY: SockOpt = 12;
pub static SO_PROTOCOL: SockOpt = 38;
pub static SO_RCVBUF: SockOpt = 8;
pub static SO_RCVBUFFORCE: SockOpt = 33;
pub static SO_RCVLOWAT: SockOpt = 18;
pub static SO_SNDLOWAT: SockOpt = 19;
pub static SO_RCVTIMEO: SockOpt = 20;
pub static SO_SNDTIMEO: SockOpt = 21;
pub static SO_REUSEADDR: SockOpt = 2;
pub static SO_RXQ_OVFL: SockOpt = 40;
pub static SO_SNDBUF: SockOpt = 7;
pub static SO_SNDBUFFORCE: SockOpt = 32;
pub static SO_TIMESTAMP: SockOpt = 29;
pub static SO_TYPE: SockOpt = 3;
pub static SO_BUSY_POLL: SockOpt = 46;
