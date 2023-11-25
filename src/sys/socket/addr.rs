#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    apple_targets,
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "haiku",
    target_os = "aix",
    target_os = "openbsd"
))]
#[cfg(feature = "net")]
pub use self::datalink::LinkAddr;
#[cfg(not(any(target_os = "solaris", target_os = "redox",)))]
#[cfg(feature = "net")]
pub use self::datalink::LinkAddress;
#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "macos"
))]
pub use self::vsock::VsockAddress;
use super::sa_family_t;
use crate::errno::Errno;
#[cfg(any(target_os = "android", target_os = "linux"))]
use crate::sys::socket::addr::alg::AlgAddress;
#[cfg(any(target_os = "android", target_os = "linux"))]
use crate::sys::socket::addr::netlink::NetlinkAddress;
#[cfg(all(
    feature = "ioctl",
    any(target_os = "ios", target_os = "macos")
))]
use crate::sys::socket::addr::sys_control::SysControlAddress;
use crate::{NixPath, Result};
use cfg_if::cfg_if;
use memoffset::offset_of;
use std::borrow::{Borrow, BorrowMut};
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr::addr_of;
#[allow(unused)]
use std::ptr::addr_of_mut;
use std::{fmt, mem, net, ptr, slice};

/// Convert a std::net::Ipv4Addr into the libc form.
#[cfg(feature = "net")]
pub(crate) const fn ipv4addr_to_libc(addr: net::Ipv4Addr) -> libc::in_addr {
    libc::in_addr {
        s_addr: u32::from_ne_bytes(addr.octets()),
    }
}

/// Convert a std::net::Ipv6Addr into the libc form.
#[cfg(feature = "net")]
pub(crate) const fn ipv6addr_to_libc(addr: &net::Ipv6Addr) -> libc::in6_addr {
    libc::in6_addr {
        s6_addr: addr.octets(),
    }
}

/// A possible error when converting a c_int to [`AddressFamily`].
#[derive(Debug, Clone, Copy)]
pub struct InvalidAddressFamilyError;

/// Address families, corresponding to `AF_*` constants in libc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AddressFamily(libc::c_int);

impl AddressFamily {
    /// Converts a c_int to an address family.
    ///
    /// The argument must fit into `sa_family_t`.
    pub const fn new(
        family: libc::c_int,
    ) -> std::result::Result<Self, InvalidAddressFamilyError> {
        if family > libc::sa_family_t::MAX as _ {
            return Err(InvalidAddressFamilyError);
        }

        Ok(Self(family))
    }

    /// Returns the c_int representation of the address family.
    pub const fn family(&self) -> libc::c_int {
        self.0
    }

    const fn of(addr: &libc::sockaddr) -> Self {
        Self(addr.sa_family as _)
    }
}

impl AddressFamily {
    /// Represents `AF_802`.
    #[cfg(any(target_os = "illumos", target_os = "solaris",))]
    pub const _802: Self = Self(libc::AF_802);
    /// Represents `AF_ALG`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const ALG: Self = Self(libc::AF_ALG);
    /// Represents `AF_APPLETALK`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const APPLETALK: Self = Self(libc::AF_APPLETALK);
    /// Represents `AF_ARP`.
    #[cfg(any(target_os = "freebsd", target_os = "netbsd",))]
    pub const ARP: Self = Self(libc::AF_ARP);
    /// Represents `AF_ASH`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const ASH: Self = Self(libc::AF_ASH);
    /// Represents `AF_ATM`.
    #[cfg(any(target_os = "dragonfly", target_os = "freebsd",))]
    pub const ATM: Self = Self(libc::AF_ATM);
    /// Represents `AF_ATMPVC`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const ATMPVC: Self = Self(libc::AF_ATMPVC);
    /// Represents `AF_ATMSVC`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const ATMSVC: Self = Self(libc::AF_ATMSVC);
    /// Represents `AF_AX25`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const AX25: Self = Self(libc::AF_AX25);
    /// Represents `AF_BLUETOOTH`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    pub const BLUETOOTH: Self = Self(libc::AF_BLUETOOTH);
    /// Represents `AF_BRIDGE`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const BRIDGE: Self = Self(libc::AF_BRIDGE);
    /// Represents `AF_CAIF`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const CAIF: Self = Self(libc::AF_CAIF);
    /// Represents `AF_CAN`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const CAN: Self = Self(libc::AF_CAN);
    /// Represents `AF_CCITT`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const CCITT: Self = Self(libc::AF_CCITT);
    /// Represents `AF_CHAOS`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const CHAOS: Self = Self(libc::AF_CHAOS);
    /// Represents `AF_CNT`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    pub const CNT: Self = Self(libc::AF_CNT);
    /// Represents `AF_COIP`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    pub const COIP: Self = Self(libc::AF_COIP);
    /// Represents `AF_DATAKIT`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const DATAKIT: Self = Self(libc::AF_DATAKIT);
    /// Represents `AF_DECnet`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const DEC_NET: Self = Self(libc::AF_DECnet);
    /// Represents `AF_DLI`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const DLI: Self = Self(libc::AF_DLI);
    /// Represents `AF_E164`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    pub const E164: Self = Self(libc::AF_E164);
    /// Represents `AF_ECMA`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const ECMA: Self = Self(libc::AF_ECMA);
    /// Represents `AF_ECONET`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const ECONET: Self = Self(libc::AF_ECONET);
    /// Represents `AF_ENCAP`.
    #[cfg(target_os = "openbsd")]
    pub const ENCAP: Self = Self(libc::AF_ENCAP);
    /// Represents `AF_FILE`.
    #[cfg(any(target_os = "illumos", target_os = "solaris",))]
    pub const FILE: Self = Self(libc::AF_FILE);
    /// Represents `AF_GOSIP`.
    #[cfg(any(target_os = "illumos", target_os = "solaris",))]
    pub const GOSIP: Self = Self(libc::AF_GOSIP);
    /// Represents `AF_HYLINK`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const HYLINK: Self = Self(libc::AF_HYLINK);
    /// Represents `AF_IB`.
    #[cfg(all(target_os = "linux", not(target_env = "uclibc")))]
    pub const IB: Self = Self(libc::AF_IB);
    /// Represents `AF_IEEE80211`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
    ))]
    pub const IEEE80211: Self = Self(libc::AF_IEEE80211);
    /// Represents `AF_IEEE802154`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const IEEE802154: Self = Self(libc::AF_IEEE802154);
    /// Represents `AF_IMPLINK`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const IMPLINK: Self = Self(libc::AF_IMPLINK);
    /// Represents `AF_INET`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "redox",
        target_os = "solaris",
    ))]
    pub const INET: Self = Self(libc::AF_INET);
    /// Represents `AF_INET6`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "redox",
        target_os = "solaris",
    ))]
    pub const INET6: Self = Self(libc::AF_INET6);
    /// Represents `AF_INET6_SDP`.
    #[cfg(target_os = "freebsd")]
    pub const INET6_SDP: Self = Self(libc::AF_INET6_SDP);
    /// Represents `AF_INET_OFFLOAD`.
    #[cfg(any(target_os = "illumos", target_os = "solaris",))]
    pub const INET_OFFLOAD: Self = Self(libc::AF_INET_OFFLOAD);
    /// Represents `AF_INET_SDP`.
    #[cfg(target_os = "freebsd")]
    pub const INET_SDP: Self = Self(libc::AF_INET_SDP);
    /// Represents `AF_IPX`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const IPX: Self = Self(libc::AF_IPX);
    /// Represents `AF_IRDA`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const IRDA: Self = Self(libc::AF_IRDA);
    /// Represents `AF_ISDN`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    pub const ISDN: Self = Self(libc::AF_ISDN);
    /// Represents `AF_ISO`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    pub const ISO: Self = Self(libc::AF_ISO);
    /// Represents `AF_IUCV`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const IUCV: Self = Self(libc::AF_IUCV);
    /// Represents `AF_KEY`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "linux",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const KEY: Self = Self(libc::AF_KEY);
    /// Represents `AF_LAT`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const LAT: Self = Self(libc::AF_LAT);
    /// Represents `AF_LINK`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const LINK: Self = Self(libc::AF_LINK);
    /// Represents `AF_LLC`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const LLC: Self = Self(libc::AF_LLC);
    /// Represents `AF_LOCAL`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const LOCAL: Self = Self(libc::AF_LOCAL);
    /// Represents `AF_MPLS`.
    #[cfg(all(
        any(
            target_os = "dragonfly",
            target_os = "linux",
            target_os = "netbsd",
            target_os = "openbsd",
        ),
        not(target_env = "uclibc"),
    ))]
    pub const MPLS: Self = Self(libc::AF_MPLS);
    /// Represents `AF_NATM`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    pub const NATM: Self = Self(libc::AF_NATM);
    /// Represents `AF_NBS`.
    #[cfg(any(target_os = "illumos", target_os = "solaris",))]
    pub const NBS: Self = Self(libc::AF_NBS);
    /// Represents `AF_NCA`.
    #[cfg(any(target_os = "illumos", target_os = "solaris",))]
    pub const NCA: Self = Self(libc::AF_NCA);
    /// Represents `AF_NDRV`.
    #[cfg(any(target_os = "ios", target_os = "macos",))]
    pub const NDRV: Self = Self(libc::AF_NDRV);
    /// Represents `AF_NETBEUI`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const NETBEUI: Self = Self(libc::AF_NETBEUI);
    /// Represents `AF_NETBIOS`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
    ))]
    pub const NETBIOS: Self = Self(libc::AF_NETBIOS);
    /// Represents `AF_NETGRAPH`.
    #[cfg(any(target_os = "dragonfly", target_os = "freebsd",))]
    pub const NETGRAPH: Self = Self(libc::AF_NETGRAPH);
    /// Represents `AF_NETLINK`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const NETLINK: Self = Self(libc::AF_NETLINK);
    /// Represents `AF_NETROM`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const NETROM: Self = Self(libc::AF_NETROM);
    /// Represents `AF_NFC`.
    #[cfg(any(target_os = "android", target_os = "linux",))]
    pub const NFC: Self = Self(libc::AF_NFC);
    /// Represents `AF_NIT`.
    #[cfg(any(target_os = "illumos", target_os = "solaris",))]
    pub const NIT: Self = Self(libc::AF_NIT);
    /// Represents `AF_NOTIFY`.
    #[cfg(target_os = "haiku")]
    pub const NOTIFY: Self = Self(libc::AF_NOTIFY);
    /// Represents `AF_NS`.
    #[cfg(any(
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const NS: Self = Self(libc::AF_NS);
    /// Represents `AF_OROUTE`.
    #[cfg(target_os = "netbsd")]
    pub const OROUTE: Self = Self(libc::AF_OROUTE);
    /// Represents `AF_OSI`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const OSI: Self = Self(libc::AF_OSI);
    /// Represents `AF_OSINET`.
    #[cfg(any(target_os = "illumos", target_os = "solaris",))]
    pub const OSINET: Self = Self(libc::AF_OSINET);
    /// Represents `AF_PACKET`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "linux",
        target_os = "solaris",
    ))]
    pub const PACKET: Self = Self(libc::AF_PACKET);
    /// Represents `AF_PHONET`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const PHONET: Self = Self(libc::AF_PHONET);
    /// Represents `AF_POLICY`.
    #[cfg(any(target_os = "illumos", target_os = "solaris",))]
    pub const POLICY: Self = Self(libc::AF_POLICY);
    /// Represents `AF_PPP`.
    #[cfg(any(target_os = "ios", target_os = "macos",))]
    pub const PPP: Self = Self(libc::AF_PPP);
    /// Represents `AF_PPPOX`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const PPPOX: Self = Self(libc::AF_PPPOX);
    /// Represents `AF_PUP`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const PUP: Self = Self(libc::AF_PUP);
    /// Represents `AF_RDS`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const RDS: Self = Self(libc::AF_RDS);
    /// Represents `AF_ROSE`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const ROSE: Self = Self(libc::AF_ROSE);
    /// Represents `AF_ROUTE`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const ROUTE: Self = Self(libc::AF_ROUTE);
    /// Represents `AF_RXRPC`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const RXRPC: Self = Self(libc::AF_RXRPC);
    /// Represents `AF_SCLUSTER`.
    #[cfg(target_os = "freebsd")]
    pub const SCLUSTER: Self = Self(libc::AF_SCLUSTER);
    /// Represents `AF_SECURITY`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const SECURITY: Self = Self(libc::AF_SECURITY);
    /// Represents `AF_SIP`.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "openbsd",
    ))]
    pub const SIP: Self = Self(libc::AF_SIP);
    /// Represents `AF_SLOW`.
    #[cfg(target_os = "freebsd")]
    pub const SLOW: Self = Self(libc::AF_SLOW);
    /// Represents `AF_SNA`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "solaris",
    ))]
    pub const SNA: Self = Self(libc::AF_SNA);
    /// Represents `AF_SYSTEM`.
    #[cfg(any(target_os = "ios", target_os = "macos",))]
    pub const SYSTEM: Self = Self(libc::AF_SYSTEM);
    /// Represents `AF_SYS_CONTROL`.
    #[cfg(any(target_os = "ios", target_os = "macos",))]
    pub const SYS_CONTROL: Self = Self(libc::AF_SYS_CONTROL);
    /// Represents `AF_TIPC`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const TIPC: Self = Self(libc::AF_TIPC);
    /// Represents `AF_TRILL`.
    #[cfg(any(target_os = "illumos", target_os = "solaris",))]
    pub const TRILL: Self = Self(libc::AF_TRILL);
    /// Represents `AF_UNIX`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "redox",
        target_os = "solaris",
    ))]
    pub const UNIX: Self = Self(libc::AF_UNIX);
    /// Represents `AF_UNSPEC`.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "redox",
        target_os = "solaris",
    ))]
    pub const UNSPEC: Self = Self(libc::AF_UNSPEC);
    /// Represents `AF_UTUN`.
    #[cfg(any(target_os = "ios", target_os = "macos",))]
    pub const UTUN: Self = Self(libc::AF_UTUN);
    /// Represents `AF_VSOCK`.
    #[cfg(any(
        target_os = "android",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
    ))]
    pub const VSOCK: Self = Self(libc::AF_VSOCK);
    /// Represents `AF_WANPIPE`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "linux",
    ))]
    pub const WANPIPE: Self = Self(libc::AF_WANPIPE);
    /// Represents `AF_X25`.
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "linux",
        target_os = "solaris",
    ))]
    pub const X25: Self = Self(libc::AF_X25);
    /// Represents `AF_XDP`.
    #[cfg(all(target_os = "linux", not(target_env = "uclibc")))]
    pub const XDP: Self = Self(libc::AF_XDP);
}

