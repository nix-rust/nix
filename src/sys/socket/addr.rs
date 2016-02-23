use super::{consts, sa_family_t};
use {Errno, Error, Result, NixPath};
use libc;
use std::{fmt, hash, mem, net, ptr};
use std::ffi::OsStr;
use std::path::Path;
use std::os::unix::ffi::OsStrExt;
#[cfg(any(target_os = "linux", target_os = "android"))]
use ::sys::socket::addr::netlink::NetlinkAddr;

// TODO: uncomment out IpAddr functions: rust-lang/rfcs#988

/*
 *
 * ===== AddressFamily =====
 *
 */

#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum AddressFamily {
    Unix = consts::AF_UNIX,
    Inet = consts::AF_INET,
    Inet6 = consts::AF_INET6,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    Netlink = consts::AF_NETLINK,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    Packet = consts::AF_PACKET,
}

#[derive(Copy)]
pub enum InetAddr {
    V4(libc::sockaddr_in),
    V6(libc::sockaddr_in6),
}

impl InetAddr {
    pub fn from_std(std: &net::SocketAddr) -> InetAddr {
        let ip = match *std {
            net::SocketAddr::V4(ref addr) => IpAddr::V4(Ipv4Addr::from_std(&addr.ip())),
            net::SocketAddr::V6(ref addr) => IpAddr::V6(Ipv6Addr::from_std(&addr.ip())),
        };

        InetAddr::new(ip, std.port())
    }

    pub fn new(ip: IpAddr, port: u16) -> InetAddr {
        match ip {
            IpAddr::V4(ref ip) => {
                InetAddr::V4(libc::sockaddr_in {
                    sin_family: AddressFamily::Inet as sa_family_t,
                    sin_port: port.to_be(),
                    sin_addr: ip.0,
                    .. unsafe { mem::zeroed() }
                })
            }
            IpAddr::V6(ref ip) => {
                InetAddr::V6(libc::sockaddr_in6 {
                    sin6_family: AddressFamily::Inet6 as sa_family_t,
                    sin6_port: port.to_be(),
                    sin6_addr: ip.0,
                    .. unsafe { mem::zeroed() }
                })
            }
        }
    }
    /// Gets the IP address associated with this socket address.
    pub fn ip(&self) -> IpAddr {
        match *self {
            InetAddr::V4(ref sa) => IpAddr::V4(Ipv4Addr(sa.sin_addr)),
            InetAddr::V6(ref sa) => IpAddr::V6(Ipv6Addr(sa.sin6_addr)),
        }
    }

    /// Gets the port number associated with this socket address
    pub fn port(&self) -> u16 {
        match *self {
            InetAddr::V6(ref sa) => u16::from_be(sa.sin6_port),
            InetAddr::V4(ref sa) => u16::from_be(sa.sin_port),
        }
    }

    pub fn to_std(&self) -> net::SocketAddr {
        match *self {
            InetAddr::V4(ref sa) => net::SocketAddr::V4(
                net::SocketAddrV4::new(
                    Ipv4Addr(sa.sin_addr).to_std(),
                    self.port())),
            InetAddr::V6(ref sa) => net::SocketAddr::V6(
                net::SocketAddrV6::new(
                    Ipv6Addr(sa.sin6_addr).to_std(),
                    self.port(),
                    sa.sin6_flowinfo,
                    sa.sin6_scope_id)),
        }
    }

    pub fn to_str(&self) -> String {
        format!("{}", self)
    }
}

impl PartialEq for InetAddr {
    fn eq(&self, other: &InetAddr) -> bool {
        match (*self, *other) {
            (InetAddr::V4(ref a), InetAddr::V4(ref b)) => {
                a.sin_port == b.sin_port &&
                    a.sin_addr.s_addr == b.sin_addr.s_addr
            }
            (InetAddr::V6(ref a), InetAddr::V6(ref b)) => {
                a.sin6_port == b.sin6_port &&
                    a.sin6_addr.s6_addr == b.sin6_addr.s6_addr &&
                    a.sin6_flowinfo == b.sin6_flowinfo &&
                    a.sin6_scope_id == b.sin6_scope_id
            }
            _ => false,
        }
    }
}

impl Eq for InetAddr {
}

impl hash::Hash for InetAddr {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        match *self {
            InetAddr::V4(ref a) => {
                ( a.sin_family,
                  a.sin_port,
                  a.sin_addr.s_addr ).hash(s)
            }
            InetAddr::V6(ref a) => {
                ( a.sin6_family,
                  a.sin6_port,
                  &a.sin6_addr.s6_addr,
                  a.sin6_flowinfo,
                  a.sin6_scope_id ).hash(s)
            }
        }
    }
}

