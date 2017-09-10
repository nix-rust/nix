use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};
use std::mem::transmute;
use libc;

extern crate eui48;
use self::eui48::{MacAddress, Eui48};

/// Represents the actual data of an address in use by an interface.
#[derive(Debug, Clone, PartialEq)]
pub enum IfAddrValue {
    IpAddr(IpAddr),
    MacAddr(MacAddress),
}

impl From<IpAddr> for IfAddrValue {
    fn from(ip: IpAddr) -> IfAddrValue {
        IfAddrValue::IpAddr(ip)
    }
}

impl From<MacAddress> for IfAddrValue {
    fn from(mac: MacAddress) -> IfAddrValue {
        IfAddrValue::MacAddr(mac)
    }
}

/// Converts a `libc::sockaddr` into an `Option<IfAddrValue>`.
///
/// It returns `None` if the libc reports a type of address other than
/// IPv4, IPv6, or EUI-84 MAC, or if the given `sockaddr_input` was null.
///
/// # Unsafety
///
/// The caller is responsible for guaranteeing that the provided reference
/// refers to valid memory.
pub unsafe fn sockaddr_to_ifaddrvalue(sockaddr_input: *mut libc::sockaddr) 
    -> Option<IfAddrValue> {
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
            libc::AF_PACKET => {
                let data_mac: &libc::sockaddr_ll = transmute(sa);
                if data_mac.sll_halen != 6 {
                    None // If the length of the hardware address (halen) isn't
                        // 6, it's not EUI48 and we can't handle it.
                } else {
                    let a = data_mac.sll_addr;
                    let eui: Eui48 = [a[0], a[1], a[2], a[3], a[4], a[5]];
                    Some(MacAddress::new(eui).into())
                }
            }
            _ => None,
        }
    } else {
        None
    }
}