macro_rules! sockaddr_family {
    () => {
        #[doc = "Returns the address family of this socket address."]
        pub const fn family(&self) -> AddressFamily {
            unsafe { AddressFamily::of(&*self.as_ptr().cast()) }
        }
    };
}

macro_rules! sockaddr_len_static {
    () => {
        #[doc="Returns the length of this socket address.\n\nThe returned length is guaranteed
        to be in bounds of `[2, SIZE]` with `SIZE` being the size of the underlying libc type."]
        #[allow(clippy::len_without_is_empty)]
        pub const fn len(&self) -> usize {
            mem::size_of::<Self>()
        }
    };
}

/// UNIX socket address.
///
/// Dyn-sized variant of [`UnixAddress`].
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(not(doc), repr(C, align(2)))]
pub struct UnixAddr {
    slice: [u8],
}

#[allow(clippy::len_without_is_empty)]
impl UnixAddr {
    const _ALIGN: () = assert!(mem::align_of::<libc::sockaddr_un>() == 2);

    // SAFETY: When dereferencing, `data` must be valid for reads of at least `len` bytes
    // and must point to a valid UNIX address.
    #[allow(clippy::cast_ptr_alignment)]
    const fn from_raw_parts(
        data: *const libc::sockaddr_un,
        len: usize,
    ) -> *const Self {
        let data_ptr = data.cast::<u8>();
        let slice_ptr = ptr::slice_from_raw_parts(data_ptr, len);

        slice_ptr as *const Self
    }

    // SAFETY: When dereferencing, `data` must be valid for reads and writes of at least `len` bytes
    // and must point to a valid UNIX address.
    #[allow(clippy::cast_ptr_alignment)]
    fn from_raw_parts_mut(
        data: *mut libc::sockaddr_un,
        len: usize,
    ) -> *mut Self {
        let data_ptr = data.cast::<u8>();
        let slice_ptr = ptr::slice_from_raw_parts_mut(data_ptr, len);

        slice_ptr as *mut Self
    }

    /// Returns the length of this socket address.
    ///
    /// The returned length is guaranteed to be in bounds of `[2, MAX]` with `MAX` being the
    /// maximum length of a socket address (i.e. the size of [`libc::sockaddr_storage`]).
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    fn kind(&self) -> UnixAddrKind<'_> {
        // SAFETY: our sockaddr is always valid because of the invariant on the struct
        unsafe { UnixAddrKind::get(&*self.as_ptr(), self.len() as _) }
    }

    /// If this address represents a filesystem path, return that path.
    pub fn path(&self) -> Option<&Path> {
        match self.kind() {
            UnixAddrKind::Pathname(path) => Some(path),
            _ => None,
        }
    }

    /// If this address represents an abstract socket, return its name.
    ///
    /// For abstract sockets only the bare name is returned, without the
    /// leading NUL byte. `None` is returned for unnamed or path-backed sockets.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[cfg_attr(docsrs, doc(cfg(all())))]
    pub fn as_abstract(&self) -> Option<&[u8]> {
        match self.kind() {
            UnixAddrKind::Abstract(name) => Some(name),
            _ => None,
        }
    }

    /// Check if this address is an "unnamed" UNIX socket address.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[cfg_attr(docsrs, doc(cfg(all())))]
    #[inline]
    pub fn is_unnamed(&self) -> bool {
        matches!(self.kind(), UnixAddrKind::Unnamed)
    }

    /// Returns the length of the path of this address.
    #[inline]
    pub fn path_len(&self) -> usize {
        self.len() - offset_of!(libc::sockaddr_un, sun_path)
    }

    /// Converts to a boxed [`UnixAddr`].
    ///
    /// # Panics
    ///
    /// This method panics if the allocation fails.
    pub fn to_boxed(&self) -> Box<Self> {
        let layout = std::alloc::Layout::for_value(self);

        unsafe {
            let box_ptr = std::alloc::alloc(layout);
            assert_ne!(box_ptr, std::ptr::null_mut());

            ptr::copy_nonoverlapping(self.as_ptr().cast(), box_ptr, self.len());

            Box::from_raw(Self::from_raw_parts_mut(box_ptr.cast(), self.len()))
        }
    }

    /// Converts to an owned [`UnixAddress`] if the address fits into it.
    ///
    /// If you are unsure whether it fits into [`UnixAddress`], consider using
    /// [`Self::to_boxed`] instead.
    pub fn to_owned(&self) -> Option<UnixAddress> {
        if self.len() > mem::size_of::<libc::sockaddr_un>() {
            return None;
        }

        let mut addr = mem::MaybeUninit::<libc::sockaddr_un>::zeroed();

        unsafe {
            ptr::copy_nonoverlapping(
                self.slice.as_ptr(),
                addr.as_mut_ptr().cast(),
                self.len(),
            );
        }

        Some(UnixAddress {
            sun: unsafe { addr.assume_init() },
            #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "redox",
            ))]
            sun_len: self.len() as _,
        })
    }

    /// Returns a raw pointer to the libc type.
    ///
    /// # Safety
    ///
    /// Keep in mind that it is in general *not* safe to dereference
    /// this pointer, as the length of this address can be smaller than
    /// the size of the libc type, leading to uninitialized fields.
    ///
    /// It is preferable to access individual fieds using [`addr_of!`].
    pub const fn as_ptr(&self) -> *const libc::sockaddr_un {
        self.slice.as_ptr().cast()
    }

    /// Returns a raw mutable pointer to the libc type.
    ///
    /// # Safety
    ///
    /// Keep in mind that it is in general *not* safe to dereference
    /// this pointer, as the length of this address can be smaller than
    /// the size of the libc type, leading to uninitialized fields.
    ///
    /// It is preferable to access individual fieds using [`addr_of_mut!`].
    pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_un {
        self.slice.as_mut_ptr().cast()
    }
}

impl PartialEq<UnixAddress> for UnixAddr {
    fn eq(&self, other: &UnixAddress) -> bool {
        *self == **other
    }
}

impl fmt::Display for UnixAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind() {
            UnixAddrKind::Pathname(path) => path.display().fmt(f),
            UnixAddrKind::Unnamed => f.pad("<unbound UNIX socket>"),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            UnixAddrKind::Abstract(name) => fmt_abstract(name, f),
        }
    }
}

impl AsRef<Addr> for UnixAddr {
    fn as_ref(&self) -> &Addr {
        unsafe { &*Addr::from_raw_parts(self.as_ptr().cast(), self.len()) }
    }
}

impl Borrow<Addr> for UnixAddr {
    fn borrow(&self) -> &Addr {
        self.as_ref()
    }
}

/// UNIX socket address.
///
/// This struct is an abstraction for socket addresses of type `AF_UNIX`.
#[derive(Clone, Copy, Debug, Eq)]
#[repr(C)]
pub struct UnixAddress {
    // INVARIANT: sun & sun_len are valid as defined by docs for from_raw_parts
    sun: libc::sockaddr_un,
    /// The length of the valid part of `sun`, including the sun_family field
    /// but excluding any trailing nul.
    // On the BSDs, this field is built into sun
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "linux",
        target_os = "redox",
    ))]
    sun_len: usize,
}

