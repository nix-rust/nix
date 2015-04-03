//! Socket interface functions
//!
//! [Further reading](http://man7.org/linux/man-pages/man7/socket.7.html)
use {Error, Result, from_ffi};
use errno::Errno;
use features;
use fcntl::{fcntl, FD_CLOEXEC, O_NONBLOCK};
use fcntl::FcntlArg::{F_SETFD, F_SETFL};
use libc::{c_void, c_int, socklen_t, size_t};
use std::{fmt, mem, ptr};
use std::os::unix::prelude::*;

mod addr;
mod consts;
mod ffi;
mod multicast;
pub mod sockopt;

/*
 *
 * ===== Re-exports =====
 *
 */

pub use self::addr::{
    AddressFamily,
    SockAddr,
    InetAddr,
    UnixAddr,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
};
pub use libc::{
    in_addr,
    in6_addr,
    sockaddr,
    sockaddr_in,
    sockaddr_in6,
    sockaddr_un,
    sa_family_t,
};

pub use self::multicast::{
    ip_mreq,
};
pub use self::consts::*;

#[cfg(any(not(target_os = "linux"), not(target_arch = "x86")))]
pub use libc::sockaddr_storage;

// Working around rust-lang/rust#23425
#[cfg(all(target_os = "linux", target_arch = "x86"))]
pub struct sockaddr_storage {
    pub ss_family: sa_family_t,
    pub __ss_align: u32,
    pub __ss_pad2: [u8; 120],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, FromPrimitive)]
#[repr(i32)]
pub enum SockType {
    Stream = consts::SOCK_STREAM,
    Datagram = consts::SOCK_DGRAM,
    SeqPacket = consts::SOCK_SEQPACKET,
    Raw = consts::SOCK_RAW,
    Rdm = consts::SOCK_RDM,
}

// Extra flags - Supported by Linux 2.6.27, normalized on other platforms
bitflags!(
    flags SockFlag: c_int {
        const SOCK_NONBLOCK = 0o0004000,
        const SOCK_CLOEXEC  = 0o2000000
    }
);