impl Clone for InetAddr {
    fn clone(&self) -> InetAddr {
        *self
    }
}

impl fmt::Display for InetAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InetAddr::V4(_) => write!(f, "{}:{}", self.ip(), self.port()),
            InetAddr::V6(_) => write!(f, "[{}]:{}", self.ip(), self.port()),
        }
    }
}

/*
 *
 * ===== IpAddr =====
 *
 */

pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

impl IpAddr {
    /// Create a new IpAddr that contains an IPv4 address.
    ///
    /// The result will represent the IP address a.b.c.d
    pub fn new_v4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(a, b, c, d))
    }

    /// Create a new IpAddr that contains an IPv6 address.
    ///
    /// The result will represent the IP address a:b:c:d:e:f
    pub fn new_v6(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> IpAddr {
        IpAddr::V6(Ipv6Addr::new(a, b, c, d, e, f, g, h))
    }

    /*
    pub fn from_std(std: &net::IpAddr) -> IpAddr {
        match *std {
            net::IpAddr::V4(ref std) => IpAddr::V4(Ipv4Addr::from_std(std)),
            net::IpAddr::V6(ref std) => IpAddr::V6(Ipv6Addr::from_std(std)),
        }
    }

    pub fn to_std(&self) -> net::IpAddr {
        match *self {
            IpAddr::V4(ref ip) => net::IpAddr::V4(ip.to_std()),
            IpAddr::V6(ref ip) => net::IpAddr::V6(ip.to_std()),
        }
    }
    */
}

impl fmt::Display for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IpAddr::V4(ref v4) => v4.fmt(f),
            IpAddr::V6(ref v6) => v6.fmt(f)
        }
    }
}

/*
 *
 * ===== Ipv4Addr =====
 *
 */

#[derive(Copy)]
pub struct Ipv4Addr(pub libc::in_addr);

impl Ipv4Addr {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Ipv4Addr {
        let ip = (((a as u32) << 24) |
                  ((b as u32) << 16) |
                  ((c as u32) <<  8) |
                  ((d as u32) <<  0)).to_be();

        Ipv4Addr(libc::in_addr { s_addr: ip })
    }

    pub fn from_std(std: &net::Ipv4Addr) -> Ipv4Addr {
        let bits = std.octets();
        Ipv4Addr::new(bits[0], bits[1], bits[2], bits[3])
    }

    pub fn any() -> Ipv4Addr {
        Ipv4Addr(libc::in_addr { s_addr: consts::INADDR_ANY })
    }

    pub fn octets(&self) -> [u8; 4] {
        let bits = u32::from_be(self.0.s_addr);
        [(bits >> 24) as u8, (bits >> 16) as u8, (bits >> 8) as u8, bits as u8]
    }

    pub fn to_std(&self) -> net::Ipv4Addr {
        let bits = self.octets();
        net::Ipv4Addr::new(bits[0], bits[1], bits[2], bits[3])
    }
}

impl PartialEq for Ipv4Addr {
    fn eq(&self, other: &Ipv4Addr) -> bool {
        self.0.s_addr == other.0.s_addr
    }
}

impl Eq for Ipv4Addr {
}

impl hash::Hash for Ipv4Addr {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        self.0.s_addr.hash(s)
    }
}

impl Clone for Ipv4Addr {
    fn clone(&self) -> Ipv4Addr {
        *self
    }
}

impl fmt::Display for Ipv4Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let octets = self.octets();
        write!(fmt, "{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
    }
}

/*
 *
 * ===== Ipv6Addr =====
 *
 */

#[derive(Clone, Copy)]
pub struct Ipv6Addr(pub libc::in6_addr);

// Note that IPv6 addresses are stored in big endian order on all architectures.
// See https://tools.ietf.org/html/rfc1700 or consult your favorite search
// engine.

macro_rules! to_u8_array {
    ($($num:ident),*) => {
        [ $(($num>>8) as u8, ($num&0xff) as u8,)* ]
    }
}

macro_rules! to_u16_array {
    ($slf:ident, $($first:expr, $second:expr),*) => {
        [$( (($slf.0.s6_addr[$first] as u16) << 8) + $slf.0.s6_addr[$second] as u16,)*]
    }
}

impl Ipv6Addr {
    pub fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Ipv6Addr {
        let mut in6_addr_var: libc::in6_addr = unsafe{mem::uninitialized()};
        in6_addr_var.s6_addr = to_u8_array!(a,b,c,d,e,f,g,h);
        Ipv6Addr(in6_addr_var)
    }