// linux man page unix(7) says there are 3 kinds of unix socket:
// pathname: addrlen = offsetof(struct sockaddr_un, sun_path) + strlen(sun_path) + 1
// unnamed: addrlen = sizeof(sa_family_t)
// abstract: addren > sizeof(sa_family_t), name = sun_path[..(addrlen - sizeof(sa_family_t))]
//
// what we call path_len = addrlen - offsetof(struct sockaddr_un, sun_path)
#[derive(PartialEq, Eq, Hash)]
enum UnixAddrKind<'a> {
    Pathname(&'a Path),
    Unnamed,
    #[cfg(any(target_os = "android", target_os = "linux"))]
    Abstract(&'a [u8]),
}
impl<'a> UnixAddrKind<'a> {
    /// Safety: sun & sun_len must be valid
    #[allow(clippy::unnecessary_cast)] // Not unnecessary on all platforms
    unsafe fn get(sun: &'a libc::sockaddr_un, sun_len: u8) -> Self {
        assert!(sun_len as usize >= offset_of!(libc::sockaddr_un, sun_path));
        let path_len =
            sun_len as usize - offset_of!(libc::sockaddr_un, sun_path);
        if path_len == 0 {
            return Self::Unnamed;
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        if sun.sun_path[0] == 0 {
            let name = unsafe {
                slice::from_raw_parts(
                    sun.sun_path.as_ptr().add(1).cast(),
                    path_len - 1,
                )
            };
            return Self::Abstract(name);
        }
        let pathname = unsafe {
            slice::from_raw_parts(sun.sun_path.as_ptr().cast(), path_len)
        };
        if pathname.last() == Some(&0) {
            // A trailing NUL is not considered part of the path, and it does
            // not need to be included in the addrlen passed to functions like
            // bind().  However, Linux adds a trailing NUL, even if one was not
            // originally present, when returning addrs from functions like
            // getsockname() (the BSDs do not do that).  So we need to filter
            // out any trailing NUL here, so sockaddrs can round-trip through
            // the kernel and still compare equal.
            Self::Pathname(Path::new(OsStr::from_bytes(
                &pathname[0..pathname.len() - 1],
            )))
        } else {
            Self::Pathname(Path::new(OsStr::from_bytes(pathname)))
        }
    }
}

impl UnixAddress {
    /// Create a new UNIX address representing a filesystem path.
    #[allow(clippy::unnecessary_cast)] // Not unnecessary on all platforms
    pub fn new<P: ?Sized + NixPath>(path: &P) -> Result<UnixAddress> {
        path.with_nix_path(|cstr| unsafe {
            let mut ret = libc::sockaddr_un {
                sun_family: libc::AF_UNIX as sa_family_t,
                ..mem::zeroed()
            };

            let bytes = cstr.to_bytes();

            if bytes.len() >= ret.sun_path.len() {
                return Err(Errno::ENAMETOOLONG);
            }

            // We add 1 for the trailing NUL
            let sun_len =
                bytes.len() + offset_of!(libc::sockaddr_un, sun_path) + 1;

            #[cfg(not(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "redox",
            )))]
            {
                ret.sun_len = sun_len as _;
            }
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                ret.sun_path.as_mut_ptr().cast(),
                bytes.len(),
            );

            Ok(UnixAddress::from_raw_parts(ret, sun_len))
        })?
    }

    /// Create a new UNIX address representing an address in the "abstract namespace".
    ///
    /// The leading nul byte for the abstract namespace is automatically added;
    /// thus the input `path` is expected to be the bare name, not NUL-prefixed.
    /// This is a Linux-specific extension, primarily used to allow chrooted
    /// processes to communicate with processes having a different filesystem view.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[allow(clippy::unnecessary_cast)] // Not unnecessary on all platforms
    pub fn new_abstract(path: &[u8]) -> Result<UnixAddress> {
        unsafe {
            let mut ret = libc::sockaddr_un {
                sun_family: libc::AF_UNIX as sa_family_t,
                ..mem::zeroed()
            };

            if path.len() >= ret.sun_path.len() {
                return Err(Errno::ENAMETOOLONG);
            }

            // We add 1 for the leading NUL
            let sun_len =
                path.len() + 1 + offset_of!(libc::sockaddr_un, sun_path);

            #[cfg(not(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "redox",
            )))]
            {
                ret.sun_len = sun_len as _;
            }

            // Abstract addresses are represented by sun_path[0] ==
            // b'\0', so copy starting one byte in.
            ptr::copy_nonoverlapping(
                path.as_ptr(),
                ret.sun_path.as_mut_ptr().offset(1).cast(),
                path.len(),
            );

            Ok(UnixAddress::from_raw_parts(ret, sun_len))
        }
    }

    /// Create an unnamed UNIX address.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    pub fn new_unnamed() -> UnixAddress {
        let ret = libc::sockaddr_un {
            sun_family: libc::AF_UNIX as sa_family_t,
            ..unsafe { mem::zeroed() }
        };

        let sun_len = offset_of!(libc::sockaddr_un, sun_path);

        #[cfg(not(any(
            target_os = "android",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "linux",
            target_os = "redox",
        )))]
        {
            ret.sun_len = sun_len as _;
        }

        unsafe { UnixAddress::from_raw_parts(ret, sun_len) }
    }

    sockaddr_family!();

    /// Returns the length of this socket address.
    ///
    /// The returned length is guaranteed to be in bounds of `[2, SIZE]` with `SIZE` being the
    /// size of the underlying libc type.
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        cfg_if! {
            if #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "redox",
            ))]
            {
                self.sun_len
            } else {
                self.sun.sun_len as _
            }
        }
    }

    /// Returns a raw pointer to the libc type.
    pub const fn as_ptr(&self) -> *const libc::sockaddr_un {
        &self.sun
    }

    /// Returns a raw mutable pointer to the libc type.
    pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_un {
        &mut self.sun
    }

    /// Create a UnixAddr from a raw `sockaddr_un` struct and a size. `sun_len`
    /// is the size of the valid portion of the struct, excluding any trailing
    /// NUL.
    ///
    /// # Safety
    /// This pair of sockaddr_un & sun_len must be a valid unix addr, which
    /// means:
    /// - sun_len >= offset_of(sockaddr_un, sun_path)
    /// - sun_len <= sockaddr_un.sun_path.len() - offset_of(sockaddr_un, sun_path)
    /// - if this is a unix addr with a pathname, sun.sun_path is a
    ///   fs path, not necessarily nul-terminated.
    pub(crate) unsafe fn from_raw_parts(
        sun: libc::sockaddr_un,
        sun_len: usize,
    ) -> UnixAddress {
        cfg_if! {
            if #[cfg(any(target_os = "android",
                     target_os = "fuchsia",
                     target_os = "illumos",
                     target_os = "linux",
                     target_os = "redox",
                ))]
            {
                UnixAddress { sun, sun_len }
            } else {
                assert_eq!(sun_len, sun.sun_len as usize);
                UnixAddress {sun}
            }
        }
    }
}

impl std::ops::Deref for UnixAddress {
    type Target = UnixAddr;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The length cannot exceed the size of the struct.
        unsafe { &*UnixAddr::from_raw_parts(&self.sun, self.len()) }
    }
}

impl std::ops::DerefMut for UnixAddress {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.len();

        // SAFETY: The length cannot exceed the size of the struct.
        unsafe { &mut *UnixAddr::from_raw_parts_mut(&mut self.sun, len) }
    }
}

impl AsRef<Addr> for UnixAddress {
    fn as_ref(&self) -> &Addr {
        (**self).as_ref()
    }
}

impl Borrow<Addr> for UnixAddress {
    fn borrow(&self) -> &Addr {
        (**self).borrow()
    }
}

#[allow(clippy::cast_ptr_alignment)]
impl From<UnixAddress> for Address {
    fn from(addr: UnixAddress) -> Self {
        let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();

        unsafe {
            ptr::copy_nonoverlapping(
                &addr.sun as *const _ as *const u8,
                storage.as_mut_ptr().cast(),
                addr.len(),
            );

            Address {
                storage: storage.assume_init(),
                #[cfg(any(
                    target_os = "android",
                    target_os = "fuchsia",
                    target_os = "illumos",
                    target_os = "linux",
                    target_os = "redox",
                ))]
                len: addr.len() as _,
            }
        }
    }
}

impl AsRef<UnixAddr> for UnixAddress {
    fn as_ref(&self) -> &UnixAddr {
        self
    }
}

impl AsMut<UnixAddr> for UnixAddress {
    fn as_mut(&mut self) -> &mut UnixAddr {
        self
    }
}

impl Borrow<UnixAddr> for UnixAddress {
    fn borrow(&self) -> &UnixAddr {
        self
    }
}

impl BorrowMut<UnixAddr> for UnixAddress {
    fn borrow_mut(&mut self) -> &mut UnixAddr {
        self
    }
}

impl PartialEq<UnixAddress> for UnixAddress {
    fn eq(&self, other: &UnixAddress) -> bool {
        **self == **other
    }
}

impl PartialEq<UnixAddr> for UnixAddress {
    fn eq(&self, other: &UnixAddr) -> bool {
        **self == *other
    }
}

impl Hash for UnixAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl std::fmt::Display for UnixAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl AsRef<libc::sockaddr_un> for UnixAddress {
    fn as_ref(&self) -> &libc::sockaddr_un {
        &self.sun
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
fn fmt_abstract(abs: &[u8], f: &mut fmt::Formatter) -> fmt::Result {
    use fmt::Write;
    f.write_str("@\"")?;
    for &b in abs {
        use fmt::Display;
        char::from(b).escape_default().fmt(f)?;
    }
    f.write_char('"')?;
    Ok(())
}

/// An IPv4 socket address
#[cfg(feature = "net")]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Ipv4Address(libc::sockaddr_in);

#[cfg(feature = "net")]
impl Ipv4Address {
    /// Returns the IP address associated with this socket address, in native
    /// endian.
    pub const fn ip(&self) -> libc::in_addr_t {
        u32::from_be(self.0.sin_addr.s_addr)
    }

    /// Creates a new socket address from IPv4 octets and a port number.
    pub fn new(a: u8, b: u8, c: u8, d: u8, port: u16) -> Self {
        Self(libc::sockaddr_in {
            #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "aix",
                target_os = "haiku",
                target_os = "openbsd"
            ))]
            sin_len: mem::size_of::<libc::sockaddr_in>() as u8,
            sin_family: libc::AF_INET as sa_family_t,
            sin_port: u16::to_be(port),
            sin_addr: libc::in_addr {
                s_addr: u32::from_ne_bytes([a, b, c, d]),
            },
            sin_zero: unsafe { mem::zeroed() },
        })
    }

    sockaddr_family!();
    sockaddr_len_static!();

    /// Returns the port number associated with this socket address, in native
    /// endian.
    pub const fn port(&self) -> u16 {
        u16::from_be(self.0.sin_port)
    }

    /// Returns a raw pointer to the libc type.
    pub const fn as_ptr(&self) -> *const libc::sockaddr_in {
        &self.0
    }

    /// Returns a raw mutable pointer to the libc type.
    pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_in {
        &mut self.0
    }
}

#[cfg(feature = "net")]
#[allow(clippy::cast_ptr_alignment)]
impl AsRef<Addr> for Ipv4Address {
    fn as_ref(&self) -> &Addr {
        unsafe {
            &*Addr::from_raw_parts(&self.0 as *const _ as *const _, self.len())
        }
    }
}

#[cfg(feature = "net")]
impl Borrow<Addr> for Ipv4Address {
    fn borrow(&self) -> &Addr {
        self.as_ref()
    }
}

#[cfg(feature = "net")]
#[allow(clippy::cast_ptr_alignment)]
impl From<Ipv4Address> for Address {
    fn from(addr: Ipv4Address) -> Self {
        let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();

        unsafe {
            ptr::copy_nonoverlapping(
                &addr.0 as *const _ as *const u8,
                storage.as_mut_ptr().cast(),
                addr.len(),
            );

            Address {
                storage: storage.assume_init(),
                #[cfg(any(
                    target_os = "android",
                    target_os = "fuchsia",
                    target_os = "illumos",
                    target_os = "linux",
                    target_os = "redox",
                ))]
                len: addr.len() as _,
            }
        }
    }
}

#[cfg(feature = "net")]
impl AsRef<libc::sockaddr_in> for Ipv4Address {
    fn as_ref(&self) -> &libc::sockaddr_in {
        &self.0
    }
}

#[cfg(feature = "net")]
impl fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ne = u32::from_be(self.0.sin_addr.s_addr);
        let port = u16::from_be(self.0.sin_port);
        write!(
            f,
            "{}.{}.{}.{}:{}",
            ne >> 24,
            (ne >> 16) & 0xFF,
            (ne >> 8) & 0xFF,
            ne & 0xFF,
            port
        )
    }
}

#[cfg(feature = "net")]
impl From<net::SocketAddrV4> for Ipv4Address {
    fn from(addr: net::SocketAddrV4) -> Self {
        Self(libc::sockaddr_in {
            #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "haiku",
                target_os = "hermit",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            sin_len: mem::size_of::<libc::sockaddr_in>() as u8,
            sin_family: libc::AF_INET as sa_family_t,
            sin_port: addr.port().to_be(), // network byte order
            sin_addr: ipv4addr_to_libc(*addr.ip()),
            ..unsafe { mem::zeroed() }
        })
    }
}

#[cfg(feature = "net")]
impl From<Ipv4Address> for net::SocketAddrV4 {
    fn from(addr: Ipv4Address) -> Self {
        net::SocketAddrV4::new(
            net::Ipv4Addr::from(addr.0.sin_addr.s_addr.to_ne_bytes()),
            u16::from_be(addr.0.sin_port),
        )
    }
}

#[cfg(feature = "net")]
impl std::str::FromStr for Ipv4Address {
    type Err = net::AddrParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        net::SocketAddrV4::from_str(s).map(Ipv4Address::from)
    }
}

/// An IPv6 socket address
#[cfg(feature = "net")]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Ipv6Address(libc::sockaddr_in6);