/// Create an endpoint for communication
///
/// [Further reading](http://man7.org/linux/man-pages/man2/socket.2.html)
pub fn socket(domain: AddressFamily, ty: SockType, flags: SockFlag) -> Result<RawFd> {
    let mut ty = ty as c_int;
    let feat_atomic = features::socket_atomic_cloexec();

    if feat_atomic {
        ty = ty | flags.bits();
    }

    // TODO: Check the kernel version
    let res = unsafe { ffi::socket(domain as c_int, ty, 0) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    if !feat_atomic {
        if flags.contains(SOCK_CLOEXEC) {
            try!(fcntl(res, F_SETFD(FD_CLOEXEC)));
        }

        if flags.contains(SOCK_NONBLOCK) {
            try!(fcntl(res, F_SETFL(O_NONBLOCK)));
        }
    }

    Ok(res)
}

/// Listen for connections on a socket
///
/// [Further reading](http://man7.org/linux/man-pages/man2/listen.2.html)
pub fn listen(sockfd: RawFd, backlog: usize) -> Result<()> {
    let res = unsafe { ffi::listen(sockfd, backlog as c_int) };
    from_ffi(res)
}

/// Bind a name to a socket
///
/// [Further reading](http://man7.org/linux/man-pages/man2/bind.2.html)
pub fn bind(fd: RawFd, addr: &SockAddr) -> Result<()> {
    let res = unsafe {
        let (ptr, len) = addr.as_ffi_pair();
        ffi::bind(fd, ptr, len)
    };

    from_ffi(res)
}

/// Accept a connection on a socket
///
/// [Further reading](http://man7.org/linux/man-pages/man2/accept.2.html)
pub fn accept(sockfd: RawFd) -> Result<RawFd> {
    let res = unsafe { ffi::accept(sockfd, ptr::null_mut(), ptr::null_mut()) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(res)
}

/// Accept a connection on a socket
///
/// [Further reading](http://man7.org/linux/man-pages/man2/accept.2.html)
pub fn accept4(sockfd: RawFd, flags: SockFlag) -> Result<RawFd> {
    accept4_polyfill(sockfd, flags)
}

#[inline]
fn accept4_polyfill(sockfd: RawFd, flags: SockFlag) -> Result<RawFd> {
    let res =  unsafe { ffi::accept(sockfd, ptr::null_mut(), ptr::null_mut()) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    if flags.contains(SOCK_CLOEXEC) {
        try!(fcntl(res, F_SETFD(FD_CLOEXEC)));
    }

    if flags.contains(SOCK_NONBLOCK) {
        try!(fcntl(res, F_SETFL(O_NONBLOCK)));
    }

    Ok(res)
}

/// Initiate a connection on a socket
///
/// [Further reading](http://man7.org/linux/man-pages/man2/connect.2.html)
pub fn connect(fd: RawFd, addr: &SockAddr) -> Result<()> {
    let res = unsafe {
        let (ptr, len) = addr.as_ffi_pair();
        ffi::connect(fd, ptr, len)
    };

    from_ffi(res)
}

/// Receive data from a connectionless or connection-oriented socket. Returns
/// the number of bytes read and the socket address of the sender.
///
/// [Further reading](http://man7.org/linux/man-pages/man2/recvmsg.2.html)
pub fn recvfrom(sockfd: RawFd, buf: &mut [u8]) -> Result<(usize, SockAddr)> {
    unsafe {
        let addr: sockaddr_storage = mem::zeroed();
        let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

        let ret = ffi::recvfrom(
            sockfd,
            buf.as_ptr() as *mut c_void,
            buf.len() as size_t,
            0,
            mem::transmute(&addr),
            &mut len as *mut socklen_t);

        if ret < 0 {
            return Err(Error::last());
        }

        sockaddr_storage_to_addr(&addr, len as usize)
            .map(|addr| (ret as usize, addr))
    }
}

pub fn sendto(fd: RawFd, buf: &[u8], addr: &SockAddr, flags: SockMessageFlags) -> Result<usize> {
    let ret = unsafe {
        let (ptr, len) = addr.as_ffi_pair();
        ffi::sendto(fd, buf.as_ptr() as *const c_void, buf.len() as size_t, flags, ptr, len)
    };

    if ret < 0 {
        Err(Error::Sys(Errno::last()))
    } else {
        Ok(ret as usize)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct linger {
    pub l_onoff: c_int,
    pub l_linger: c_int
}

/*
 *
 * ===== Socket Options =====
 *
 */

/// The protocol level at which to get / set socket options. Used as an
/// argument to `getsockopt` and `setsockopt`.
///
/// [Further reading](http://man7.org/linux/man-pages/man2/setsockopt.2.html)
#[repr(i32)]
pub enum SockLevel {
    Socket = SOL_SOCKET,
    Tcp = IPPROTO_TCP,
    Ip = IPPROTO_IP,
    Ipv6 = IPPROTO_IPV6,
    Udp = IPPROTO_UDP,
}

/// Represents a socket option that can be accessed or set. Used as an argument
/// to `getsockopt` and `setsockopt`.
pub trait SockOpt : Copy + fmt::Debug {
    /// Type of `getsockopt` return value
    type Get;

    /// Type of value used to set the socket option. Used as the argument to
    /// `setsockopt`.
    type Set;

    #[doc(hidden)]
    fn get(&self, fd: RawFd, level: c_int) -> Result<Self::Get>;

    #[doc(hidden)]
    fn set(&self, fd: RawFd, level: c_int, val: Self::Set) -> Result<()>;
}

/// Get the current value for the requested socket option
///
/// [Further reading](http://man7.org/linux/man-pages/man2/setsockopt.2.html)
pub fn getsockopt<O: SockOpt>(fd: RawFd, level: SockLevel, opt: O) -> Result<O::Get> {
    opt.get(fd, level as c_int)
}

/// Sets the value for the requested socket option
///
/// [Further reading](http://man7.org/linux/man-pages/man2/setsockopt.2.html)
pub fn setsockopt<O: SockOpt>(fd: RawFd, level: SockLevel, opt: O, val: O::Set) -> Result<()> {
    opt.set(fd, level as c_int, val)
}

/// Get the address of the peer connected to the socket `fd`.
///
/// [Further reading](http://man7.org/linux/man-pages/man2/getpeername.2.html)
pub fn getpeername(fd: RawFd) -> Result<SockAddr> {
    unsafe {
        let addr: sockaddr_storage = mem::uninitialized();
        let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

        let ret = ffi::getpeername(fd, mem::transmute(&addr), &mut len);

        if ret < 0 {
            return Err(Error::last());
        }

        sockaddr_storage_to_addr(&addr, len as usize)
    }
}

/// Get the current address to which the socket `fd` is bound.
///
/// [Further reading](http://man7.org/linux/man-pages/man2/getsockname.2.html)
pub fn getsockname(fd: RawFd) -> Result<SockAddr> {
    unsafe {
        let addr: sockaddr_storage = mem::uninitialized();
        let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

        let ret = ffi::getsockname(fd, mem::transmute(&addr), &mut len);

        if ret < 0 {
            return Err(Error::last());
        }

        sockaddr_storage_to_addr(&addr, len as usize)
    }
}

pub unsafe fn sockaddr_storage_to_addr(
    addr: &sockaddr_storage,
    len: usize) -> Result<SockAddr> {

    match addr.ss_family as c_int {
        consts::AF_INET => {
            assert!(len as usize == mem::size_of::<sockaddr_in>());
            let ret = *(addr as *const _ as *const sockaddr_in);
            Ok(SockAddr::Inet(InetAddr::V4(ret)))
        }
        consts::AF_INET6 => {
            assert!(len as usize == mem::size_of::<sockaddr_in6>());
            Ok(SockAddr::Inet(InetAddr::V6((*(addr as *const _ as *const sockaddr_in6)))))
        }
        consts::AF_UNIX => {
            assert!(len as usize == mem::size_of::<sockaddr_un>());
            Ok(SockAddr::Unix(UnixAddr(*(addr as *const _ as *const sockaddr_un))))
        }
        af => panic!("unexpected address family {}", af),
    }
}

#[test]
pub fn test_struct_sizes() {
    use nixtest;
    nixtest::assert_size_of::<sockaddr_storage>("sockaddr_storage");
}
