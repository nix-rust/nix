use {NixResult, NixError};
use super::{consts, sa_family_t, in_addr, sockaddr_in, sockaddr_in6, sockaddr_un, AF_UNIX, AF_INET};
use errno::Errno;
use libc;
use std::{fmt, mem, net, path, ptr};
use std::ffi::{AsOsStr, CStr, OsStr};
use std::os::unix::OsStrExt;

/*
 *
 * ===== AddressFamily =====
 *
 */

#[repr(i32)]
#[derive(Copy, PartialEq, Eq, Debug)]
pub enum AddressFamily {
    Unix = consts::AF_UNIX,
    Inet = consts::AF_INET,
    Inet6 = consts::AF_INET6,
}

/*
 *
 * ===== Sock addr =====
 *
 */

/// Represents a socket address
#[derive(Copy)]
pub enum SockAddr {
    IpV4(sockaddr_in),
    IpV6(sockaddr_in6),
    Unix(sockaddr_un)
}

impl SockAddr {
    pub fn family(&self) -> AddressFamily {
        match *self {
            SockAddr::IpV4(..) => AddressFamily::Inet,
            SockAddr::IpV6(..) => AddressFamily::Inet6,
            SockAddr::Unix(..) => AddressFamily::Unix,
        }
    }

    pub fn to_str(&self) -> String {
        format!("{}", self)
    }
}

impl fmt::Display for SockAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::num::Int;

        match *self {
            SockAddr::IpV4(sin) => {
                let ip = Int::from_be(sin.sin_addr.s_addr);
                let port = Int::from_be(sin.sin_port);

                write!(f, "{}.{}.{}.{}:{}",
                       (ip >> 24) & 0xff,
                       (ip >> 16) & 0xff,
                       (ip >> 8) & 0xff,
                       (ip) & 0xff,
                       port)
            }
            _ => write!(f, "[some sock addr type... Debug is not implemented :(]"),
        }
    }
}

/// A trait for values which can be converted or resolved to a SockAddr.
pub trait ToSockAddr {
    /// Converts the value to a SockAddr
    fn to_sock_addr(&self) -> NixResult<SockAddr>;

    /// Converts and yields the value as a SockAddr
    fn with_sock_addr<T, F: FnOnce(&SockAddr) -> T>(&self, action: F) -> NixResult<T> {
        Ok(action(&try!(self.to_sock_addr())))
    }
}

impl ToSockAddr for SockAddr {
    fn to_sock_addr(&self) -> NixResult<SockAddr> {
        Ok(*self)
    }

    fn with_sock_addr<T, F: FnOnce(&SockAddr) -> T>(&self, action: F) -> NixResult<T> {
        Ok(action(self))
    }
}

/// Convert a path into a unix domain socket address
impl ToSockAddr for path::Path {
    fn to_sock_addr(&self) -> NixResult<SockAddr> {
        let bytes = self.as_os_str().as_bytes();

        Ok(SockAddr::Unix(unsafe {
            let mut ret = sockaddr_un {
                sun_family: AF_UNIX as sa_family_t,
                .. mem::zeroed()
            };

            // Make sure the destination has enough capacity
            if bytes.len() >= ret.sun_path.len() {
                return Err(NixError::Sys(Errno::ENAMETOOLONG));
            }

            // Copy the path
            ptr::copy_memory(
                ret.sun_path.as_mut_ptr(),
                bytes.as_ptr() as *const i8,
                bytes.len());

            ret
        }))
    }
}

/// Convert a path buf into a unix domain socket address
impl ToSockAddr for path::PathBuf {
    fn to_sock_addr(&self) -> NixResult<SockAddr> {
        (**self).to_sock_addr()
    }
}

/// Convert an inet address into a socket address
impl ToSockAddr for net::SocketAddr {
    fn to_sock_addr(&self) -> NixResult<SockAddr> {
        use std::net::IpAddr;
        use std::num::Int;

        match self.ip() {
            IpAddr::V4(ip) => {
                let addr = ip.to_in_addr()
                    .expect("in_addr conversion expected to be successful");

                Ok(SockAddr::IpV4(sockaddr_in {
                    sin_family: AF_INET as sa_family_t,
                    sin_port: self.port().to_be(),
                    sin_addr: addr,
                    .. unsafe { mem::zeroed() }
                }))
            }
            _ => unimplemented!()
        }
    }
}

/// Convert from a socket address
pub trait FromSockAddr {
    fn from_sock_addr(addr: &SockAddr) -> Option<Self>;
}