#[cfg(feature = "net")]
impl Ipv6Address {
    sockaddr_family!();
    sockaddr_len_static!();

    /// Returns the flow information associated with this address.
    pub const fn flowinfo(&self) -> u32 {
        self.0.sin6_flowinfo
    }

    /// Returns the IP address associated with this socket address.
    pub fn ip(&self) -> net::Ipv6Addr {
        net::Ipv6Addr::from(self.0.sin6_addr.s6_addr)
    }

    /// Returns the port number associated with this socket address, in native
    /// endian.
    pub const fn port(&self) -> u16 {
        u16::from_be(self.0.sin6_port)
    }

    /// Returns the scope ID associated with this address.
    pub const fn scope_id(&self) -> u32 {
        self.0.sin6_scope_id
    }

    /// Returns a raw pointer to the libc type.
    pub const fn as_ptr(&self) -> *const libc::sockaddr_in6 {
        &self.0
    }

    /// Returns a raw mutable pointer to the libc type.
    pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_in6 {
        &mut self.0
    }
}

#[cfg(feature = "net")]
#[allow(clippy::cast_ptr_alignment)]
impl AsRef<Addr> for Ipv6Address {
    fn as_ref(&self) -> &Addr {
        unsafe {
            &*Addr::from_raw_parts(&self.0 as *const _ as *const _, self.len())
        }
    }
}

#[cfg(feature = "net")]
impl Borrow<Addr> for Ipv6Address {
    fn borrow(&self) -> &Addr {
        self.as_ref()
    }
}

#[cfg(feature = "net")]
#[cfg(feature = "net")]
#[allow(clippy::cast_ptr_alignment)]
impl From<Ipv6Address> for Address {
    fn from(addr: Ipv6Address) -> Self {
        let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();

        unsafe {
            ptr::copy_nonoverlapping(
                &addr.0 as *const _ as *const u8,
                storage.as_mut_ptr().cast(),
                addr.len(),
            );

            Address {
                storage: storage.assume_init(),
                #[cfg(any(
                    target_os = "android",
                    target_os = "fuchsia",
                    target_os = "illumos",
                    target_os = "linux",
                    target_os = "redox",
                ))]
                len: addr.len() as _,
            }
        }
    }
}

#[cfg(feature = "net")]
impl AsRef<libc::sockaddr_in6> for Ipv6Address {
    fn as_ref(&self) -> &libc::sockaddr_in6 {
        &self.0
    }
}

#[cfg(feature = "net")]
impl fmt::Display for Ipv6Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // These things are really hard to display properly.  Easier to let std
        // do it.
        let std = net::SocketAddrV6::new(
            self.ip(),
            self.port(),
            self.flowinfo(),
            self.scope_id(),
        );
        std.fmt(f)
    }
}

#[cfg(feature = "net")]
impl From<net::SocketAddrV6> for Ipv6Address {
    fn from(addr: net::SocketAddrV6) -> Self {
        #[allow(clippy::needless_update)] // It isn't needless on Illumos
        Self(libc::sockaddr_in6 {
            #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "haiku",
                target_os = "hermit",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            sin6_len: mem::size_of::<libc::sockaddr_in6>() as u8,
            sin6_family: libc::AF_INET6 as sa_family_t,
            sin6_port: addr.port().to_be(), // network byte order
            sin6_addr: ipv6addr_to_libc(addr.ip()),
            sin6_flowinfo: addr.flowinfo(), // host byte order
            sin6_scope_id: addr.scope_id(), // host byte order
            ..unsafe { mem::zeroed() }
        })
    }
}

#[cfg(feature = "net")]
impl From<Ipv6Address> for net::SocketAddrV6 {
    fn from(addr: Ipv6Address) -> Self {
        net::SocketAddrV6::new(
            net::Ipv6Addr::from(addr.0.sin6_addr.s6_addr),
            u16::from_be(addr.0.sin6_port),
            addr.0.sin6_flowinfo,
            addr.0.sin6_scope_id,
        )
    }
}

#[cfg(feature = "net")]
impl std::str::FromStr for Ipv6Address {
    type Err = net::AddrParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        net::SocketAddrV6::from_str(s).map(Ipv6Address::from)
    }
}

macro_rules! sockaddr_storage_conv {
    ($fname:ident, $nixty:ty, $cty:ident, $af:ident, $doc:tt) => {
        #[doc = $doc]
        pub fn $fname(&self) -> Option<&$nixty> {
            if self.family() != AddressFamily::$af {
                return None;
            }

            let addr = self.as_ptr().cast();

            unsafe { Some(&*addr) }
        }
    };
}

/// General purpose socket address with arbitrary address family.
///
/// Dyn-sized variant of [`Address`].
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(not(doc), repr(C, align(2)))]
pub struct Addr {
    slice: [u8],
}

#[allow(clippy::len_without_is_empty)]
impl Addr {
    const _ALIGN: () = assert!(mem::align_of::<libc::sockaddr_storage>() == 2);

    // SAFETY: When dereferencing, `data` must be valid for reads of at least `len` bytes.
    #[allow(clippy::cast_ptr_alignment)]
    const fn from_raw_parts(
        data: *const libc::sockaddr_storage,
        len: usize,
    ) -> *const Self {
        let data_ptr = data.cast::<u8>();
        let slice_ptr = ptr::slice_from_raw_parts(data_ptr, len);

        slice_ptr as *const Self
    }

    // SAFETY: When dereferencing, `data` must be valid for reads and writes of at least `len` bytes.
    #[allow(clippy::cast_ptr_alignment)]
    fn from_raw_parts_mut(
        data: *mut libc::sockaddr_storage,
        len: usize,
    ) -> *mut Self {
        let data_ptr = data.cast::<u8>();
        let slice_ptr = ptr::slice_from_raw_parts_mut(data_ptr, len);

        slice_ptr as *mut Self
    }

    /// Returns an empty socket address.
    pub const fn empty() -> &'static Self {
        let p = ptr::NonNull::<libc::sockaddr_storage>::dangling()
            .as_ptr()
            .cast_const();

        unsafe { &*Self::from_raw_parts(p, 0) }
    }

    /// Returns the length of this socket address.
    ///
    /// The returned length is guaranteed to be in bounds of `[2, MAX]` with `MAX` being the
    /// maximum length of a socket address (i.e. the size of [`libc::sockaddr_storage`]).
    ///
    /// The length can also be zero, in which case the address hasn't been initialized yet.
    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    /// Returns the address family associated with this socket address.
    ///
    /// If the length of this address is zero, [`AddressFamily::UNSPEC`] is returned.
    #[allow(clippy::cast_ptr_alignment)]
    pub const fn family(&self) -> AddressFamily {
        if self.len() == 0 {
            return AddressFamily::UNSPEC;
        }

        unsafe { AddressFamily(*addr_of!((*self.as_ptr()).ss_family) as _) }
    }

    #[cfg(any(target_os = "android", target_os = "linux"))]
    sockaddr_storage_conv!(
        to_alg,
        AlgAddress,
        sockaddr_alg,
        ALG,
        "Converts to [`AlgAddress`], if the address family matches."
    );

    #[cfg(feature = "net")]
    sockaddr_storage_conv!(
        to_ipv4,
        Ipv4Address,
        sockaddr_in,
        INET,
        "Converts to [`Ipv4Address`], if the address family matches."
    );

    #[cfg(feature = "net")]
    sockaddr_storage_conv!(
        to_ipv6,
        Ipv6Address,
        sockaddr_in6,
        INET6,
        "Converts to [`Ipv6Address`], if the address family matches."
    );

    #[cfg(any(
        target_os = "android",
        target_os = "linux",
        target_os = "macos"
    ))]
    sockaddr_storage_conv!(
        to_vsock,
        VsockAddress,
        sockaddr_vm,
        VSOCK,
        "Converts to [`VsockAddress`], if the address family matches."
    );

    #[cfg(all(
        feature = "net",
        any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "illumos",
            target_os = "netbsd",
            target_os = "haiku",
            target_os = "aix",
            target_os = "openbsd",
        ),
    ))]
    /// Converts to [`LinkAddr`], if the address family matches.
    pub fn to_link(&self) -> Option<&LinkAddr> {
        if self.family() != AddressFamily::LINK {
            return None;
        }

        unsafe {
            Some(&*LinkAddr::from_raw_parts(
                self.as_ptr().cast(),
                self.len() as _,
            ))
        }
    }

    #[cfg(all(
        feature = "net",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux",),
    ))]
    sockaddr_storage_conv!(
        to_link,
        LinkAddress,
        sockaddr_ll,
        PACKET,
        "Converts to [`LinkAddress`], if the address family matches."
    );

    #[cfg(any(target_os = "android", target_os = "linux"))]
    sockaddr_storage_conv!(
        to_netlink,
        NetlinkAddress,
        sockaddr_nl,
        NETLINK,
        "Converts to [`NetlinkAddress`], if the address family matches."
    );

    #[cfg(all(feature = "ioctl", any(target_os = "ios", target_os = "macos")))]
    sockaddr_storage_conv!(
        to_sys_control,
        SysControlAddress,
        sockaddr_ctl,
        SYSTEM,
        "Converts to [`SysControlAddress`], if the address family matches."
    );

    /// Converts to [`UnixAddr`], if the address family matches.
    pub fn to_unix(&self) -> Option<&UnixAddr> {
        if self.family() != AddressFamily::UNIX {
            return None;
        }

        unsafe {
            Some(&*UnixAddr::from_raw_parts(
                self.as_ptr().cast(),
                self.len() as _,
            ))
        }
    }

    /// Converts to a boxed [`Addr`].
    ///
    /// # Panics
    ///
    /// This method panics if the allocation fails.
    pub fn to_boxed(&self) -> Box<Self> {
        let layout = std::alloc::Layout::for_value(self);

        unsafe {
            let box_ptr = if layout.size() != 0 {
                std::alloc::alloc(layout)
            } else {
                ptr::NonNull::<libc::sockaddr_storage>::dangling()
                    .as_ptr()
                    .cast()
            };

            assert_ne!(box_ptr, std::ptr::null_mut());

            ptr::copy_nonoverlapping(self.as_ptr().cast(), box_ptr, self.len());

            Box::from_raw(Self::from_raw_parts_mut(box_ptr.cast(), self.len()))
        }
    }

    /// Returns a raw pointer to the libc type.
    ///
    /// # Safety
    ///
    /// Keep in mind that it is in general *not* safe to dereference
    /// this pointer, as the length of this address can be smaller than
    /// the size of the libc type, leading to uninitialized fields.
    ///
    /// It is preferable to access individual fieds using [`addr_of!`].
    pub const fn as_ptr(&self) -> *const libc::sockaddr_storage {
        self.slice.as_ptr().cast()
    }

    /// Returns a raw mutable pointer to the libc type.
    ///
    /// # Safety
    ///
    /// Keep in mind that it is in general *not* safe to dereference
    /// this pointer, as the length of this address can be smaller than
    /// the size of the libc type, leading to uninitialized fields.
    ///
    /// It is preferable to access individual fieds using [`addr_of_mut!`].
    pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_storage {
        self.slice.as_mut_ptr().cast()
    }
}

impl AsRef<Addr> for Addr {
    fn as_ref(&self) -> &Addr {
        self
    }
}

impl ToOwned for Addr {
    type Owned = Address;

    fn to_owned(&self) -> Self::Owned {
        debug_assert!(self.len() <= mem::size_of::<libc::sockaddr_storage>());

        let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();

        unsafe {
            ptr::copy_nonoverlapping(
                self.as_ptr().cast::<u8>(),
                storage.as_mut_ptr().cast(),
                self.len(),
            );
        }

        Address {
            storage: unsafe { storage.assume_init() },
            #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "redox",
            ))]
            len: self.len() as _,
        }
    }
}