    pub fn from_std(std: &net::Ipv6Addr) -> Ipv6Addr {
        let s = std.segments();
        Ipv6Addr::new(s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7])
    }

    /// Return the eight 16-bit segments that make up this address
    pub fn segments(&self) -> [u16; 8] {
        to_u16_array!(self, 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15)
    }

    pub fn to_std(&self) -> net::Ipv6Addr {
        let s = self.segments();
        net::Ipv6Addr::new(s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7])
    }
}

impl fmt::Display for Ipv6Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.to_std().fmt(fmt)
    }
}

/*
 *
 * ===== UnixAddr =====
 *
 */

/// A wrapper around sockaddr_un. We track the length of sun_path,
/// because it may not be null-terminated (unconnected and abstract
/// sockets). Note that the actual sockaddr length is greater by
/// size_of::<sa_family_t>().
#[derive(Copy)]
pub struct UnixAddr(pub libc::sockaddr_un, pub usize);

impl UnixAddr {
    /// Create a new sockaddr_un representing a filesystem path.
    pub fn new<P: ?Sized + NixPath>(path: &P) -> Result<UnixAddr> {
        try!(path.with_nix_path(|cstr| {
            unsafe {
                let mut ret = libc::sockaddr_un {
                    sun_family: AddressFamily::Unix as sa_family_t,
                    .. mem::zeroed()
                };

                let bytes = cstr.to_bytes_with_nul();

                if bytes.len() > ret.sun_path.len() {
                    return Err(Error::Sys(Errno::ENAMETOOLONG));
                }

                ptr::copy_nonoverlapping(bytes.as_ptr(),
                                         ret.sun_path.as_mut_ptr() as *mut u8,
                                         bytes.len());

                Ok(UnixAddr(ret, bytes.len()))
            }
        }))
    }

    /// Create a new sockaddr_un representing an address in the
    /// "abstract namespace". This is a Linux-specific extension,
    /// primarily used to allow chrooted processes to communicate with
    /// specific daemons.
    pub fn new_abstract(path: &[u8]) -> Result<UnixAddr> {
        unsafe {
            let mut ret = libc::sockaddr_un {
                sun_family: AddressFamily::Unix as sa_family_t,
                .. mem::zeroed()
            };

            if path.len() > ret.sun_path.len() {
                return Err(Error::Sys(Errno::ENAMETOOLONG));
            }

            // Abstract addresses are represented by sun_path[0] ==
            // b'\0', so copy starting one byte in.
            ptr::copy_nonoverlapping(path.as_ptr(),
                                     ret.sun_path.as_mut_ptr().offset(1) as *mut u8,
                                     path.len());

            Ok(UnixAddr(ret, path.len()))
        }
    }

    fn sun_path(&self) -> &[u8] {
        unsafe { mem::transmute(&self.0.sun_path[..self.1]) }
    }

    /// If this address represents a filesystem path, return that path.
    pub fn path(&self) -> Option<&Path> {
        if self.1 == 0 || self.0.sun_path[0] == 0 {
            // unbound or abstract
            None
        } else {
            let p = self.sun_path();
            Some(Path::new(<OsStr as OsStrExt>::from_bytes(&p[..p.len()-1])))
        }
    }
}

impl PartialEq for UnixAddr {
    fn eq(&self, other: &UnixAddr) -> bool {
        self.sun_path() == other.sun_path()
    }
}

impl Eq for UnixAddr {
}

impl hash::Hash for UnixAddr {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        ( self.0.sun_family, self.sun_path() ).hash(s)
    }
}

impl Clone for UnixAddr {
    fn clone(&self) -> UnixAddr {
        *self
    }
}

impl fmt::Display for UnixAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.1 == 0 {
            f.write_str("<unbound UNIX socket>")
        } else if let Some(path) = self.path() {
            path.display().fmt(f)
        } else {
            let display = String::from_utf8_lossy(&self.sun_path()[1..]);
            write!(f, "@{}", display)
        }
    }
}

/*
 *
 * ===== Sock addr =====
 *
 */

/// Represents a socket address
#[derive(Copy)]
pub enum SockAddr {
    Inet(InetAddr),
    Unix(UnixAddr),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    Netlink(NetlinkAddr)
}

impl SockAddr {
    pub fn new_inet(addr: InetAddr) -> SockAddr {
        SockAddr::Inet(addr)
    }