impl FromSockAddr for net::SocketAddr {
    fn from_sock_addr(addr: &SockAddr) -> Option<net::SocketAddr> {
        use std::net::{IpAddr, Ipv4Addr};
        use std::num::Int;

        match *addr {
            SockAddr::IpV4(ref addr) => {
                let ip = Int::from_be(addr.sin_addr.s_addr);
                let ip = Ipv4Addr::new(
                    ((ip >> 24) as u8) & 0xff,
                    ((ip >> 16) as u8) & 0xff,
                    ((ip >>  8) as u8) & 0xff,
                    ((ip >>  0) as u8) & 0xff);

                Some(net::SocketAddr::new(IpAddr::V4(ip), Int::from_be(addr.sin_port)))
            }
            SockAddr::IpV6(_) => unimplemented!(),
            _ =>  None,
        }
    }
}

impl FromSockAddr for path::PathBuf {
    fn from_sock_addr(addr: &SockAddr) -> Option<path::PathBuf> {
        if let SockAddr::Unix(ref addr) = *addr {
            unsafe {
                let bytes = CStr::from_ptr(addr.sun_path.as_ptr()).to_bytes();
                let osstr = <OsStr as OsStrExt>::from_bytes(bytes);
                return Some(path::PathBuf::new(osstr));
            }
        }

        None
    }
}

/*
 *
 * ===== IpAddr =====
 *
 */

/// Convert to an IpAddr
pub trait ToIpAddr {
    fn to_ip_addr(self) -> Option<net::IpAddr>;
}

impl ToIpAddr for net::IpAddr {
    fn to_ip_addr(self) -> Option<net::IpAddr> {
        Some(self)
    }
}

impl<'a> ToIpAddr for &'a net::IpAddr {
    fn to_ip_addr(self) -> Option<net::IpAddr> {
        Some(*self)
    }
}

impl ToIpAddr for net::Ipv4Addr {
    fn to_ip_addr(self) -> Option<net::IpAddr> {
        Some(net::IpAddr::V4(self))
    }
}

impl<'a> ToIpAddr for &'a net::Ipv4Addr {
    fn to_ip_addr(self) -> Option<net::IpAddr> {
        (*self).to_ip_addr()
    }
}

impl ToIpAddr for net::Ipv6Addr {
    fn to_ip_addr(self) -> Option<net::IpAddr> {
        Some(net::IpAddr::V6(self))
    }
}

impl<'a> ToIpAddr for &'a net::Ipv6Addr {
    fn to_ip_addr(self) -> Option<net::IpAddr> {
        (*self).to_ip_addr()
    }
}

/*
 *
 * ===== InAddr =====
 *
 */

/// Convert to an in_addr
pub trait ToInAddr {
    fn to_in_addr(self) -> Option<libc::in_addr>;
}

impl ToInAddr for SockAddr {
    fn to_in_addr(self) -> Option<libc::in_addr> {
        match self {
            SockAddr::IpV4(sock) => Some(sock.sin_addr),
            _ => None,
        }
    }
}

impl<'a> ToInAddr for &'a SockAddr {
    fn to_in_addr(self) -> Option<libc::in_addr> {
        match *self {
            SockAddr::IpV4(ref sock) => Some(sock.sin_addr),
            _ => None,
        }
    }
}

impl ToInAddr for net::IpAddr {
    fn to_in_addr(self) -> Option<libc::in_addr> {
        match self {
            net::IpAddr::V4(addr) => addr.to_in_addr(),
            _ => None,
        }
    }
}

impl<'a> ToInAddr for &'a net::IpAddr {
    fn to_in_addr(self) -> Option<libc::in_addr> {
        match *self {
            net::IpAddr::V4(addr) => addr.to_in_addr(),
            _ => None,
        }
    }
}

impl ToInAddr for net::Ipv4Addr {
    fn to_in_addr(self) -> Option<libc::in_addr> {
        use std::num::Int;

        let addr = self.octets();
        let ip = (((addr[0] as u32) << 24) |
                  ((addr[1] as u32) << 16) |
                  ((addr[2] as u32) <<  8) |
                  ((addr[3] as u32) <<  0)).to_be();

        Some(in_addr { s_addr: ip })
    }
}

impl<'a> ToInAddr for &'a net::Ipv4Addr {
    fn to_in_addr(self) -> Option<libc::in_addr> {
        (*self).to_in_addr()
    }
}