impl PartialEq<Address> for Addr {
    fn eq(&self, other: &Address) -> bool {
        *self == **other
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.family() {
            #[cfg(any(target_os = "android", target_os = "linux"))]
            AddressFamily::ALG => self.to_alg().unwrap().fmt(f),
            #[cfg(feature = "net")]
            AddressFamily::INET => self.to_ipv4().unwrap().fmt(f),
            #[cfg(feature = "net")]
            AddressFamily::INET6 => self.to_ipv6().unwrap().fmt(f),
            #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "illumos",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            #[cfg(feature = "net")]
            AddressFamily::LINK => self.to_link().unwrap().fmt(f),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            AddressFamily::NETLINK => self.to_netlink().unwrap().fmt(f),
            #[cfg(any(
                target_os = "android",
                target_os = "linux",
                target_os = "fuchsia"
            ))]
            #[cfg(feature = "net")]
            AddressFamily::PACKET => self.to_link().unwrap().fmt(f),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            #[cfg(feature = "ioctl")]
            AddressFamily::SYSTEM => self.to_sys_control().unwrap().fmt(f),
            AddressFamily::UNIX => self.to_unix().unwrap().fmt(f),
            #[cfg(any(
                target_os = "android",
                target_os = "linux",
                target_os = "macos"
            ))]
            AddressFamily::VSOCK => self.to_vsock().unwrap().fmt(f),
            AddressFamily::UNSPEC => "<Address family unspecified>".fmt(f),
            _ => "<Address family unknown>".fmt(f),
        }
    }
}

impl Default for &Addr {
    fn default() -> Self {
        Addr::empty()
    }
}

/// General purpose socket address with arbitrary address family.
///
/// Just like C's `sockaddr_storage`, this type is large enough to hold any type
/// of sockaddr.  It can be used as an argument with functions like
/// [`bind`](super::bind) and [`getsockname`](super::getsockname).
///
/// Can be converted to a more specific type using the `to_*` methods.
///
/// # Example
///
/// ```
/// # use nix::sys::socket::*;
/// # use std::str::FromStr;
/// # use std::os::unix::io::AsRawFd;
/// let localhost = Ipv4Address::from_str("127.0.0.1:8081").unwrap();
/// let fd = socket(AddressFamily::INET, SockType::Stream, SockFlag::empty(),
///     None).unwrap();
/// bind(fd.as_raw_fd(), &localhost).expect("bind");
/// let ss: Address = getsockname(fd.as_raw_fd()).expect("getsockname");
/// assert_eq!(&localhost, ss.to_ipv4().unwrap());
/// ```
#[derive(Clone, Copy, Debug, Eq)]
#[repr(C)]
pub struct Address {
    storage: libc::sockaddr_storage,
    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "linux",
        target_os = "redox",
    ))]
    pub(super) len: usize,
}

#[allow(clippy::len_without_is_empty)]
impl Address {
    sockaddr_family!();

    /// Returns the length of this socket address.
    ///
    /// The returned length is guaranteed to be in bounds of `[2, SIZE]` with `SIZE` being the size
    /// of the underlying libc type.
    ///
    /// The length can also be zero, in which case the address hasn't been initialized yet.
    pub const fn len(&self) -> usize {
        cfg_if! {
            if #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "redox",
            ))] {
                self.len
            } else {
                self.storage.ss_len as usize
            }
        }
    }

    /// Returns a raw pointer to the libc type.
    pub const fn as_ptr(&self) -> *const libc::sockaddr_storage {
        &self.storage
    }

    /// Returns a raw mutable pointer to the libc type.
    pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_storage {
        &mut self.storage
    }
}

impl Default for Address {
    fn default() -> Self {
        Self {
            storage: unsafe { mem::zeroed() },
            #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "redox",
            ))]
            len: 0,
        }
    }
}

impl AsRef<libc::sockaddr_storage> for Address {
    fn as_ref(&self) -> &libc::sockaddr_storage {
        &self.storage
    }
}

#[cfg(feature = "net")]
impl From<net::SocketAddrV4> for Address {
    fn from(s: net::SocketAddrV4) -> Self {
        unsafe {
            let mut ss = MaybeUninit::<libc::sockaddr_storage>::zeroed();

            let sin = Ipv4Address::from(s);

            ptr::copy_nonoverlapping(&sin.0, ss.as_mut_ptr().cast(), 1);

            Self {
                storage: ss.assume_init(),
                #[cfg(any(
                    target_os = "android",
                    target_os = "fuchsia",
                    target_os = "illumos",
                    target_os = "linux",
                    target_os = "redox",
                ))]
                len: mem::size_of::<libc::sockaddr_in>() as _,
            }
        }
    }
}

#[cfg(feature = "net")]
impl From<net::SocketAddrV6> for Address {
    fn from(s: net::SocketAddrV6) -> Self {
        unsafe {
            let mut ss = MaybeUninit::<libc::sockaddr_storage>::zeroed();

            let sin = Ipv6Address::from(s);

            ptr::copy_nonoverlapping(&sin.0, ss.as_mut_ptr().cast(), 1);

            Self {
                storage: ss.assume_init(),
                #[cfg(any(
                    target_os = "android",
                    target_os = "fuchsia",
                    target_os = "illumos",
                    target_os = "linux",
                    target_os = "redox",
                ))]
                len: mem::size_of::<libc::sockaddr_in6>() as _,
            }
        }
    }
}

#[cfg(feature = "net")]
impl From<net::SocketAddr> for Address {
    fn from(s: net::SocketAddr) -> Self {
        match s {
            net::SocketAddr::V4(sa4) => Self::from(sa4),
            net::SocketAddr::V6(sa6) => Self::from(sa6),
        }
    }
}

impl std::ops::Deref for Address {
    type Target = Addr;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Addr::from_raw_parts(&self.storage, self.len()) }
    }
}

impl std::ops::DerefMut for Address {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.len();

        unsafe { &mut *Addr::from_raw_parts_mut(&mut self.storage, len) }
    }
}

impl AsRef<Addr> for Address {
    fn as_ref(&self) -> &Addr {
        self
    }
}

impl AsMut<Addr> for Address {
    fn as_mut(&mut self) -> &mut Addr {
        self
    }
}

impl Borrow<Addr> for Address {
    fn borrow(&self) -> &Addr {
        self
    }
}

impl BorrowMut<Addr> for Address {
    fn borrow_mut(&mut self) -> &mut Addr {
        self
    }
}

impl PartialEq<Address> for Address {
    fn eq(&self, other: &Address) -> bool {
        **self == **other
    }
}

impl PartialEq<Addr> for Address {
    fn eq(&self, other: &Addr) -> bool {
        **self == *other
    }
}

impl Hash for Address {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod netlink {
    use super::*;
    use libc::{sa_family_t, sockaddr_nl};
    use std::{fmt, mem};

    /// Address for the Linux kernel user interface device.
    ///
    /// # References
    ///
    /// [netlink(7)](https://man7.org/linux/man-pages/man7/netlink.7.html)
    #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
    #[repr(transparent)]
    pub struct NetlinkAddress(pub(in super::super) sockaddr_nl);

    impl NetlinkAddress {
        /// Construct a new socket address from its port ID and multicast groups
        /// mask.
        pub fn new(pid: u32, groups: u32) -> NetlinkAddress {
            let mut addr: sockaddr_nl = unsafe { mem::zeroed() };
            addr.nl_family = libc::AF_NETLINK as sa_family_t;
            addr.nl_pid = pid;
            addr.nl_groups = groups;

            NetlinkAddress(addr)
        }

        sockaddr_family!();
        sockaddr_len_static!();

        /// Return the socket's port ID.
        pub const fn pid(&self) -> u32 {
            self.0.nl_pid
        }

        /// Return the socket's multicast groups mask
        pub const fn groups(&self) -> u32 {
            self.0.nl_groups
        }

        /// Returns a raw pointer to the libc type.
        pub const fn as_ptr(&self) -> *const libc::sockaddr_nl {
            &self.0
        }

        /// Returns a raw mutable pointer to the libc type.
        pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_nl {
            &mut self.0
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    impl AsRef<Addr> for NetlinkAddress {
        fn as_ref(&self) -> &Addr {
            unsafe {
                &*Addr::from_raw_parts(
                    &self.0 as *const _ as *const _,
                    self.len(),
                )
            }
        }
    }

    impl Borrow<Addr> for NetlinkAddress {
        fn borrow(&self) -> &Addr {
            self.as_ref()
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    impl From<NetlinkAddress> for Address {
        fn from(addr: NetlinkAddress) -> Self {
            let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();

            unsafe {
                ptr::copy_nonoverlapping(
                    &addr.0 as *const _ as *const u8,
                    storage.as_mut_ptr().cast(),
                    addr.len(),
                );

                Address {
                    storage: storage.assume_init(),
                    #[cfg(any(
                        target_os = "android",
                        target_os = "fuchsia",
                        target_os = "illumos",
                        target_os = "linux",
                        target_os = "redox",
                    ))]
                    len: addr.len() as _,
                }
            }
        }
    }

    impl AsRef<libc::sockaddr_nl> for NetlinkAddress {
        fn as_ref(&self) -> &libc::sockaddr_nl {
            &self.0
        }
    }

    impl fmt::Display for NetlinkAddress {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "pid: {} groups: {}", self.pid(), self.groups())
        }
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod alg {
    use super::*;
    use libc::{sockaddr_alg, AF_ALG};
    use std::ffi::CStr;
    use std::hash::{Hash, Hasher};
    use std::{fmt, mem, str};

    /// Socket address for the Linux kernel crypto API
    #[derive(Copy, Clone)]
    #[repr(transparent)]
    pub struct AlgAddress(pub(in super::super) sockaddr_alg);

    impl AlgAddress {
        sockaddr_family!();
        sockaddr_len_static!();

        /// Returns a raw pointer to the libc type.
        pub const fn as_ptr(&self) -> *const libc::sockaddr_alg {
            &self.0
        }

        /// Returns a raw mutable pointer to the libc type.
        pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_alg {
            &mut self.0
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    impl AsRef<Addr> for AlgAddress {
        fn as_ref(&self) -> &Addr {
            unsafe {
                &*Addr::from_raw_parts(
                    &self.0 as *const _ as *const _,
                    self.len(),
                )
            }
        }
    }

    impl Borrow<Addr> for AlgAddress {
        fn borrow(&self) -> &Addr {
            self.as_ref()
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    impl From<AlgAddress> for Address {
        fn from(addr: AlgAddress) -> Self {
            let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();

            unsafe {
                ptr::copy_nonoverlapping(
                    &addr.0 as *const _ as *const u8,
                    storage.as_mut_ptr().cast(),
                    addr.len(),
                );

                Address {
                    storage: storage.assume_init(),
                    #[cfg(any(
                        target_os = "android",
                        target_os = "fuchsia",
                        target_os = "illumos",
                        target_os = "linux",
                        target_os = "redox",
                    ))]
                    len: addr.len() as _,
                }
            }
        }
    }

    impl AsRef<libc::sockaddr_alg> for AlgAddress {
        fn as_ref(&self) -> &libc::sockaddr_alg {
            &self.0
        }
    }

    // , PartialEq, Eq, Debug, Hash
    impl PartialEq for AlgAddress {
        fn eq(&self, other: &Self) -> bool {
            let (inner, other) = (self.0, other.0);
            (
                inner.salg_family,
                &inner.salg_type[..],
                inner.salg_feat,
                inner.salg_mask,
                &inner.salg_name[..],
            ) == (
                other.salg_family,
                &other.salg_type[..],
                other.salg_feat,
                other.salg_mask,
                &other.salg_name[..],
            )
        }
    }

    impl Eq for AlgAddress {}

    impl Hash for AlgAddress {
        fn hash<H: Hasher>(&self, s: &mut H) {
            let inner = self.0;
            (
                inner.salg_family,
                &inner.salg_type[..],
                inner.salg_feat,
                inner.salg_mask,
                &inner.salg_name[..],
            )
                .hash(s);
        }
    }

    impl AlgAddress {
        /// Construct an `AF_ALG` socket from its cipher name and type.
        pub fn new(alg_type: &str, alg_name: &str) -> AlgAddress {
            let mut addr: sockaddr_alg = unsafe { mem::zeroed() };
            addr.salg_family = AF_ALG as u16;
            addr.salg_type[..alg_type.len()]
                .copy_from_slice(alg_type.to_string().as_bytes());
            addr.salg_name[..alg_name.len()]
                .copy_from_slice(alg_name.to_string().as_bytes());

            AlgAddress(addr)
        }

        /// Return the socket's cipher type, for example `hash` or `aead`.
        pub fn alg_type(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.0.salg_type.as_ptr().cast()) }
        }

        /// Return the socket's cipher name, for example `sha1`.
        pub fn alg_name(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.0.salg_name.as_ptr().cast()) }
        }
    }

    impl fmt::Display for AlgAddress {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "type: {} alg: {}",
                self.alg_name().to_string_lossy(),
                self.alg_type().to_string_lossy()
            )
        }
    }

    impl fmt::Debug for AlgAddress {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(self, f)
        }
    }
}

