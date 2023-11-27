//! Network interface name resolution.
//!
//! Uses Linux and/or POSIX functions to resolve interface names like "eth0"
//! or "socan1" into device numbers.

use std::fmt;
use crate::{Error, NixPath, Result};
use libc::c_uint;

/// Resolve an interface into a interface number.
pub fn if_nametoindex<P: ?Sized + NixPath>(name: &P) -> Result<c_uint> {
    let if_index = name
        .with_nix_path(|name| unsafe { libc::if_nametoindex(name.as_ptr()) })?;

    if if_index == 0 {
        Err(Error::last())
    } else {
        Ok(if_index)
    }
}

libc_bitflags!(
    /// Standard interface flags, used by `getifaddrs`
    pub struct InterfaceFlags: libc::c_int {
        /// Interface is running. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_UP;
        /// Valid broadcast address set. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_BROADCAST;
        /// Internal debugging flag. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        #[cfg(not(target_os = "haiku"))]
        IFF_DEBUG;
        /// Interface is a loopback interface. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_LOOPBACK;
        /// Interface is a point-to-point link. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_POINTOPOINT;
        /// Avoid use of trailers. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        #[cfg(any(
                  linux_android,
                  solarish,
                  apple_targets,
                  target_os = "fuchsia",
                  target_os = "netbsd"))]
        IFF_NOTRAILERS;
        /// Interface manages own routes.
        #[cfg(any(target_os = "dragonfly"))]
        IFF_SMART;
        /// Resources allocated. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        #[cfg(any(
                  linux_android,
                  bsd,
                  solarish,
                  target_os = "fuchsia"))]
        IFF_RUNNING;
        /// No arp protocol, L2 destination address not set. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_NOARP;
        /// Interface is in promiscuous mode. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_PROMISC;
        /// Receive all multicast packets. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_ALLMULTI;
        /// Master of a load balancing bundle. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        #[cfg(any(linux_android, target_os = "fuchsia"))]
        IFF_MASTER;
        /// transmission in progress, tx hardware queue is full
        #[cfg(any(target_os = "freebsd", apple_targets, netbsdlike))]
        IFF_OACTIVE;
        /// Protocol code on board.
        #[cfg(solarish)]
        IFF_INTELLIGENT;
        /// Slave of a load balancing bundle. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        #[cfg(any(linux_android, target_os = "fuchsia"))]
        IFF_SLAVE;
        /// Can't hear own transmissions.
        #[cfg(any(freebsdlike, netbsdlike, target_os = "macos"))]
        IFF_SIMPLEX;
        /// Supports multicast. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        IFF_MULTICAST;
        /// Per link layer defined bit.
        #[cfg(bsd)]
        IFF_LINK0;
        /// Multicast using broadcast.
        #[cfg(solarish)]
        IFF_MULTI_BCAST;
        /// Is able to select media type via ifmap. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        #[cfg(any(linux_android, target_os = "fuchsia"))]
        IFF_PORTSEL;
        /// Per link layer defined bit.
        #[cfg(bsd)]
        IFF_LINK1;
        /// Non-unique address.
        #[cfg(solarish)]
        IFF_UNNUMBERED;
        /// Auto media selection active. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        #[cfg(any(linux_android, target_os = "fuchsia"))]
        IFF_AUTOMEDIA;
        /// Per link layer defined bit.
        #[cfg(bsd)]
        IFF_LINK2;
        /// Use alternate physical connection.
        #[cfg(any(freebsdlike, apple_targets))]
        IFF_ALTPHYS;
        /// DHCP controls interface.
        #[cfg(solarish)]
        IFF_DHCPRUNNING;
        /// The addresses are lost when the interface goes down. (see
        /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html))
        #[cfg(any(linux_android, target_os = "fuchsia"))]
        IFF_DYNAMIC;
        /// Do not advertise.
        #[cfg(solarish)]
        IFF_PRIVATE;
        /// Driver signals L1 up. Volatile.
        #[cfg(any(target_os = "fuchsia", target_os = "linux"))]
        IFF_LOWER_UP;
        /// Interface is in polling mode.
        #[cfg(any(target_os = "dragonfly"))]
        IFF_POLLING_COMPAT;
        /// Unconfigurable using ioctl(2).
        #[cfg(any(target_os = "freebsd"))]
        IFF_CANTCONFIG;
        /// Do not transmit packets.
        #[cfg(solarish)]
        IFF_NOXMIT;
        /// Driver signals dormant. Volatile.
        #[cfg(any(target_os = "fuchsia", target_os = "linux"))]
        IFF_DORMANT;
        /// User-requested promisc mode.
        #[cfg(freebsdlike)]
        IFF_PPROMISC;
        /// Just on-link subnet.
        #[cfg(solarish)]
        IFF_NOLOCAL;
        /// Echo sent packets. Volatile.
        #[cfg(any(target_os = "fuchsia", target_os = "linux"))]
        IFF_ECHO;
        /// User-requested monitor mode.
        #[cfg(freebsdlike)]
        IFF_MONITOR;
        /// Address is deprecated.
        #[cfg(solarish)]
        IFF_DEPRECATED;
        /// Static ARP.
        #[cfg(freebsdlike)]
        IFF_STATICARP;
        /// Address from stateless addrconf.
        #[cfg(solarish)]
        IFF_ADDRCONF;
        /// Interface is in polling mode.
        #[cfg(any(target_os = "dragonfly"))]
        IFF_NPOLLING;
        /// Router on interface.
        #[cfg(solarish)]
        IFF_ROUTER;
        /// Interface is in polling mode.
        #[cfg(any(target_os = "dragonfly"))]
        IFF_IDIRECT;
        /// Interface is winding down
        #[cfg(any(target_os = "freebsd"))]
        IFF_DYING;
        /// No NUD on interface.
        #[cfg(solarish)]
        IFF_NONUD;
        /// Interface is being renamed
        #[cfg(any(target_os = "freebsd"))]
        IFF_RENAMING;
        /// Anycast address.
        #[cfg(solarish)]
        IFF_ANYCAST;
        /// Don't exchange routing info.
        #[cfg(solarish)]
        IFF_NORTEXCH;
        /// Do not provide packet information
        #[cfg(any(linux_android, target_os = "fuchsia"))]
        IFF_NO_PI as libc::c_int;
        /// TUN device (no Ethernet headers)
        #[cfg(any(linux_android, target_os = "fuchsia"))]
        IFF_TUN as libc::c_int;
        /// TAP device
        #[cfg(any(linux_android, target_os = "fuchsia"))]
        IFF_TAP as libc::c_int;
        /// IPv4 interface.
        #[cfg(solarish)]
        IFF_IPV4;
        /// IPv6 interface.
        #[cfg(solarish)]
        IFF_IPV6;
        /// in.mpathd test address
        #[cfg(solarish)]
        IFF_NOFAILOVER;
        /// Interface has failed
        #[cfg(solarish)]
        IFF_FAILED;
        /// Interface is a hot-spare
        #[cfg(solarish)]
        IFF_STANDBY;
        /// Functioning but not used
        #[cfg(solarish)]
        IFF_INACTIVE;
        /// Interface is offline
        #[cfg(solarish)]
        IFF_OFFLINE;
        #[cfg(target_os = "solaris")]
        IFF_COS_ENABLED;
        /// Prefer as source addr.
        #[cfg(target_os = "solaris")]
        IFF_PREFERRED;
        /// RFC3041
        #[cfg(target_os = "solaris")]
        IFF_TEMPORARY;
        /// MTU set with SIOCSLIFMTU
        #[cfg(target_os = "solaris")]
        IFF_FIXEDMTU;
        /// Cannot send / receive packets
        #[cfg(target_os = "solaris")]
        IFF_VIRTUAL;
        /// Local address in use
        #[cfg(target_os = "solaris")]
        IFF_DUPLICATE;
        /// IPMP IP interface
        #[cfg(target_os = "solaris")]
        IFF_IPMP;
    }
);

