use {NixError, NixResult, from_ffi};
use errno::Errno;
use features;
use fcntl::{fcntl, FD_CLOEXEC, O_NONBLOCK};
use fcntl::FcntlArg::{F_SETFD, F_SETFL};
use libc::{c_void, c_int, socklen_t, size_t, ssize_t};
use std::{fmt, mem, ptr};
use std::os::unix::prelude::*;

mod addr;
mod consts;
mod ffi;
pub mod sockopt;

/*
 *
 * ===== Re-exports =====
 *
 */

pub use self::addr::{
    SockAddr,
    ToSockAddr,
    FromSockAddr
};
pub use libc::{
    in_addr,
    sockaddr,
    sockaddr_storage,
    sockaddr_in,
    sockaddr_in6,
    sockaddr_un,
    sa_family_t,
    ip_mreq
};
pub use self::consts::*;

// Extra flags - Supported by Linux 2.6.27, normalized on other platforms
bitflags!(
    flags SockFlag: c_int {
        const SOCK_NONBLOCK = 0o0004000,
        const SOCK_CLOEXEC  = 0o2000000
    }
);

pub fn socket(domain: AddressFamily, mut ty: SockType, flags: SockFlag) -> NixResult<Fd> {
    let feat_atomic = features::socket_atomic_cloexec();

    if feat_atomic {
        ty = ty | flags.bits();
    }

    // TODO: Check the kernel version
    let res = unsafe { ffi::socket(domain, ty, 0) };

    if res < 0 {
        return Err(NixError::Sys(Errno::last()));
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

pub fn listen(sockfd: Fd, backlog: usize) -> NixResult<()> {
    let res = unsafe { ffi::listen(sockfd, backlog as c_int) };
    from_ffi(res)
}

pub fn bind<A: ToSockAddr>(sockfd: Fd, addr: &A) -> NixResult<()> {
    let res = unsafe {
        try!(addr.with_sock_addr(|addr| {
            match *addr {
                SockAddr::IpV4(ref addr) => ffi::bind(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in>() as socklen_t),
                SockAddr::IpV6(ref addr) => ffi::bind(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in6>() as socklen_t),
                SockAddr::Unix(ref addr) => ffi::bind(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_un>() as socklen_t)
            }
        }))
    };

    from_ffi(res)
}

pub fn accept(sockfd: Fd) -> NixResult<Fd> {
    let res = unsafe { ffi::accept(sockfd, ptr::null_mut(), ptr::null_mut()) };

    if res < 0 {
        return Err(NixError::Sys(Errno::last()));
    }

    Ok(res)
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub fn accept4(sockfd: Fd, flags: SockFlag) -> NixResult<Fd> {
    use libc::sockaddr;

    type F = unsafe extern "C" fn(c_int, *mut sockaddr, *mut socklen_t, c_int) -> c_int;

    extern {
        #[linkage = "extern_weak"]
        static accept4: *const ();
    }

    if !accept4.is_null() {
        let res = unsafe {
            mem::transmute::<*const (), F>(accept4)(
                sockfd, ptr::null_mut(), ptr::null_mut(), flags.bits)
        };

        if res < 0 {
            return Err(NixError::Sys(Errno::last()));
        }

        Ok(res)
    } else {
        accept4_polyfill(sockfd, flags)
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn accept4(sockfd: Fd, flags: SockFlag) -> NixResult<Fd> {
    accept4_polyfill(sockfd, flags)
}

#[inline]
fn accept4_polyfill(sockfd: Fd, flags: SockFlag) -> NixResult<Fd> {
    let res =  unsafe { ffi::accept(sockfd, ptr::null_mut(), ptr::null_mut()) };

    if res < 0 {
        return Err(NixError::Sys(Errno::last()));
    }

    if flags.contains(SOCK_CLOEXEC) {
        try!(fcntl(res, F_SETFD(FD_CLOEXEC)));
    }

    if flags.contains(SOCK_NONBLOCK) {
        try!(fcntl(res, F_SETFL(O_NONBLOCK)));
    }

    Ok(res)
}

pub fn connect<A: ToSockAddr>(sockfd: Fd, addr: &A) -> NixResult<()> {
    let res = unsafe {
        try!(addr.with_sock_addr(|addr| {
            match *addr {
                SockAddr::IpV4(ref addr) => ffi::connect(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in>() as socklen_t),
                SockAddr::IpV6(ref addr) => ffi::connect(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in6>() as socklen_t),
                SockAddr::Unix(ref addr) => ffi::connect(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_un>() as socklen_t)
            }
        }))
    };

    from_ffi(res)
}

mod sa_helpers {
    use std::mem;
    use libc::{sockaddr_storage, sockaddr_in, sockaddr_in6, sockaddr_un};
    use super::SockAddr;

    pub fn to_sockaddr_ipv4(addr: &sockaddr_storage) -> SockAddr {
        let sin : &sockaddr_in = unsafe { mem::transmute(addr) };
        SockAddr::IpV4( *sin )
    }

    pub fn to_sockaddr_ipv6(addr: &sockaddr_storage) -> SockAddr {
        let sin6 : &sockaddr_in6 = unsafe { mem::transmute(addr) };
        SockAddr::IpV6( *sin6 )
    }

    pub fn to_sockaddr_unix(addr: &sockaddr_storage) -> SockAddr {
        let sun : &sockaddr_un = unsafe { mem::transmute(addr) };
        SockAddr::Unix( *sun )
    }
}

pub fn recvfrom(sockfd: Fd, buf: &mut [u8]) -> NixResult<(usize, SockAddr)> {
    let saddr : sockaddr_storage = unsafe { mem::zeroed() };
    let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

    let ret = unsafe {
        ffi::recvfrom(sockfd, buf.as_ptr() as *mut c_void, buf.len() as size_t, 0, mem::transmute(&saddr), &mut len as *mut socklen_t)
    };

    if ret < 0 {
        return Err(NixError::Sys(Errno::last()));
    }

    Ok((ret as usize,
            match saddr.ss_family as i32 {
                AF_INET => { sa_helpers::to_sockaddr_ipv4(&saddr) },
                AF_INET6 => { sa_helpers::to_sockaddr_ipv6(&saddr) },
                AF_UNIX => { sa_helpers::to_sockaddr_unix(&saddr) },
                _ => unimplemented!()
            }
        ))
}

fn print_ipv4_addr(sin: &sockaddr_in, f: &mut fmt::Formatter) -> fmt::Result {
    use std::num::Int;

    let ip_addr = Int::from_be(sin.sin_addr.s_addr);
    let port = Int::from_be(sin.sin_port);

    write!(f, "{}.{}.{}.{}:{}",
           (ip_addr >> 24) & 0xff,
           (ip_addr >> 16) & 0xff,
           (ip_addr >> 8) & 0xff,
           (ip_addr) & 0xff,
           port)
}

impl fmt::Debug for SockAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SockAddr::IpV4(sin) => print_ipv4_addr(&sin, f),
            _ => unimplemented!()
        }
    }
}

///
/// Generic wrapper around sendto
fn sendto_sockaddr<T>(sockfd: Fd, buf: &[u8], flags: SockMessageFlags, addr: &T) -> ssize_t {
    unsafe {
        ffi::sendto(
            sockfd,
            buf.as_ptr() as *const c_void,
            buf.len() as size_t,
            flags,
            mem::transmute(addr),
            mem::size_of::<T>() as socklen_t)
    }
}

pub fn sendto(sockfd: Fd, buf: &[u8], addr: &SockAddr, flags: SockMessageFlags) -> NixResult<usize> {
    let ret = match *addr {
        SockAddr::IpV4(ref addr) => sendto_sockaddr(sockfd, buf, flags, addr),
        SockAddr::IpV6(ref addr) => sendto_sockaddr(sockfd, buf, flags, addr),
        SockAddr::Unix(ref addr) => sendto_sockaddr(sockfd, buf, flags, addr),
    };

    if ret < 0 {
        Err(NixError::Sys(Errno::last()))
    } else {
        Ok(ret as usize)
    }
}

#[repr(C)]
#[derive(Copy)]
pub struct linger {
    pub l_onoff: c_int,
    pub l_linger: c_int
}

/*
 *
 * ===== Socket Options =====
 *
 */

/// Represents a socket option that can be accessed or set
pub trait SockOpt : Copy + fmt::Debug {
    /// Type of `getsockopt` return value
    type Get;

    /// Type of value used to set the socket option. Used as the argument to
    /// `setsockopt`.
    type Set;

    #[doc(hidden)]
    fn get(&self, fd: Fd, level: c_int) -> NixResult<Self::Get>;

    #[doc(hidden)]
    fn set(&self, fd: Fd, level: c_int, val: Self::Set) -> NixResult<()>;
}

pub enum SockLevel {
    Socket,
    Tcp,
    Ip,
    Ipv6,
    Udp
}

impl SockLevel {
    fn as_cint(&self) -> c_int {
        use self::SockLevel::*;

        match *self {
            Socket => consts::SOL_SOCKET,
            Tcp    => consts::IPPROTO_TCP,
            Ip     => consts::IPPROTO_IP,
            Ipv6   => consts::IPPROTO_IPV6,
            Udp    => consts::IPPROTO_UDP,
        }
    }
}

/// Get the current value for the requested socket option
///
/// [Further reading](http://man7.org/linux/man-pages/man2/setsockopt.2.html)
pub fn getsockopt<O: SockOpt>(fd: Fd, level: SockLevel, opt: O) -> NixResult<O::Get> {
    opt.get(fd, level.as_cint())
}

/// Sets the value for the requested socket option
///
/// [Further reading](http://man7.org/linux/man-pages/man2/setsockopt.2.html)
pub fn setsockopt<O: SockOpt>(fd: Fd, level: SockLevel, opt: O, val: O::Set) -> NixResult<()> {
    opt.set(fd, level.as_cint(), val)
}

fn getpeername_sockaddr<T>(sockfd: Fd, addr: &T) -> NixResult<bool> {
    let addrlen_expected = mem::size_of::<T>() as socklen_t;
    let mut addrlen = addrlen_expected;

    let ret = unsafe { ffi::getpeername(sockfd, mem::transmute(addr), &mut addrlen) };
    if ret < 0 {
        return Err(NixError::Sys(Errno::last()));
    }

    Ok(addrlen == addrlen_expected)
}

pub fn getpeername(sockfd: Fd, addr: &mut SockAddr) -> NixResult<bool> {
    match *addr {
        SockAddr::IpV4(ref addr) => getpeername_sockaddr(sockfd, addr),
        SockAddr::IpV6(ref addr) => getpeername_sockaddr(sockfd, addr),
        SockAddr::Unix(ref addr) => getpeername_sockaddr(sockfd, addr)
    }
}

fn getsockname_sockaddr<T>(sockfd: Fd, addr: &T) -> NixResult<bool> {
    let addrlen_expected = mem::size_of::<T>() as socklen_t;
    let mut addrlen = addrlen_expected;

    let ret = unsafe { ffi::getsockname(sockfd, mem::transmute(addr), &mut addrlen) };
    if ret < 0 {
        return Err(NixError::Sys(Errno::last()));
    }

    Ok(addrlen == addrlen_expected)
}

pub fn getsockname(sockfd: Fd, addr: &mut SockAddr) -> NixResult<bool> {
    match *addr {
        SockAddr::IpV4(ref addr) => getsockname_sockaddr(sockfd, addr),
        SockAddr::IpV6(ref addr) => getsockname_sockaddr(sockfd, addr),
        SockAddr::Unix(ref addr) => getsockname_sockaddr(sockfd, addr)
    }
}