feature! {
#![feature = "ioctl"]
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub mod sys_control {
    use libc::{self, c_uchar};
    use std::{fmt, mem};
    use std::os::unix::io::RawFd;
    use crate::{Errno, Result};
    use super::*;

    // FIXME: Move type into `libc`
    #[repr(C)]
    #[derive(Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct ctl_ioc_info {
        pub ctl_id: u32,
        pub ctl_name: [c_uchar; MAX_KCTL_NAME],
    }

    const CTL_IOC_MAGIC: u8 = b'N';
    const CTL_IOC_INFO: u8 = 3;
    const MAX_KCTL_NAME: usize = 96;

    ioctl_readwrite!(ctl_info, CTL_IOC_MAGIC, CTL_IOC_INFO, ctl_ioc_info);

    /// Apple system control socket
    ///
    /// # References
    ///
    /// <https://developer.apple.com/documentation/kernel/sockaddr_ctl>
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    #[repr(transparent)]
    pub struct SysControlAddress(pub(in super::super) libc::sockaddr_ctl);

    impl SysControlAddress {
        sockaddr_family!();
        sockaddr_len_static!();

        /// Returns a raw pointer to the libc type.
        pub const fn as_ptr(&self) -> *const libc::sockaddr_ctl {
            &self.0
        }

        /// Returns a raw mutable pointer to the libc type.
        pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_ctl {
            &mut self.0
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    impl AsRef<Addr> for SysControlAddress {
        fn as_ref(&self) -> &Addr {
            unsafe { &*Addr::from_raw_parts(&self.0 as *const _ as *const _, self.len()) }
        }
    }

    impl Borrow<Addr> for SysControlAddress {
        fn borrow(&self) -> &Addr {
            self.as_ref()
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    impl From<SysControlAddress> for Address {
        fn from(addr: SysControlAddress) -> Self {
            let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();

            unsafe {
                ptr::copy_nonoverlapping(
                    &addr.0 as *const _ as *const u8,
                    storage.as_mut_ptr().cast(),
                    addr.len(),
                );

                Address {
                    storage: storage.assume_init(),
                    #[cfg(any(
                        target_os = "android",
                        target_os = "fuchsia",
                        target_os = "illumos",
                        target_os = "linux",
                        target_os = "redox",
                    ))]
                    len: addr.len() as _,
                }
            }
        }
    }

    impl AsRef<libc::sockaddr_ctl> for SysControlAddress {
        fn as_ref(&self) -> &libc::sockaddr_ctl {
            &self.0
        }
    }

    impl SysControlAddress {
        /// Construct a new `SysControlAddress` from its kernel unique identifier
        /// and unit number.
        pub const fn new(id: u32, unit: u32) -> SysControlAddress {
            let addr = libc::sockaddr_ctl {
                sc_len: mem::size_of::<libc::sockaddr_ctl>() as c_uchar,
                sc_family: libc::AF_SYSTEM as c_uchar,
                ss_sysaddr: libc::AF_SYS_CONTROL as u16,
                sc_id: id,
                sc_unit: unit,
                sc_reserved: [0; 5]
            };

            SysControlAddress(addr)
        }

        /// Construct a new `SysControlAddress` from its human readable name and
        /// unit number.
        pub fn from_name(sockfd: RawFd, name: &str, unit: u32) -> Result<SysControlAddress> {
            if name.len() > MAX_KCTL_NAME {
                return Err(Errno::ENAMETOOLONG);
            }

            let mut ctl_name = [0; MAX_KCTL_NAME];
            ctl_name[..name.len()].clone_from_slice(name.as_bytes());
            let mut info = ctl_ioc_info { ctl_id: 0, ctl_name };

            unsafe { ctl_info(sockfd, &mut info)?; }

            Ok(SysControlAddress::new(info.ctl_id, unit))
        }

        /// Return the kernel unique identifier
        pub const fn id(&self) -> u32 {
            self.0.sc_id
        }

        /// Return the kernel controller private unit number.
        pub const fn unit(&self) -> u32 {
            self.0.sc_unit
        }
    }

    impl fmt::Display for SysControlAddress {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Debug::fmt(self, f)
        }
    }
}
}

#[cfg(any(target_os = "android", target_os = "linux", target_os = "fuchsia"))]
mod datalink {
    feature! {
    #![feature = "net"]
    use super::*;

    /// Hardware Address
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    #[repr(transparent)]
    pub struct LinkAddress(pub(in super::super) libc::sockaddr_ll);

    impl LinkAddress {
        sockaddr_family!();
        sockaddr_len_static!();

        /// Physical-layer protocol
        pub fn protocol(&self) -> u16 {
            self.0.sll_protocol
        }

        /// Interface number
        pub fn ifindex(&self) -> usize {
            self.0.sll_ifindex as usize
        }

        /// ARP hardware type
        pub fn hatype(&self) -> u16 {
            self.0.sll_hatype
        }

        /// Packet type
        pub fn pkttype(&self) -> u8 {
            self.0.sll_pkttype
        }

        /// Length of MAC address
        pub fn halen(&self) -> usize {
            self.0.sll_halen as usize
        }

        /// Physical-layer address (MAC)
        // Returns an Option just for cross-platform compatibility
        pub fn addr(&self) -> Option<[u8; 6]> {
            Some([
                self.0.sll_addr[0],
                self.0.sll_addr[1],
                self.0.sll_addr[2],
                self.0.sll_addr[3],
                self.0.sll_addr[4],
                self.0.sll_addr[5],
            ])
        }

        /// Returns a raw pointer to the libc type.
        pub const fn as_ptr(&self) -> *const libc::sockaddr_ll {
            &self.0
        }

        /// Returns a raw mutable pointer to the libc type.
        pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_ll {
            &mut self.0
        }
    }

    impl fmt::Display for LinkAddress {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if let Some(addr) = self.addr() {
                write!(f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    addr[0],
                    addr[1],
                    addr[2],
                    addr[3],
                    addr[4],
                    addr[5])
            } else {
                Ok(())
            }
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    impl AsRef<Addr> for LinkAddress {
        fn as_ref(&self) -> &Addr {
            unsafe { &*Addr::from_raw_parts(&self.0 as *const _ as *const _, self.len()) }
        }
    }

    impl Borrow<Addr> for LinkAddress {
        fn borrow(&self) -> &Addr {
            self.as_ref()
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    impl From<LinkAddress> for Address {
        fn from(addr: LinkAddress) -> Self {
            let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();

            unsafe {
                ptr::copy_nonoverlapping(
                    &addr.0 as *const _ as *const u8,
                    storage.as_mut_ptr().cast(),
                    addr.len(),
                );

                Address {
                    storage: storage.assume_init(),
                    #[cfg(any(
                        target_os = "android",
                        target_os = "fuchsia",
                        target_os = "illumos",
                        target_os = "linux",
                        target_os = "redox",
                    ))]
                    len: addr.len() as _,
                }
            }
        }
    }

    impl AsRef<libc::sockaddr_ll> for LinkAddress {
        fn as_ref(&self) -> &libc::sockaddr_ll {
            &self.0
        }
    }

    }
}

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "haiku",
    target_os = "aix",
    target_os = "openbsd"
))]
mod datalink {
    feature! {
    #![feature = "net"]
    use super::*;

    unsafe fn link_true_len(addr: *const libc::sockaddr_dl) -> usize {
        const OFFSET: usize = offset_of!(libc::sockaddr_dl, sdl_data);

        cfg_if! {
            if #[cfg(target_os = "haiku")] {
                unsafe {
                    OFFSET + *addr_of!((*addr).sdl_nlen) as usize + *addr_of!((*addr).sdl_alen) as usize
                }
            } else {
                unsafe {
                    OFFSET + *addr_of!((*addr).sdl_nlen) as usize + *addr_of!((*addr).sdl_alen) as usize + *addr_of!((*addr).sdl_slen) as usize
                }
            }
        }
    }

    /// Hardware Address
    #[derive(Debug, PartialEq, Eq, Hash)]
    #[cfg_attr(not(doc), repr(C, align(2)))]
    pub struct LinkAddr {
        slice: [u8],
    }

    #[allow(clippy::len_without_is_empty)]
    impl LinkAddr {
        const _ALIGN: () = assert!(mem::align_of::<libc::sockaddr_dl>() == 2);

        // SAFETY: When dereferencing, `data` must be valid for reads of at least `len` bytes
        // and must point to a valid link layer address.
        #[allow(clippy::cast_ptr_alignment)]
        pub(super) const fn from_raw_parts(data: *const libc::sockaddr_dl, len: usize) -> *const Self {
            let data_ptr = data.cast::<u8>();
            let slice_ptr = ptr::slice_from_raw_parts(data_ptr, len);

            slice_ptr as *const Self
        }

        // SAFETY: When dereferencing, `data` must be valid for reads and writes of at least `len` bytes
        // and must point to a valid link layer address.
        #[allow(clippy::cast_ptr_alignment)]
        fn from_raw_parts_mut(data: *mut libc::sockaddr_dl, len: usize) -> *mut Self {
            let data_ptr = data.cast::<u8>();
            let slice_ptr = ptr::slice_from_raw_parts_mut(data_ptr, len);


            slice_ptr as *mut Self
        }

        /// Returns the length of this socket address.
        ///
        /// The returned length is guaranteed to be in bounds of `[2, MAX]` with `MAX` being the
        /// maximum length of a socket address (i.e. the size of [`libc::sockaddr_storage`]).
        pub fn len(&self) -> usize {
            self.slice.len()
        }

        /// interface index, if != 0, system given index for interface
        #[cfg(not(target_os = "haiku"))]
        pub fn ifindex(&self) -> usize {
            unsafe {
                *addr_of!((*self.as_ptr()).sdl_index) as _
            }
        }

        /// Datalink type
        #[cfg(not(target_os = "haiku"))]
        pub fn datalink_type(&self) -> u8 {
            unsafe {
                *addr_of!((*self.as_ptr()).sdl_type) as _
            }
        }

        /// MAC address start position
        pub fn nlen(&self) -> usize {
            unsafe {
                *addr_of!((*self.as_ptr()).sdl_nlen) as _
            }
        }

        /// link level address length
        pub fn alen(&self) -> usize {
            unsafe {
                *addr_of!((*self.as_ptr()).sdl_alen) as _
            }
        }

        /// link layer selector length
        #[cfg(not(target_os = "haiku"))]
        pub fn slen(&self) -> usize {
            unsafe {
                *addr_of!((*self.as_ptr()).sdl_slen) as _
            }
        }

        /// Returns the "true" length of this link address, including
        /// its name, address and selector.
        ///
        /// This value can both be smaller or larger than [`Self::len`].
        pub fn true_len(&self) -> usize {
            let len = unsafe { link_true_len(self.as_ptr()) };

            debug_assert!(len <= self.len());

            len
        }

