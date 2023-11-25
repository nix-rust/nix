//! Query network interface addresses
//!
//! Uses the Linux and/or BSD specific function `getifaddrs` to query the list
//! of interfaces and their associated addresses.
use cfg_if::cfg_if;
use std::ffi;
use std::iter::Iterator;
use std::marker::PhantomData;
use std::mem;
use std::option::Option;

use crate::net::if_::*;
use crate::sys::socket::RawAddr;
use crate::{Errno, Result};

/// Describes a single address for an interface as returned by `getifaddrs`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceAddress<'a> {
    /// Name of the network interface
    pub interface_name: String,
    /// Flags as from `SIOCGIFFLAGS` ioctl
    pub flags: InterfaceFlags,
    /// Network address of this interface
    pub address: Option<RawAddr<'a>>,
    /// Netmask of this interface
    pub netmask: Option<RawAddr<'a>>,
    /// Broadcast address of this interface, if applicable
    pub broadcast: Option<RawAddr<'a>>,
    /// Point-to-point destination address
    pub destination: Option<RawAddr<'a>>,
}

cfg_if! {
    if #[cfg(any(target_os = "android", target_os = "emscripten", target_os = "fuchsia", target_os = "linux"))] {
        fn get_ifu_from_sockaddr(info: &libc::ifaddrs) -> *const libc::sockaddr {
            info.ifa_ifu
        }
    } else {
        fn get_ifu_from_sockaddr(info: &libc::ifaddrs) -> *const libc::sockaddr {
            info.ifa_dstaddr
        }
    }
}

impl<'a> InterfaceAddress<'a> {
    /// Create an `InterfaceAddress` from the libc struct.
    fn from_libc_ifaddrs(info: &libc::ifaddrs) -> InterfaceAddress {
        let ifname = unsafe { ffi::CStr::from_ptr(info.ifa_name) };
        let address = unsafe { RawAddr::new(&*info.ifa_addr) };
        let netmask = unsafe { RawAddr::new(&*info.ifa_netmask) };
        let mut addr = InterfaceAddress {
            interface_name: ifname.to_string_lossy().to_string(),
            flags: InterfaceFlags::from_bits_truncate(info.ifa_flags as i32),
            address,
            netmask,
            broadcast: None,
            destination: None,
        };

        let ifu = get_ifu_from_sockaddr(info);
        if addr.flags.contains(InterfaceFlags::IFF_POINTOPOINT) {
            addr.destination = unsafe { RawAddr::new(&*ifu) };
        } else if addr.flags.contains(InterfaceFlags::IFF_BROADCAST) {
            addr.broadcast = unsafe { RawAddr::new(&*ifu) };
        }

        addr
    }
}

/// Holds the results of `getifaddrs`.
///
/// Use the function `getifaddrs` to create this struct and [`Self::iter`]
/// to create the iterator. Note that the actual list of interfaces can be
/// iterated once and will be freed as soon as the Iterator goes out of scope.
#[derive(Debug)]
pub struct InterfaceAddresses {
    base: *mut libc::ifaddrs,
}

impl InterfaceAddresses {
    /// Create an iterator over the list of interfaces.
    pub fn iter(&self) -> InterfaceAddressIterator<'_> {
        InterfaceAddressIterator {
            next: self.base,
            _a: PhantomData,
        }
    }
}

impl Drop for InterfaceAddresses {
    fn drop(&mut self) {
        unsafe { libc::freeifaddrs(self.base) };
    }
}

/// Holds the results of `getifaddrs`.
///
/// Use the function `getifaddrs` to create this Iterator. Note that the
/// actual list of interfaces can be iterated once and will be freed as
/// soon as the Iterator goes out of scope.
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct InterfaceAddressIterator<'a> {
    next: *mut libc::ifaddrs,
    _a: PhantomData<&'a ()>,
}

impl<'a> Iterator for InterfaceAddressIterator<'a> {
    type Item = InterfaceAddress<'a>;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match unsafe { self.next.as_ref() } {
            Some(ifaddr) => {
                self.next = ifaddr.ifa_next;
                Some(InterfaceAddress::from_libc_ifaddrs(ifaddr))
            }
            None => None,
        }
    }
}

/// Get interface addresses using libc's `getifaddrs`
///
/// Note that the underlying implementation differs between OSes. Only the
/// most common address families are supported by the nix crate (due to
/// lack of time and complexity of testing). The address family is encoded
/// in the specific variant of `SockaddrStorage` returned for the fields
/// `address`, `netmask`, `broadcast`, and `destination`. For any entry not
/// supported, the returned list will contain a `None` entry.
///
/// # Example
/// ```
/// let addrs = nix::ifaddrs::getifaddrs().unwrap();
/// for ifaddr in addrs.iter() {
///   match ifaddr.address {
///     Some(address) => {
///       println!("interface {} address {}",
///                ifaddr.interface_name, address);
///     },
///     None => {
///       println!("interface {} with unsupported address family",
///                ifaddr.interface_name);
///     }
///   }
/// }
/// ```
pub fn getifaddrs() -> Result<InterfaceAddresses> {
    let mut addrs = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();
    unsafe {
        Errno::result(libc::getifaddrs(addrs.as_mut_ptr())).map(|_| {
            InterfaceAddresses {
                base: addrs.assume_init(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::sys::socket::{AddressFamily, Ipv4Address};

    // Only checks if `getifaddrs` can be invoked without panicking.
    #[test]
    fn test_getifaddrs() {
        let _ = getifaddrs();
    }

    // Ensures getting the netmask works, and in particular that
    // `workaround_xnu_bug` works properly.
    #[test]
    fn test_getifaddrs_netmask_correct() {
        let addrs = getifaddrs().unwrap();
        for iface in addrs.iter() {
            let sock = if let Some(sock) = iface.netmask {
                sock
            } else {
                continue;
            };
            if sock.family() == AddressFamily::INET {
                let _ = sock.to_ipv4().unwrap();
                return;
            } else if sock.family() == AddressFamily::INET6 {
                let _ = sock.to_ipv6().unwrap();
                return;
            }
        }
        panic!("No address?");
    }

    #[test]
    fn test_get_ifaddrs_netmasks_eq() {
        let mut netmasks = HashMap::new();

        let ifs = getifaddrs().unwrap();

        for ifa in ifs.iter() {
            let Some(netmask) =
                ifa.netmask.filter(|n| n.family() == AddressFamily::INET)
            else {
                continue;
            };

            let ipv4 = *netmask.to_ipv4().unwrap();

            let [a, b, c, d] = ipv4.ip().to_be_bytes();

            let x = Ipv4Address::new(a, b, c, d, ipv4.port());

            assert_eq!(ipv4, x);

            netmasks.insert(
                ifa.interface_name.clone(),
                *netmask.to_ipv4().unwrap(),
            );
        }

        drop(ifs);

        let ifs = getifaddrs().unwrap();

        for ifa in ifs.iter() {
            let Some(netmask) =
                ifa.netmask.filter(|n| n.family() == AddressFamily::INET)
            else {
                continue;
            };

            assert_eq!(
                netmasks.get(&ifa.interface_name).unwrap(),
                netmask.to_ipv4().unwrap(),
            );
        }
    }
}