    pub fn new_unix<P: ?Sized + NixPath>(path: &P) -> Result<SockAddr> {
        Ok(SockAddr::Unix(try!(UnixAddr::new(path))))
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub fn new_netlink(pid: u32, groups: u32) -> SockAddr {
        SockAddr::Netlink(NetlinkAddr::new(pid, groups))
    }

    pub fn family(&self) -> AddressFamily {
        match *self {
            SockAddr::Inet(InetAddr::V4(..)) => AddressFamily::Inet,
            SockAddr::Inet(InetAddr::V6(..)) => AddressFamily::Inet6,
            SockAddr::Unix(..) => AddressFamily::Unix,
            #[cfg(any(target_os = "linux", target_os = "android"))]
            SockAddr::Netlink(..) => AddressFamily::Netlink,
        }
    }

    pub fn to_str(&self) -> String {
        format!("{}", self)
    }

    pub unsafe fn as_ffi_pair(&self) -> (&libc::sockaddr, libc::socklen_t) {
        match *self {
            SockAddr::Inet(InetAddr::V4(ref addr)) => (mem::transmute(addr), mem::size_of::<libc::sockaddr_in>() as libc::socklen_t),
            SockAddr::Inet(InetAddr::V6(ref addr)) => (mem::transmute(addr), mem::size_of::<libc::sockaddr_in6>() as libc::socklen_t),
            SockAddr::Unix(UnixAddr(ref addr, len)) => (mem::transmute(addr), (len + mem::size_of::<libc::sa_family_t>()) as libc::socklen_t),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            SockAddr::Netlink(NetlinkAddr(ref sa)) => (mem::transmute(sa), mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t),
        }
    }
}

impl PartialEq for SockAddr {
    fn eq(&self, other: &SockAddr) -> bool {
        match (*self, *other) {
            (SockAddr::Inet(ref a), SockAddr::Inet(ref b)) => {
                a == b
            }
            (SockAddr::Unix(ref a), SockAddr::Unix(ref b)) => {
                a == b
            }
            #[cfg(any(target_os = "linux", target_os = "android"))]
            (SockAddr::Netlink(ref a), SockAddr::Netlink(ref b)) => {
                a == b
            }
            _ => false,
        }
    }
}

impl Eq for SockAddr {
}

impl hash::Hash for SockAddr {
    fn hash<H: hash::Hasher>(&self, s: &mut H) {
        match *self {
            SockAddr::Inet(ref a) => a.hash(s),
            SockAddr::Unix(ref a) => a.hash(s),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            SockAddr::Netlink(ref a) => a.hash(s),
        }
    }
}

impl Clone for SockAddr {
    fn clone(&self) -> SockAddr {
        *self
    }
}

impl fmt::Display for SockAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SockAddr::Inet(ref inet) => inet.fmt(f),
            SockAddr::Unix(ref unix) => unix.fmt(f),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            SockAddr::Netlink(ref nl) => nl.fmt(f),
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod netlink {
    use ::sys::socket::addr::{AddressFamily};
    use libc::{sa_family_t, sockaddr_nl};
    use std::{fmt, mem};
    use std::hash::{Hash, Hasher};

    #[derive(Copy, Clone)]
    pub struct NetlinkAddr(pub sockaddr_nl);

    // , PartialEq, Eq, Debug, Hash
    impl PartialEq for NetlinkAddr {
        fn eq(&self, other: &Self) -> bool {
            let (inner, other) = (self.0, other.0);
            (inner.nl_family, inner.nl_pid, inner.nl_groups) ==
            (other.nl_family, other.nl_pid, other.nl_groups)
        }
    }

    impl Eq for NetlinkAddr {}

    impl Hash for NetlinkAddr {
        fn hash<H: Hasher>(&self, s: &mut H) {
            let inner = self.0;
            (inner.nl_family, inner.nl_pid, inner.nl_groups).hash(s);
        }
    }


    impl NetlinkAddr {
        pub fn new(pid: u32, groups: u32) -> NetlinkAddr {
            let mut addr: sockaddr_nl = unsafe { mem::zeroed() };
            addr.nl_family = AddressFamily::Netlink as sa_family_t;
            addr.nl_pid = pid;
            addr.nl_groups = groups;

            NetlinkAddr(addr)
        }

        pub fn pid(&self) -> u32 {
            self.0.nl_pid
        }

        pub fn groups(&self) -> u32 {
            self.0.nl_groups
        }
    }

    impl fmt::Display for NetlinkAddr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "pid: {} groups: {}", self.pid(), self.groups())
        }
    }
}