        /// Physical-layer address.
        ///
        /// Usually 6 bytes (MAC address).
        // The cast is not unnecessary on all platforms.
        pub fn addr(&self) -> Option<&[u8]> {
            let nlen = self.nlen();
            let alen = self.alen();

            let offset = offset_of!(libc::sockaddr_dl, sdl_data) + nlen;

            if offset + alen > self.len() {
                return None;
            }

            Some(&self.slice[offset..offset + alen])
        }

        /// Converts to a boxed [`UnixAddr`].
        ///
        /// # Panics
        ///
        /// This method panics if the allocation fails.
        pub fn to_boxed(&self) -> Box<Self> {
            let layout = std::alloc::Layout::for_value(self);

            unsafe {
                let box_ptr = std::alloc::alloc(layout);
                assert_ne!(box_ptr, std::ptr::null_mut());

                ptr::copy_nonoverlapping(self.as_ptr().cast(), box_ptr, self.len());

                Box::from_raw(Self::from_raw_parts_mut(box_ptr.cast(), self.len()))
            }
        }

        /// Converts to an owned [`LinkAddress`] if it fits.
        ///
        /// If you are unsure whether it fits into [`LinkAddress`], consider using
        /// [`Self::to_boxed`] instead.
        pub fn to_owned(&self) -> Option<LinkAddress> {
            if self.len() > mem::size_of::<libc::sockaddr_dl>() {
                return None;
            }

            unsafe {
                let mut addr = MaybeUninit::<libc::sockaddr_dl>::zeroed();

                ptr::copy_nonoverlapping(self.slice.as_ptr(), addr.as_mut_ptr().cast(), self.len());

                Some(LinkAddress(addr.assume_init()))
            }
        }

        /// Converts to an owned [`LinkAddress`], truncating the length to its "true" length.
        ///
        /// This method returns `None` if the "true" length is larger that the length of the address
        /// or the size of `libc::sockaddr_dl`.
        pub fn to_owned_truncate(&self) -> Option<LinkAddress> {
            cfg_if! {
                if #[cfg(not(target_os = "illumos"))] {
                    if self.true_len() > self.len() || self.true_len() > mem::size_of::<libc::sockaddr_dl>() {
                        return None;
                    }

                    unsafe {
                        let mut addr = MaybeUninit::<libc::sockaddr_dl>::zeroed();

                        ptr::copy_nonoverlapping(self.slice.as_ptr(), addr.as_mut_ptr().cast(), self.true_len());

                        addr_of_mut!((*addr.as_mut_ptr()).sdl_len).write(self.true_len() as _);

                        Some(LinkAddress(addr.assume_init()))
                    }
                } else {
                    // For illumos, len and true_len are equal
                    self.to_owned()
                }
            }
        }

        /// Returns a raw pointer to the libc type.
        ///
        /// # Safety
        ///
        /// Keep in mind that it is in general *not* safe to dereference
        /// this pointer, as the length of this address can be smaller than
        /// the size of the libc type, leading to uninitialized fields.
        ///
        /// It is preferable to access individual fieds using [`addr_of!`].
        pub const fn as_ptr(&self) -> *const libc::sockaddr_dl {
            self.slice.as_ptr().cast()
        }

        /// Returns a raw mutable pointer to the libc type.
        ///
        /// # Safety
        ///
        /// Keep in mind that it is in general *not* safe to dereference
        /// this pointer, as the length of this address can be smaller than
        /// the size of the libc type, leading to uninitialized fields.
        ///
        /// It is preferable to access individual fieds using [`addr_of_mut!`].
        pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_dl {
            self.slice.as_mut_ptr().cast()
        }
    }

    impl fmt::Display for LinkAddr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if let Some(addr) = self.addr() {
                if addr.len() == 6 {
                    write!(f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                        addr[0],
                        addr[1],
                        addr[2],
                        addr[3],
                        addr[4],
                        addr[5],
                    )
                } else {
                    write!(f, "<unsupported address format>")
                }
            } else {
                Ok(())
            }
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    impl AsRef<Addr> for LinkAddr {
        fn as_ref(&self) -> &Addr {
            unsafe { &*Addr::from_raw_parts(self.as_ptr() as *const _, self.len()) }
        }
    }

    impl Borrow<Addr> for LinkAddr {
        fn borrow(&self) -> &Addr {
            self.as_ref()
        }
    }

    /// Hardware Address
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    #[repr(transparent)]
    pub struct LinkAddress(pub(in super::super) libc::sockaddr_dl);

    impl LinkAddress {
        sockaddr_family!();

        /// Returns the length of this socket address.
        ///
        /// The returned length is guaranteed to be in bounds of [2, SIZE] with SIZE being the size
        /// of the underlying libc type.
        fn len(&self) -> usize {
            cfg_if! {
                if #[cfg(not(target_os = "illumos"))] {
                    let len = self.0.sdl_len as _;
                } else {
                    // On illumos, `sdl_len` is not available. That's the best guess we can make.
                    let len = unsafe { link_true_len(&self.0) };
                }
            }

            debug_assert!(len <= mem::size_of::<libc::sockaddr_dl>());

            len
        }


        /// Returns a raw pointer to the libc type.
        pub const fn as_ptr(&self) -> *const libc::sockaddr_dl {
            &self.0
        }

        /// Returns a raw mutable pointer to the libc type.
        pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_dl {
            &mut self.0
        }
    }

    impl std::ops::Deref for LinkAddress {
        type Target = LinkAddr;

        fn deref(&self) -> &Self::Target {
            unsafe {
                &*LinkAddr::from_raw_parts(&self.0, self.len())
            }
        }
    }

    impl std::ops::DerefMut for LinkAddress {
        fn deref_mut(&mut self) -> &mut Self::Target {
            let len = self.len();

            unsafe {
                &mut *LinkAddr::from_raw_parts_mut(&mut self.0, len)
            }
        }
    }

    impl fmt::Display for LinkAddress {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            (**self).fmt(f)
        }
    }

    impl AsRef<Addr> for LinkAddress {
        fn as_ref(&self) -> &Addr {
            (**self).as_ref()
        }
    }

    impl Borrow<Addr> for LinkAddress {
        fn borrow(&self) -> &Addr {
            (**self).borrow()
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    impl From<LinkAddress> for Address {
        fn from(addr: LinkAddress) -> Self {
            let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();

            unsafe {
                ptr::copy_nonoverlapping(
                    &addr.0 as *const _ as *const u8,
                    storage.as_mut_ptr().cast(),
                    addr.len(),
                );

                Address {
                    storage: storage.assume_init(),
                    #[cfg(any(
                        target_os = "android",
                        target_os = "fuchsia",
                        target_os = "illumos",
                        target_os = "linux",
                        target_os = "redox",
                    ))]
                    len: addr.len() as _,
                }
            }
        }
    }

    impl AsRef<libc::sockaddr_dl> for LinkAddress {
        fn as_ref(&self) -> &libc::sockaddr_dl {
            &self.0
        }
    }
    }
}

#[cfg(any(target_os = "android", target_os = "linux", target_os = "macos"))]
pub mod vsock {
    use super::*;
    use libc::{sa_family_t, sockaddr_vm};
    use std::hash::{Hash, Hasher};
    use std::{fmt, mem};

    /// Socket address for VMWare VSockets protocol
    ///
    /// # References
    ///
    /// [vsock(7)](https://man7.org/linux/man-pages/man7/vsock.7.html)
    #[derive(Copy, Clone)]
    #[repr(transparent)]
    pub struct VsockAddress(pub(in super::super) sockaddr_vm);

    #[allow(clippy::cast_ptr_alignment)]
    impl AsRef<Addr> for VsockAddress {
        fn as_ref(&self) -> &Addr {
            unsafe {
                &*Addr::from_raw_parts(
                    &self.0 as *const _ as *const _,
                    self.len(),
                )
            }
        }
    }

    impl Borrow<Addr> for VsockAddress {
        fn borrow(&self) -> &Addr {
            self.as_ref()
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    impl From<VsockAddress> for Address {
        fn from(addr: VsockAddress) -> Self {
            let mut storage = MaybeUninit::<libc::sockaddr_storage>::zeroed();

            unsafe {
                ptr::copy_nonoverlapping(
                    &addr.0 as *const _ as *const u8,
                    storage.as_mut_ptr().cast(),
                    addr.len(),
                );

                Address {
                    storage: storage.assume_init(),
                    #[cfg(any(
                        target_os = "android",
                        target_os = "fuchsia",
                        target_os = "illumos",
                        target_os = "linux",
                        target_os = "redox",
                    ))]
                    len: addr.len() as _,
                }
            }
        }
    }

    impl AsRef<libc::sockaddr_vm> for VsockAddress {
        fn as_ref(&self) -> &libc::sockaddr_vm {
            &self.0
        }
    }

    impl PartialEq for VsockAddress {
        #[cfg(any(target_os = "android", target_os = "linux"))]
        fn eq(&self, other: &Self) -> bool {
            let (inner, other) = (self.0, other.0);
            (inner.svm_family, inner.svm_cid, inner.svm_port)
                == (other.svm_family, other.svm_cid, other.svm_port)
        }
        #[cfg(target_os = "macos")]
        fn eq(&self, other: &Self) -> bool {
            let (inner, other) = (self.0, other.0);
            (
                inner.svm_family,
                inner.svm_cid,
                inner.svm_port,
                inner.svm_len,
            ) == (
                other.svm_family,
                other.svm_cid,
                other.svm_port,
                inner.svm_len,
            )
        }
    }

    impl Eq for VsockAddress {}

    impl Hash for VsockAddress {
        #[cfg(any(target_os = "android", target_os = "linux"))]
        fn hash<H: Hasher>(&self, s: &mut H) {
            let inner = self.0;
            (inner.svm_family, inner.svm_cid, inner.svm_port).hash(s);
        }
        #[cfg(target_os = "macos")]
        fn hash<H: Hasher>(&self, s: &mut H) {
            let inner = self.0;
            (
                inner.svm_family,
                inner.svm_cid,
                inner.svm_port,
                inner.svm_len,
            )
                .hash(s);
        }
    }

    /// VSOCK Address
    ///
    /// The address for AF_VSOCK socket is defined as a combination of a
    /// 32-bit Context Identifier (CID) and a 32-bit port number.
    impl VsockAddress {
        sockaddr_family!();
        sockaddr_len_static!();

        /// Construct a `VsockAddr` from its raw fields.
        pub fn new(cid: u32, port: u32) -> VsockAddress {
            let mut addr: sockaddr_vm = unsafe { mem::zeroed() };
            addr.svm_family = libc::AF_VSOCK as sa_family_t;
            addr.svm_cid = cid;
            addr.svm_port = port;

            #[cfg(target_os = "macos")]
            {
                addr.svm_len = std::mem::size_of::<sockaddr_vm>() as u8;
            }
            VsockAddress(addr)
        }

        /// Context Identifier (CID)
        pub fn cid(&self) -> u32 {
            self.0.svm_cid
        }

        /// Port number
        pub fn port(&self) -> u32 {
            self.0.svm_port
        }

        /// Returns a raw pointer to the libc type.
        pub const fn as_ptr(&self) -> *const sockaddr_vm {
            &self.0
        }

        /// Returns a raw mutable pointer to the libc type.
        pub fn as_mut_ptr(&mut self) -> *mut sockaddr_vm {
            &mut self.0
        }
    }

    impl fmt::Display for VsockAddress {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "cid: {} port: {}", self.cid(), self.port())
        }
    }

    impl fmt::Debug for VsockAddress {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(self, f)
        }
    }
}

macro_rules! raw_address_conv {
    ($fname:ident, $nixty:tt, $libcty:ident, $af:ident, $doc:tt) => {
        #[doc = $doc]
        pub fn $fname(&self) -> Option<&$nixty> {
            if self.family() != AddressFamily::$af {
                return None;
            }

            unsafe { Some(&*(self.as_ptr().cast())) }
        }
    };
}

/// Non-owning pointer to a raw socket address.
///
/// In contrast to [`Addr`], this type may or may not have
/// a length associated with it, depending on the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct RawAddr<'a> {
    addr: &'a libc::sockaddr,
}

