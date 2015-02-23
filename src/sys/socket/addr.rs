use {NixResult, NixError};
use super::{sa_family_t, in_addr, sockaddr_in, sockaddr_in6, sockaddr_un, AF_UNIX, AF_INET};
use errno::Errno;
use std::{mem, net, path, ptr};
use std::ffi::{AsOsStr, CStr, OsStr};
use std::os::unix::OsStrExt;

/// Represents a socket address
#[derive(Copy)]
pub enum SockAddr {
    // TODO: Rename these variants IpV4, IpV6, Unix
    IpV4(sockaddr_in),
    IpV6(sockaddr_in6),
    Unix(sockaddr_un)
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

/// Convert an inet address into a socket address
impl ToSockAddr for net::SocketAddr {
    fn to_sock_addr(&self) -> NixResult<SockAddr> {
        use std::net::IpAddr;
        use std::num::Int;

        match self.ip() {
            IpAddr::V4(ip) => {
                let addr = ip.octets();
                Ok(SockAddr::IpV4(sockaddr_in {
                    sin_family: AF_INET as sa_family_t,
                    sin_port: self.port(),
                    sin_addr: in_addr {
                        s_addr: Int::from_be(
                            ((addr[0] as u32) << 24) |
                            ((addr[1] as u32) << 16) |
                            ((addr[2] as u32) <<  8) |
                            ((addr[3] as u32) <<  0))
                    },
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

                Some(net::SocketAddr::new(IpAddr::V4(ip), addr.sin_port))
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
