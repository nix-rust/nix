//! Query network interface addresses
//!
//! Uses the Linux and/or BSD specific function `getifaddrs` to query the list
//! of interfaces and their associated addresses.

use cfg_if::cfg_if;
use std::ffi;
use std::iter::Iterator;
use std::mem;
use std::option::Option;

use crate::net::if_::*;
use crate::sys::socket::SockAddr;
use crate::{Errno, Result};

/// Describes a single address for an interface as returned by `getifaddrs`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InterfaceAddress {
    /// Name of the network interface
    pub interface_name: String,
    /// Flags as from `SIOCGIFFLAGS` ioctl
    pub flags: InterfaceFlags,
    /// Network address of this interface
    pub address: Option<SockAddr>,
    /// Netmask of this interface
    pub netmask: Option<SockAddr>,
    /// Broadcast address of this interface, if applicable
    pub broadcast: Option<SockAddr>,
    /// Point-to-point destination address
    pub destination: Option<SockAddr>,
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

impl InterfaceAddress {
    /// Create an `InterfaceAddress` from the libc struct.
    fn from_libc_ifaddrs(info: &libc::ifaddrs) -> InterfaceAddress {
        let ifname = unsafe { ffi::CStr::from_ptr(info.ifa_name) };
        let get_sockaddr = |sa: *const libc::sockaddr| {
            if sa.is_null() {
                return None;
            }
            // TODO: there's gotta be a better way to do this but this is basically
            //       what the man pages recommend
            let len = match unsafe { (*sa).sa_family } as _ {
                libc::AF_INET => mem::size_of::<libc::sockaddr_in>(),
                libc::AF_INET6 => mem::size_of::<libc::sockaddr_in6>(),
                #[cfg(any(
                    target_os = "android",
                    target_os = "linux",
                    target_os = "illumos",
                    target_os = "fuchsia",
                    target_os = "solaris"
                ))]
                libc::AF_PACKET => mem::size_of::<libc::sockaddr_in>(),
                #[cfg(any(target_os = "android", target_os = "linux"))]
                libc::AF_NETLINK => mem::size_of::<libc::sockaddr_nl>(),
                _ => return None,
            };
            unsafe { SockAddr::from_raw_sockaddr(sa, len as libc::socklen_t) }.ok()
        };
        let address = get_sockaddr(info.ifa_addr);
        let netmask = get_sockaddr(info.ifa_netmask);
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
            addr.destination = get_sockaddr(ifu);
        } else if addr.flags.contains(InterfaceFlags::IFF_BROADCAST) {
            addr.broadcast = get_sockaddr(ifu);
        }

        addr
    }
}

/// Holds the results of `getifaddrs`.
///
/// Use the function `getifaddrs` to create this Iterator. Note that the
/// actual list of interfaces can be iterated once and will be freed as
/// soon as the Iterator goes out of scope.
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct InterfaceAddressIterator {
    base: *mut libc::ifaddrs,
    next: *mut libc::ifaddrs,
}

impl Drop for InterfaceAddressIterator {
    fn drop(&mut self) {
        unsafe { libc::freeifaddrs(self.base) };
    }
}

impl Iterator for InterfaceAddressIterator {
    type Item = InterfaceAddress;
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
/// in the specific variant of `SockAddr` returned for the fields `address`,
/// `netmask`, `broadcast`, and `destination`. For any entry not supported,
/// the returned list will contain a `None` entry.
///
/// # Example
/// ```
/// let addrs = nix::ifaddrs::getifaddrs().unwrap();
/// for ifaddr in addrs {
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
pub fn getifaddrs() -> Result<InterfaceAddressIterator> {
    let mut addrs = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();
    unsafe {
        Errno::result(libc::getifaddrs(addrs.as_mut_ptr())).map(|_| {
            let addrs = addrs.assume_init();
            InterfaceAddressIterator {
                base: addrs,
                next: addrs,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Only checks if `getifaddrs` can be invoked without panicking.
    #[test]
    fn test_getifaddrs() {
        let _ = getifaddrs();
    }
}
