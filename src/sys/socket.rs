#![cfg(target_os = "linux")]

use std::{mem, ptr};
use libc::{c_int, sockaddr, sockaddr_in, sockaddr_in6, sockaddr_un, socklen_t};
use fcntl::Fd;
use errno::{SysResult, SysError, from_ffi};

mod ffi {
    use libc::{c_int, sockaddr, socklen_t};

    extern {
        pub fn socket(domain: c_int, ty: c_int, proto: c_int) -> c_int;

        pub fn listen(sockfd: c_int, backlog: c_int) -> c_int;

        pub fn bind(sockfd: c_int, addr: *const sockaddr, addrlen: socklen_t) -> c_int;

        pub fn accept(
            sockfd: c_int,
            addr: *const sockaddr,
            addrlen: *mut socklen_t) -> c_int;

        pub fn accept4(
            sockfd: c_int,
            addr: *const sockaddr,
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

pub enum BindAddr<'a> {
    BindIpV4(&'a sockaddr_in),
    BindIpV6(&'a sockaddr_in6),
    BindUnix(&'a sockaddr_un)
}

pub fn bind(sockfd: Fd, addr: BindAddr) -> SysResult<()> {
    let res = unsafe {
        match addr {
            BindIpV4(addr) => ffi::bind(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in>() as socklen_t),
            BindIpV6(addr) => ffi::bind(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in6>() as socklen_t),
            BindUnix(addr) => ffi::bind(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_un>() as socklen_t)
        }
    };

    from_ffi(res)
}

pub fn accept(sockfd: Fd) -> SysResult<Fd> {
    let res = unsafe { ffi::accept(sockfd, ptr::null(), ptr::mut_null()) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
}

pub fn accept4(sockfd: Fd, flags: SockFlag) -> SysResult<Fd> {
    // TODO: Check the kernel version
    let res = unsafe { ffi::accept4(sockfd, ptr::null(), ptr::mut_null(), flags.bits) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
}
