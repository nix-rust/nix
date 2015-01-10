use std::{mem, ptr, fmt};
use libc::{c_void, c_int, socklen_t, size_t, ssize_t};
use fcntl::{Fd, fcntl, FD_CLOEXEC, O_NONBLOCK};
use fcntl::FcntlArg::{F_SETFD, F_SETFL};
use errno::{SysResult, SysError, from_ffi};
use features;

pub use libc::{in_addr, sockaddr, sockaddr_storage, sockaddr_in, sockaddr_in6, sockaddr_un, sa_family_t, ip_mreq};

pub use self::consts::*;

mod ffi {
    use libc::{c_int, c_void, socklen_t};
    pub use libc::{socket, listen, bind, accept, connect, setsockopt, sendto, recvfrom, getsockname, getpeername};

    extern {
        pub fn getsockopt(
            sockfd: c_int,
            level: c_int,
            optname: c_int,
            optval: *mut c_void,
            optlen: *mut socklen_t) -> c_int;
    }
}

// Extra flags - Supported by Linux 2.6.27, normalized on other platforms
bitflags!(
    flags SockFlag: c_int {
        const SOCK_NONBLOCK = 0o0004000,
        const SOCK_CLOEXEC  = 0o2000000
    }
);

#[derive(Copy)]
pub enum SockAddr {
    SockIpV4(sockaddr_in),
    SockIpV6(sockaddr_in6),
    SockUnix(sockaddr_un)
}

#[cfg(target_os = "linux")]
mod consts {
    use libc::{c_int, uint8_t};

    pub type AddressFamily = c_int;

    pub const AF_UNIX: AddressFamily  = 1;
    pub const AF_LOCAL: AddressFamily = AF_UNIX;
    pub const AF_INET: AddressFamily  = 2;
    pub const AF_INET6: AddressFamily = 10;

    pub type SockType = c_int;

    pub const SOCK_STREAM: SockType = 1;
    pub const SOCK_DGRAM: SockType = 2;
    pub const SOCK_SEQPACKET: SockType = 5;
    pub const SOCK_RAW: SockType = 3;
    pub const SOCK_RDM: SockType = 4;

    pub type SockLevel = c_int;

    pub const SOL_IP: SockLevel     = 0;
    pub const IPPROTO_IP: SockLevel = SOL_IP;
    pub const SOL_SOCKET: SockLevel = 1;
    pub const SOL_TCP: SockLevel    = 6;
    pub const IPPROTO_TCP: SockLevel = SOL_TCP;
    pub const SOL_UDP: SockLevel    = 17;
    pub const SOL_IPV6: SockLevel   = 41;

    pub type SockOpt = c_int;

    pub const SO_ACCEPTCONN: SockOpt = 30;
    pub const SO_BINDTODEVICE: SockOpt = 25;
    pub const SO_BROADCAST: SockOpt = 6;
    pub const SO_BSDCOMPAT: SockOpt = 14;
    pub const SO_DEBUG: SockOpt = 1;
    pub const SO_DOMAIN: SockOpt = 39;
    pub const SO_ERROR: SockOpt = 4;
    pub const SO_DONTROUTE: SockOpt = 5;
    pub const SO_KEEPALIVE: SockOpt = 9;
    pub const SO_LINGER: SockOpt = 13;
    pub const SO_MARK: SockOpt = 36;
    pub const SO_OOBINLINE: SockOpt = 10;
    pub const SO_PASSCRED: SockOpt = 16;
    pub const SO_PEEK_OFF: SockOpt = 42;
    pub const SO_PEERCRED: SockOpt = 17;
    pub const SO_PRIORITY: SockOpt = 12;
    pub const SO_PROTOCOL: SockOpt = 38;
    pub const SO_RCVBUF: SockOpt = 8;
    pub const SO_RCVBUFFORCE: SockOpt = 33;
    pub const SO_RCVLOWAT: SockOpt = 18;
    pub const SO_SNDLOWAT: SockOpt = 19;
    pub const SO_RCVTIMEO: SockOpt = 20;
    pub const SO_SNDTIMEO: SockOpt = 21;
    pub const SO_REUSEADDR: SockOpt = 2;
    pub const SO_REUSEPORT: SockOpt = 15;
    pub const SO_RXQ_OVFL: SockOpt = 40;
    pub const SO_SNDBUF: SockOpt = 7;
    pub const SO_SNDBUFFORCE: SockOpt = 32;
    pub const SO_TIMESTAMP: SockOpt = 29;
    pub const SO_TYPE: SockOpt = 3;
    pub const SO_BUSY_POLL: SockOpt = 46;