#[allow(clippy::len_without_is_empty)]
impl<'a> RawAddr<'a> {
    /// Creates a new `RawAddr` from a reference to a socket address.
    ///
    /// # Safety
    ///
    /// The reference must be castable to the concrete type of the address,
    /// based on its address family.
    #[allow(unused)]
    pub(crate) const unsafe fn new(addr: &'a libc::sockaddr) -> Option<Self> {
        Some(Self { addr })
    }

    /// Returns a raw pointer to the libc type.
    ///
    /// This pointer can be casted to a pointer of the concrete type of the address,
    /// based on its address family.
    pub const fn as_ptr(&self) -> *const libc::sockaddr {
        self.addr as *const _
    }

    /// Returns the address family of the address.
    pub const fn family(&self) -> AddressFamily {
        AddressFamily::of(self.addr)
    }

    /// Returns the length of the address, if possible to derive.
    pub fn len(&self) -> Option<usize> {
        cfg_if! {
            if #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "redox",
            ))] {
                match self.family() {
                    #[cfg(any(target_os = "android", target_os = "linux"))]
                    AddressFamily::ALG => Some(mem::size_of::<libc::sockaddr_alg>()),
                    #[cfg(feature = "net")]
                    AddressFamily::INET => Some(mem::size_of::<libc::sockaddr_in>()),
                    #[cfg(feature = "net")]
                    AddressFamily::INET6 => Some(mem::size_of::<libc::sockaddr_in6>()),
                    #[cfg(any(target_os = "android", target_os = "linux"))]
                    AddressFamily::NETLINK => Some(mem::size_of::<libc::sockaddr_nl>()),
                    #[cfg(all(
                        feature = "net",
                        any(
                            target_os = "android",
                            target_os = "fuchsia",
                            target_os = "linux",
                        ),
                    ))]
                    AddressFamily::PACKET => Some(mem::size_of::<libc::sockaddr_ll>()),
                    #[cfg(all(
                        feature = "net",
                        target_os = "illumos",
                    ))]
                    AddressFamily::LINK => {
                        let ptr = self.as_ptr().cast::<libc::sockaddr_dl>();
                        let nlen = unsafe { addr_of!((*ptr).sdl_nlen).read() as usize };
                        let alen = unsafe { addr_of!((*ptr).sdl_alen).read() as usize };
                        let slen = unsafe { addr_of!((*ptr).sdl_slen).read() as usize };
                        Some(offset_of!(libc::sockaddr_dl, sdl_data) + nlen + alen + slen)
                    }
                    #[cfg(any(target_os = "android", target_os = "linux"))]
                    AddressFamily::VSOCK => Some(mem::size_of::<libc::sockaddr_vm>()),
                    _ => None,
                }
            } else {
                Some(self.addr.sa_len as _)
            }
        }
    }

    #[cfg(any(target_os = "android", target_os = "linux"))]
    raw_address_conv!(
        to_alg,
        AlgAddress,
        sockaddr_alg,
        ALG,
        "Converts to [`AlgAddress`], if the address family matches."
    );

    #[cfg(feature = "net")]
    raw_address_conv!(
        to_ipv4,
        Ipv4Address,
        sockaddr_in,
        INET,
        "Converts to [`Ipv4Address`], if the address family matches."
    );

    #[cfg(feature = "net")]
    raw_address_conv!(
        to_ipv6,
        Ipv6Address,
        sockaddr_in6,
        INET6,
        "Converts to [`Ipv6Address`], if the address family matches."
    );

    #[cfg(any(
        target_os = "android",
        target_os = "linux",
        target_os = "macos"
    ))]
    raw_address_conv!(
        to_vsock,
        VsockAddress,
        sockaddr_vm,
        VSOCK,
        "Converts to [`VsockAddress`], if the address family matches."
    );

    #[cfg(all(
        feature = "net",
        any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "illumos",
            target_os = "netbsd",
            target_os = "haiku",
            target_os = "aix",
            target_os = "openbsd",
        ),
    ))]
    /// Converts to [`LinkAddr`], if the address family matches.
    pub fn to_link(&self) -> Option<&LinkAddr> {
        if self.family() != AddressFamily::LINK {
            return None;
        }

        unsafe {
            Some(&*LinkAddr::from_raw_parts(
                self.as_ptr().cast(),
                self.len().unwrap(),
            ))
        }
    }

    #[cfg(all(
        feature = "net",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux",),
    ))]
    raw_address_conv!(
        to_link,
        LinkAddress,
        sockaddr_ll,
        PACKET,
        "Converts to [`LinkAddress`], if the address family matches."
    );

    #[cfg(any(target_os = "android", target_os = "linux"))]
    raw_address_conv!(
        to_netlink,
        NetlinkAddress,
        sockaddr_nl,
        NETLINK,
        "Converts to [`NetlinkAddress`], if the address family matches."
    );

    #[cfg(all(feature = "ioctl", any(target_os = "ios", target_os = "macos")))]
    raw_address_conv!(
        to_sys_control,
        SysControlAddress,
        sockaddr_ctl,
        SYSTEM,
        "Converts to [`SysControlAddress`], if the address family matches."
    );

    /// Converts to [`UnixAddr`], if the address family matches.
    #[cfg(not(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "linux",
        target_os = "redox",
    )))]
    pub fn to_unix(&self) -> Option<&UnixAddr> {
        if self.family() != AddressFamily::UNIX {
            return None;
        }

        unsafe {
            Some(&*UnixAddr::from_raw_parts(
                self.as_ptr().cast(),
                self.len().unwrap(),
            ))
        }
    }

    /// Converts to [`Addr`], if its length can be derived.
    pub fn to_addr(&self) -> Option<&Addr> {
        self.len().map(|l| unsafe {
            &*Addr::from_raw_parts(&*self.as_ptr().cast(), l)
        })
    }
}

impl<'a> fmt::Display for RawAddr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_addr() {
            Some(sa) => sa.fmt(f),
            None => "<unsupported>".fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod types {
        use super::*;

        #[test]
        fn test_ipv4addr_to_libc() {
            let s = std::net::Ipv4Addr::new(1, 2, 3, 4);
            let l = ipv4addr_to_libc(s);
            assert_eq!(l.s_addr, u32::to_be(0x01020304));
        }

        #[test]
        fn test_ipv6addr_to_libc() {
            let s = std::net::Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8);
            let l = ipv6addr_to_libc(&s);
            assert_eq!(
                l.s6_addr,
                [0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8]
            );
        }
    }

    #[cfg(not(target_os = "redox"))]
    mod link {
        #![allow(clippy::cast_ptr_alignment)]

        #[allow(unused_imports)]
        use super::*;

        /// Don't panic when trying to display an empty datalink address
        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        #[test]
        fn test_datalink_display() {
            use super::super::LinkAddress;
            use std::mem;

            let len = offset_of!(libc::sockaddr_dl, sdl_data) + 3;

            let la = LinkAddress(libc::sockaddr_dl {
                sdl_len: len as _,
                sdl_family: 18,
                sdl_index: 5,
                sdl_type: 24,
                sdl_nlen: 3,
                sdl_alen: 0,
                sdl_slen: 0,
                ..unsafe { mem::zeroed() }
            });
            format!("{la}");
        }

        #[cfg(any(target_os = "ios", target_os = "macos"))]
        #[test]
        fn macos_tap() {
            let bytes = [
                20i8, 18, 7, 0, 6, 3, 6, 0, 101, 110, 48, 24, 101, -112, -35,
                76, -80, 0, 0, 0,
            ];
            let ptr = bytes.as_ptr();
            let sa = ptr as *const libc::sockaddr;
            let len = bytes.len();

            let sock_addr = unsafe { &*Addr::from_raw_parts(sa.cast(), len) };
            assert_eq!(sock_addr.family(), AddressFamily::LINK);
            match sock_addr.to_link() {
                Some(dl) => {
                    assert_eq!(
                        dl.addr(),
                        Some(&[24u8, 101, 144, 221, 76, 176][..])
                    )
                }
                None => panic!("Can't unwrap sockaddr storage"),
            }
        }

        #[cfg(target_os = "illumos")]
        #[test]
        fn illumos_tap() {
            let bytes = [25u8, 0, 0, 0, 6, 0, 6, 0, 24, 101, 144, 221, 76, 176];
            let ptr = bytes.as_ptr();
            let sa = ptr as *const libc::sockaddr;
            let len = bytes.len();
            let sock_addr = unsafe { &*Addr::from_raw_parts(sa.cast(), len) };

            assert_eq!(sock_addr.family(), AddressFamily::LINK);

            assert_eq!(
                sock_addr.to_link().unwrap().addr(),
                Some(&[24u8, 101, 144, 221, 76, 176][..])
            );
        }

        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "illumos",
            target_os = "netbsd",
            // haiku doesn't support `getifaddrs`
            // aix doesn't support `getifaddrs`
            target_os = "openbsd",
        ))]
        #[test]
        fn test_to_owned() {
            let ifaddrs = crate::ifaddrs::getifaddrs().unwrap();

            for addr in ifaddrs
                .iter()
                .map(|ifa| ifa.address)
                .filter_map(|a| a.filter(|a| a.family() == AddressFamily::LINK))
            {
                let link_addr: &LinkAddr = addr.to_link().unwrap();

                // On FreeBSD, the length gets rounded up to the next multiple
                // of 8 bytes, which is 56 bytes.
                //
                // Source: https://github.com/freebsd/freebsd-src/blob/dcc4d2939f789a6d1f272ffeab2068ba2b7525ea/lib/libc/net/getifaddrs.c#L61
                //
                // That means, that its length is larger than 54, the size of
                // `libc::sockaddr_dl`.
                #[cfg(target_os = "freebsd")]
                assert_eq!(link_addr.len(), 56);

                #[cfg(target_os = "freebsd")]
                assert_eq!(link_addr.to_owned(), None);

                let owned = link_addr.to_owned_truncate().unwrap();

                assert!(owned.len() <= std::mem::size_of::<LinkAddress>());

                assert_eq!(owned.len(), owned.true_len());

                // We can dereference `owned` and get the same result when
                // displaying it.
                assert_eq!(format!("{}", owned), format!("{}", &*owned));
            }
        }
    }

    mod sockaddr_in {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn display() {
            let s = "127.0.0.1:8080";
            let addr = Ipv4Address::from_str(s).unwrap();
            assert_eq!(s, format!("{addr}"));
        }
    }

    mod sockaddr_in6 {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn display() {
            let s = "[1234:5678:90ab:cdef::1111:2222]:8080";
            let addr = Ipv6Address::from_str(s).unwrap();
            assert_eq!(s, format!("{addr}"));
        }

        #[test]
        // Ensure that we can convert to-and-from std::net variants without change.
        fn to_and_from() {
            let s = "[1234:5678:90ab:cdef::1111:2222]:8080";
            let mut nix_sin6 = Ipv6Address::from_str(s).unwrap();
            nix_sin6.0.sin6_flowinfo = 0x12345678;
            nix_sin6.0.sin6_scope_id = 0x9abcdef0;

            let std_sin6: std::net::SocketAddrV6 = nix_sin6.into();
            assert_eq!(nix_sin6, std_sin6.into());
        }
    }

    mod unixaddr {
        #[allow(unused_imports)]
        use super::*;

        #[cfg(any(target_os = "android", target_os = "linux"))]
        #[test]
        fn abstract_sun_path() {
            let name = String::from("nix\0abstract\0test");
            let addr = UnixAddress::new_abstract(name.as_bytes()).unwrap();

            let sun_path1 =
                unsafe { &(*addr.as_ptr()).sun_path[..addr.path_len()] };
            let sun_path2 = [
                0, 110, 105, 120, 0, 97, 98, 115, 116, 114, 97, 99, 116, 0,
                116, 101, 115, 116,
            ];
            assert_eq!(sun_path1, sun_path2);
        }
    }
}