impl fmt::Display for InterfaceFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}


#[cfg(any(
    bsd,
    target_os = "fuchsia",
    target_os = "linux",
    target_os = "illumos",
))]
mod if_nameindex {
    use super::*;

    use std::ffi::CStr;
    use std::fmt;
    use std::marker::PhantomData;
    use std::ptr::NonNull;

    /// A network interface. Has a name like "eth0" or "wlp4s0" or "wlan0", as well as an index
    /// (1, 2, 3, etc) that identifies it in the OS's networking stack.
    #[allow(missing_copy_implementations)]
    #[repr(transparent)]
    pub struct Interface(libc::if_nameindex);

    impl Interface {
        /// Obtain the index of this interface.
        pub fn index(&self) -> c_uint {
            self.0.if_index
        }

        /// Obtain the name of this interface.
        pub fn name(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.0.if_name) }
        }
    }

    impl fmt::Debug for Interface {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Interface")
                .field("index", &self.index())
                .field("name", &self.name())
                .finish()
        }
    }

    /// A list of the network interfaces available on this system. Obtained from [`if_nameindex()`].
    #[repr(transparent)]
    pub struct Interfaces {
        ptr: NonNull<libc::if_nameindex>,
    }

    impl Interfaces {
        /// Iterate over the interfaces in this list.
        #[inline]
        pub fn iter(&self) -> InterfacesIter<'_> {
            self.into_iter()
        }

        /// Convert this to a slice of interfaces. Note that the underlying interfaces list is
        /// null-terminated, so calling this calculates the length. If random access isn't needed,
        /// [`Interfaces::iter()`] should be used instead.
        pub fn to_slice(&self) -> &[Interface] {
            let ifs = self.ptr.as_ptr().cast();
            let len = self.iter().count();
            unsafe { std::slice::from_raw_parts(ifs, len) }
        }
    }

    impl Drop for Interfaces {
        fn drop(&mut self) {
            unsafe { libc::if_freenameindex(self.ptr.as_ptr()) };
        }
    }

    impl fmt::Debug for Interfaces {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.to_slice().fmt(f)
        }
    }

    impl<'a> IntoIterator for &'a Interfaces {
        type IntoIter = InterfacesIter<'a>;
        type Item = &'a Interface;
        #[inline]
        fn into_iter(self) -> Self::IntoIter {
            InterfacesIter {
                ptr: self.ptr.as_ptr(),
                _marker: PhantomData,
            }
        }
    }

    /// An iterator over the interfaces in an [`Interfaces`].
    #[derive(Debug)]
    pub struct InterfacesIter<'a> {
        ptr: *const libc::if_nameindex,
        _marker: PhantomData<&'a Interfaces>,
    }

    impl<'a> Iterator for InterfacesIter<'a> {
        type Item = &'a Interface;
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            unsafe {
                if (*self.ptr).if_index == 0 {
                    None
                } else {
                    let ret = &*(self.ptr as *const Interface);
                    self.ptr = self.ptr.add(1);
                    Some(ret)
                }
            }
        }
    }

    /// Retrieve a list of the network interfaces available on the local system.
    ///
    /// ```
    /// let interfaces = nix::net::if_::if_nameindex().unwrap();
    /// for iface in &interfaces {
    ///     println!("Interface #{} is called {}", iface.index(), iface.name().to_string_lossy());
    /// }
    /// ```
    pub fn if_nameindex() -> Result<Interfaces> {
        unsafe {
            let ifs = libc::if_nameindex();
            let ptr = NonNull::new(ifs).ok_or_else(Error::last)?;
            Ok(Interfaces { ptr })
        }
    }
}
#[cfg(any(
    bsd,
    target_os = "fuchsia",
    target_os = "linux",
    target_os = "illumos",
))]
pub use if_nameindex::*;