    // Socket options for TCP sockets
    pub const TCP_NODELAY: SockOpt = 1;
    pub const TCP_MAXSEG: SockOpt = 2;
    pub const TCP_CORK: SockOpt = 3;

    // Socket options for the IP layer of the socket
    pub const IP_MULTICAST_IF: SockOpt = 32;

    pub type IpMulticastTtl = uint8_t;

    pub const IP_MULTICAST_TTL: SockOpt = 33;
    pub const IP_MULTICAST_LOOP: SockOpt = 34;
    pub const IP_ADD_MEMBERSHIP: SockOpt = 35;
    pub const IP_DROP_MEMBERSHIP: SockOpt = 36;

    pub type InAddrT = u32;

    // Declarations of special addresses
    pub const INADDR_ANY: InAddrT = 0;
    pub const INADDR_NONE: InAddrT = 0xffffffff;
    pub const INADDR_BROADCAST: InAddrT = 0xffffffff;

    pub type SockMessageFlags = i32;
    // Flags for send/recv and their relatives
    pub const MSG_OOB: SockMessageFlags = 0x1;
    pub const MSG_PEEK: SockMessageFlags = 0x2;
    pub const MSG_DONTWAIT: SockMessageFlags = 0x40;
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod consts {
    use libc::{c_int, uint8_t};

    pub type AddressFamily = c_int;

    pub const AF_UNIX: AddressFamily  = 1;
    pub const AF_LOCAL: AddressFamily = AF_UNIX;
    pub const AF_INET: AddressFamily  = 2;
    pub const AF_INET6: AddressFamily = 30;

    pub type SockType = c_int;

    pub const SOCK_STREAM: SockType = 1;
    pub const SOCK_DGRAM: SockType = 2;
    pub const SOCK_SEQPACKET: SockType = 5;
    pub const SOCK_RAW: SockType = 3;
    pub const SOCK_RDM: SockType = 4;

    pub type SockLevel = c_int;

    pub const SOL_SOCKET: SockLevel = 0xffff;
    pub const IPPROTO_IP: SockLevel = 0;
    pub const IPPROTO_TCP: SockLevel = 6;
    pub const IPPROTO_UDP: SockLevel = 17;

    pub type SockOpt = c_int;

    pub const SO_ACCEPTCONN: SockOpt          = 0x0002;
    pub const SO_BROADCAST: SockOpt           = 0x0020;
    pub const SO_DEBUG: SockOpt               = 0x0001;
    pub const SO_DONTTRUNC: SockOpt           = 0x2000;
    pub const SO_ERROR: SockOpt               = 0x1007;
    pub const SO_DONTROUTE: SockOpt           = 0x0010;
    pub const SO_KEEPALIVE: SockOpt           = 0x0008;
    pub const SO_LABEL: SockOpt               = 0x1010;
    pub const SO_LINGER: SockOpt              = 0x0080;
    pub const SO_NREAD: SockOpt               = 0x1020;
    pub const SO_NKE: SockOpt                 = 0x1021;
    pub const SO_NOSIGPIPE: SockOpt           = 0x1022;
    pub const SO_NOADDRERR: SockOpt           = 0x1023;
    pub const SO_NOTIFYCONFLICT: SockOpt      = 0x1026;
    pub const SO_NP_EXTENSIONS: SockOpt       = 0x1083;
    pub const SO_NWRITE: SockOpt              = 0x1024;
    pub const SO_OOBINLINE: SockOpt           = 0x0100;
    pub const SO_PEERLABEL: SockOpt           = 0x1011;
    pub const SO_RCVBUF: SockOpt              = 0x1002;
    pub const SO_RCVLOWAT: SockOpt            = 0x1004;
    pub const SO_SNDLOWAT: SockOpt            = 0x1003;
    pub const SO_RCVTIMEO: SockOpt            = 0x1006;
    pub const SO_SNDTIMEO: SockOpt            = 0x1005;
    pub const SO_RANDOMPORT: SockOpt          = 0x1082;
    pub const SO_RESTRICTIONS: SockOpt        = 0x1081;
    pub const SO_RESTRICT_DENYIN: SockOpt     = 0x00000001;
    pub const SO_RESTRICT_DENYOUT: SockOpt    = 0x00000002;
    pub const SO_REUSEADDR: SockOpt           = 0x0004;
    pub const SO_REUSEPORT: SockOpt           = 0x0200;
    pub const SO_REUSESHAREUID: SockOpt       = 0x1025;
    pub const SO_SNDBUF: SockOpt              = 0x1001;
    pub const SO_TIMESTAMP: SockOpt           = 0x0400;
    pub const SO_TIMESTAMP_MONOTONIC: SockOpt = 0x0800;
    pub const SO_TYPE: SockOpt                = 0x1008;
    pub const SO_WANTMORE: SockOpt            = 0x4000;
    pub const SO_WANTOOBFLAG: SockOpt         = 0x8000;
    #[allow(overflowing_literals)]
    pub const SO_RESTRICT_DENYSET: SockOpt    = 0x80000000;

    // Socket options for TCP sockets
    pub const TCP_NODELAY: SockOpt = 1;
    pub const TCP_MAXSEG: SockOpt = 2;

    // Socket options for the IP layer of the socket
    pub const IP_MULTICAST_IF: SockOpt = 9;

    pub type IpMulticastTtl = uint8_t;

    pub const IP_MULTICAST_TTL: SockOpt = 10;
    pub const IP_MULTICAST_LOOP: SockOpt = 11;
    pub const IP_ADD_MEMBERSHIP: SockOpt = 12;
    pub const IP_DROP_MEMBERSHIP: SockOpt = 13;

    pub type InAddrT = u32;

    // Declarations of special addresses
    pub const INADDR_ANY: InAddrT = 0;
    pub const INADDR_NONE: InAddrT = 0xffffffff;
    pub const INADDR_BROADCAST: InAddrT = 0xffffffff;

    pub type SockMessageFlags = i32;
    // Flags for send/recv and their relatives
    pub const MSG_OOB: SockMessageFlags = 0x1;
    pub const MSG_PEEK: SockMessageFlags = 0x2;
    pub const MSG_DONTWAIT: SockMessageFlags = 0x80;
}

pub fn socket(domain: AddressFamily, mut ty: SockType, flags: SockFlag) -> SysResult<Fd> {
    let feat_atomic = features::socket_atomic_cloexec();

    if feat_atomic {
        ty = ty | flags.bits();
    }

    // TODO: Check the kernel version
    let res = unsafe { ffi::socket(domain, ty, 0) };

    if res < 0 {
        return Err(SysError::last());
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

pub fn listen(sockfd: Fd, backlog: usize) -> SysResult<()> {
    let res = unsafe { ffi::listen(sockfd, backlog as c_int) };
    from_ffi(res)
}

pub fn bind(sockfd: Fd, addr: &SockAddr) -> SysResult<()> {
    use self::SockAddr::*;

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
    let res = unsafe { ffi::accept(sockfd, ptr::null_mut(), ptr::null_mut()) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub fn accept4(sockfd: Fd, flags: SockFlag) -> SysResult<Fd> {
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
            return Err(SysError::last());
        }

        Ok(res)
    } else {
        accept4_polyfill(sockfd, flags)
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn accept4(sockfd: Fd, flags: SockFlag) -> SysResult<Fd> {
    accept4_polyfill(sockfd, flags)
}

#[inline]
fn accept4_polyfill(sockfd: Fd, flags: SockFlag) -> SysResult<Fd> {
    let res =  unsafe { ffi::accept(sockfd, ptr::null_mut(), ptr::null_mut()) };

    if res < 0 {
        return Err(SysError::last());
    }

    if flags.contains(SOCK_CLOEXEC) {
        try!(fcntl(res, F_SETFD(FD_CLOEXEC)));
    }

    if flags.contains(SOCK_NONBLOCK) {
        try!(fcntl(res, F_SETFL(O_NONBLOCK)));
    }

    Ok(res)
}

pub fn connect(sockfd: Fd, addr: &SockAddr) -> SysResult<()> {
    use self::SockAddr::*;

    let res = unsafe {
        match *addr {
            SockIpV4(ref addr) => ffi::connect(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in>() as socklen_t),
            SockIpV6(ref addr) => ffi::connect(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_in6>() as socklen_t),
            SockUnix(ref addr) => ffi::connect(sockfd, mem::transmute(addr), mem::size_of::<sockaddr_un>() as socklen_t)
        }
    };

    from_ffi(res)
}

mod sa_helpers {
    use std::mem;
    use libc::{sockaddr_storage, sockaddr_in, sockaddr_in6, sockaddr_un};
    use super::SockAddr;
    use super::SockAddr::*;

    pub fn to_sockaddr_ipv4(addr: &sockaddr_storage) -> SockAddr {
        let sin : &sockaddr_in = unsafe { mem::transmute(addr) };
        SockIpV4( *sin )
    }

    pub fn to_sockaddr_ipv6(addr: &sockaddr_storage) -> SockAddr {
        let sin6 : &sockaddr_in6 = unsafe { mem::transmute(addr) };
        SockIpV6( *sin6 )
    }

    pub fn to_sockaddr_unix(addr: &sockaddr_storage) -> SockAddr {
        let sun : &sockaddr_un = unsafe { mem::transmute(addr) };
        SockUnix( *sun )
    }
}

pub fn recvfrom(sockfd: Fd, buf: &mut [u8]) -> SysResult<(usize, SockAddr)> {
    let saddr : sockaddr_storage = unsafe { mem::zeroed() };
    let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

    let ret = unsafe {
        ffi::recvfrom(sockfd, buf.as_ptr() as *mut c_void, buf.len() as size_t, 0, mem::transmute(&saddr), &mut len as *mut socklen_t)
    };

    if ret < 0 {
        return Err(SysError::last());
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

impl fmt::Show for SockAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SockAddr::SockIpV4(sin) => print_ipv4_addr(&sin, f),
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

pub fn sendto(sockfd: Fd, buf: &[u8], addr: &SockAddr, flags: SockMessageFlags) -> SysResult<usize> {
    use self::SockAddr::*;

    let ret = match *addr {
        SockIpV4(ref addr) => sendto_sockaddr(sockfd, buf, flags, addr),
        SockIpV6(ref addr) => sendto_sockaddr(sockfd, buf, flags, addr),
        SockUnix(ref addr) => sendto_sockaddr(sockfd, buf, flags, addr),
    };

    if ret < 0 {
        Err(SysError::last())
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

pub fn getsockopt<T>(fd: Fd, level: SockLevel, opt: SockOpt, val: &mut T) -> SysResult<usize> {
    let mut len = mem::size_of::<T>() as socklen_t;

    let res = unsafe {
        ffi::getsockopt(
            fd, level, opt,
            mem::transmute(val),
            &mut len as *mut socklen_t)
    };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(len as usize)
}

pub fn setsockopt<T>(fd: Fd, level: SockLevel, opt: SockOpt, val: &T) -> SysResult<()> {
    let len = mem::size_of::<T>() as socklen_t;

    let res = unsafe {
            ffi::setsockopt(
            fd, level, opt,
            mem::transmute(val),
            len)
    };

    from_ffi(res)
}

fn getpeername_sockaddr<T>(sockfd: Fd, addr: &T) -> SysResult<bool> {
    let addrlen_expected = mem::size_of::<T>() as socklen_t;
    let mut addrlen = addrlen_expected;

    let ret = unsafe { ffi::getpeername(sockfd, mem::transmute(addr), &mut addrlen) };
    if ret < 0 {
        return Err(SysError::last());
    }

    Ok(addrlen == addrlen_expected)
}

pub fn getpeername(sockfd: Fd, addr: &mut SockAddr) -> SysResult<bool> {
    use self::SockAddr::*;

    match *addr {
        SockIpV4(ref addr) => getpeername_sockaddr(sockfd, addr),
        SockIpV6(ref addr) => getpeername_sockaddr(sockfd, addr),
        SockUnix(ref addr) => getpeername_sockaddr(sockfd, addr)
    }
}

fn getsockname_sockaddr<T>(sockfd: Fd, addr: &T) -> SysResult<bool> {
    let addrlen_expected = mem::size_of::<T>() as socklen_t;
    let mut addrlen = addrlen_expected;

    let ret = unsafe { ffi::getsockname(sockfd, mem::transmute(addr), &mut addrlen) };
    if ret < 0 {
        return Err(SysError::last());
    }

    Ok(addrlen == addrlen_expected)
}

pub fn getsockname(sockfd: Fd, addr: &mut SockAddr) -> SysResult<bool> {
    use self::SockAddr::*;

    match *addr {
        SockIpV4(ref addr) => getsockname_sockaddr(sockfd, addr),
        SockIpV6(ref addr) => getsockname_sockaddr(sockfd, addr),
        SockUnix(ref addr) => getsockname_sockaddr(sockfd, addr)
    }
}
