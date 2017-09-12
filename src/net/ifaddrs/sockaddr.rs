use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};
use std::mem::transmute;
use std::fmt;
use libc;

/// Represents the actual data of an address in use by an interface.
#[derive(Clone)]
pub enum IfAddrValue<'a> {
    IpAddr(IpAddr),
    Other(&'a libc::sockaddr),
}

impl<'a> fmt::Debug for IfAddrValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IfAddrValue::IpAddr(ref addr) => write!(f, "IfAddrValue({:?})", addr),
            IfAddrValue::Other(_) => write!(f, "IfAddrValue(<Unknown Address Type>)"),
        }
    }
}

impl<'a> From<IpAddr> for IfAddrValue<'a> {
    fn from(ip: IpAddr) -> IfAddrValue<'a> {
        IfAddrValue::IpAddr(ip)
    }
}

impl<'a> From<&'a libc::sockaddr> for IfAddrValue<'a> {
    fn from(addr: &'a libc::sockaddr) -> IfAddrValue<'a> {
        IfAddrValue::Other(addr)
    }
}

/// Converts a `libc::sockaddr` into an `Option<IfAddrValue>`.
///
/// It returns `None` if the libc reports a type of address other than
/// IPv4, or IPv6, or if the given `sockaddr_input` was null.
///
/// # Unsafety
///
/// The caller is responsible for guaranteeing that the provided reference
/// refers to valid memory.
pub unsafe fn sockaddr_to_ifaddrvalue<'a>(sockaddr_input: *mut libc::sockaddr) 
    -> Option<IfAddrValue<'a>> {
    if let Some(sa) = sockaddr_input.as_ref() {
        // Only IPv4 and IPv6 are supported.
        match sa.sa_family as i32 {
            libc::AF_INET => {
                let data_v4: &libc::sockaddr_in = transmute(sa);
                // Transmuting a u32 into a [u8; 4] because
                // the address is in network byte order.
                let s_addr_v4: [u8; 4] = transmute(data_v4.sin_addr.s_addr);
                Some(IpAddr::V4(Ipv4Addr::from(s_addr_v4)).into())
            }
            libc::AF_INET6 => {
                let data_v6: &libc::sockaddr_in6 = transmute(sa);
                Some(IpAddr::V6(Ipv6Addr::from(data_v6.sin6_addr.s6_addr)).into())
            }
            _ => {
                Some(sa.into())
            },
        }
    } else {
        None
    }
}
