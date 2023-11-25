#[cfg(any(
    bsd,
    linux_android,
    target_os = "illumos",
    target_os = "haiku",
    target_os = "fuchsia",
    target_os = "aix",
))]
#[cfg(feature = "net")]
pub use self::datalink::LinkAddr;
#[cfg(any(linux_android, target_os = "macos"))]
pub use self::vsock::VsockAddr;
use super::sa_family_t;
use crate::errno::Errno;
#[cfg(linux_android)]
use crate::sys::socket::addr::alg::AlgAddr;
#[cfg(linux_android)]
use crate::sys::socket::addr::netlink::NetlinkAddr;
#[cfg(all(feature = "ioctl", apple_targets))]
use crate::sys::socket::addr::sys_control::SysControlAddr;
use crate::{NixPath, Result};
use cfg_if::cfg_if;
use memoffset::offset_of;
use std::convert::TryInto;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
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
    #[cfg(solarish)]
    pub const _802: Self = Self(libc::AF_802);
    /// Represents `AF_ALG`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const ALG: Self = Self(libc::AF_ALG);
    /// Represents `AF_APPLETALK`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        linux_android,
        netbsdlike,
        solarish,
        target_os = "fuchsia",
        target_os = "haiku",
    ))]
    pub const APPLETALK: Self = Self(libc::AF_APPLETALK);
    /// Represents `AF_ARP`.
    #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
    pub const ARP: Self = Self(libc::AF_ARP);
    /// Represents `AF_ASH`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const ASH: Self = Self(libc::AF_ASH);
    /// Represents `AF_ATM`.
    #[cfg(freebsdlike)]
    pub const ATM: Self = Self(libc::AF_ATM);
    /// Represents `AF_ATMPVC`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const ATMPVC: Self = Self(libc::AF_ATMPVC);
    /// Represents `AF_ATMSVC`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const ATMSVC: Self = Self(libc::AF_ATMSVC);
    /// Represents `AF_AX25`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const AX25: Self = Self(libc::AF_AX25);
    /// Represents `AF_BLUETOOTH`.
    #[cfg(any(
        linux_android,
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    pub const BLUETOOTH: Self = Self(libc::AF_BLUETOOTH);
    /// Represents `AF_BRIDGE`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const BRIDGE: Self = Self(libc::AF_BRIDGE);
    /// Represents `AF_CAIF`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const CAIF: Self = Self(libc::AF_CAIF);
    /// Represents `AF_CAN`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const CAN: Self = Self(libc::AF_CAN);
    /// Represents `AF_CCITT`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike, solarish))]
    pub const CCITT: Self = Self(libc::AF_CCITT);
    /// Represents `AF_CHAOS`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike, solarish))]
    pub const CHAOS: Self = Self(libc::AF_CHAOS);
    /// Represents `AF_CNT`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike))]
    pub const CNT: Self = Self(libc::AF_CNT);
    /// Represents `AF_COIP`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike))]
    pub const COIP: Self = Self(libc::AF_COIP);
    /// Represents `AF_DATAKIT`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike, solarish))]
    pub const DATAKIT: Self = Self(libc::AF_DATAKIT);
    /// Represents `AF_DECnet`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        linux_android,
        netbsdlike,
        solarish,
        target_os = "fuchsia",
    ))]
    pub const DECNET: Self = Self(libc::AF_DECnet);
    /// Represents `AF_DLI`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        netbsdlike,
        solarish,
        target_os = "haiku",
    ))]
    pub const DLI: Self = Self(libc::AF_DLI);
    /// Represents `AF_E164`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike))]
    pub const E164: Self = Self(libc::AF_E164);
    /// Represents `AF_ECMA`.
    #[cfg(any(apple_targets, freebsdlike, solarish, target_os = "openbsd"))]
    pub const ECMA: Self = Self(libc::AF_ECMA);
    /// Represents `AF_ECONET`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const ECONET: Self = Self(libc::AF_ECONET);
    /// Represents `AF_ENCAP`.
    #[cfg(target_os = "openbsd")]
    pub const ENCAP: Self = Self(libc::AF_ENCAP);
    /// Represents `AF_FILE`.
    #[cfg(any(target_os = "illumos", target_os = "solaris"))]
    pub const FILE: Self = Self(libc::AF_FILE);
    /// Represents `AF_GOSIP`.
    #[cfg(solarish)]
    pub const GOSIP: Self = Self(libc::AF_GOSIP);
    /// Represents `AF_HYLINK`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike, solarish))]
    pub const HYLINK: Self = Self(libc::AF_HYLINK);
    /// Represents `AF_IB`.
    #[cfg(all(target_os = "linux", not(target_env = "uclibc")))]
    pub const IB: Self = Self(libc::AF_IB);
    /// Represents `AF_IEEE80211`.
    #[cfg(any(
        apple_targets,
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
    ))]
    pub const IEEE80211: Self = Self(libc::AF_IEEE80211);
    /// Represents `AF_IEEE802154`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const IEEE802154: Self = Self(libc::AF_IEEE802154);
    /// Represents `AF_IMPLINK`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike, solarish))]
    pub const IMPLINK: Self = Self(libc::AF_IMPLINK);
    /// Represents `AF_INET`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        linux_android,
        netbsdlike,
        solarish,
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "redox",
    ))]
    pub const INET: Self = Self(libc::AF_INET);
    /// Represents `AF_INET6`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        linux_android,
        netbsdlike,
        solarish,
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "redox",
    ))]
    pub const INET6: Self = Self(libc::AF_INET6);
    /// Represents `AF_INET6_SDP`.
    #[cfg(target_os = "freebsd")]
    pub const INET6_SDP: Self = Self(libc::AF_INET6_SDP);
    /// Represents `AF_INET_OFFLOAD`.
    #[cfg(solarish)]
    pub const INET_OFFLOAD: Self = Self(libc::AF_INET_OFFLOAD);
    /// Represents `AF_INET_SDP`.
    #[cfg(target_os = "freebsd")]
    pub const INET_SDP: Self = Self(libc::AF_INET_SDP);
    /// Represents `AF_IPX`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        linux_android,
        netbsdlike,
        solarish,
        target_os = "fuchsia",
        target_os = "haiku",
    ))]
    pub const IPX: Self = Self(libc::AF_IPX);
    /// Represents `AF_IRDA`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const IRDA: Self = Self(libc::AF_IRDA);
    /// Represents `AF_ISDN`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        linux_android,
        netbsdlike,
        target_os = "fuchsia",
    ))]
    pub const ISDN: Self = Self(libc::AF_ISDN);
    /// Represents `AF_ISO`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike))]
    pub const ISO: Self = Self(libc::AF_ISO);
    /// Represents `AF_IUCV`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const IUCV: Self = Self(libc::AF_IUCV);
    /// Represents `AF_KEY`.
    #[cfg(any(
        linux_android,
        solarish,
        target_os = "fuchsia",
        target_os = "openbsd",
    ))]
    pub const KEY: Self = Self(libc::AF_KEY);
    /// Represents `AF_LAT`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike, solarish))]
    pub const LAT: Self = Self(libc::AF_LAT);
    /// Represents `AF_LINK`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        netbsdlike,
        solarish,
        target_os = "haiku",
    ))]
    pub const LINK: Self = Self(libc::AF_LINK);
    /// Represents `AF_LLC`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const LLC: Self = Self(libc::AF_LLC);
    /// Represents `AF_LOCAL`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        linux_android,
        netbsdlike,
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "illumos",
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
    #[cfg(any(apple_targets, freebsdlike, netbsdlike))]
    pub const NATM: Self = Self(libc::AF_NATM);
    /// Represents `AF_NBS`.
    #[cfg(solarish)]
    pub const NBS: Self = Self(libc::AF_NBS);
    /// Represents `AF_NCA`.
    #[cfg(solarish)]
    pub const NCA: Self = Self(libc::AF_NCA);
    /// Represents `AF_NDRV`.
    #[cfg(apple_targets)]
    pub const NDRV: Self = Self(libc::AF_NDRV);
    /// Represents `AF_NETBEUI`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const NETBEUI: Self = Self(libc::AF_NETBEUI);
    /// Represents `AF_NETBIOS`.
    #[cfg(any(apple_targets, freebsdlike))]
    pub const NETBIOS: Self = Self(libc::AF_NETBIOS);
    /// Represents `AF_NETGRAPH`.
    #[cfg(freebsdlike)]
    pub const NETGRAPH: Self = Self(libc::AF_NETGRAPH);
    /// Represents `AF_NETLINK`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const NETLINK: Self = Self(libc::AF_NETLINK);
    /// Represents `AF_NETROM`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const NETROM: Self = Self(libc::AF_NETROM);
    /// Represents `AF_NFC`.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    pub const NFC: Self = Self(libc::AF_NFC);
    /// Represents `AF_NIT`.
    #[cfg(solarish)]
    pub const NIT: Self = Self(libc::AF_NIT);
    /// Represents `AF_NOTIFY`.
    #[cfg(target_os = "haiku")]
    pub const NOTIFY: Self = Self(libc::AF_NOTIFY);
    /// Represents `AF_NS`.
    #[cfg(any(apple_targets, netbsdlike, solarish))]
    pub const NS: Self = Self(libc::AF_NS);
    /// Represents `AF_OROUTE`.
    #[cfg(target_os = "netbsd")]
    pub const OROUTE: Self = Self(libc::AF_OROUTE);
    /// Represents `AF_OSI`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike, solarish))]
    pub const OSI: Self = Self(libc::AF_OSI);
    /// Represents `AF_OSINET`.
    #[cfg(solarish)]
    pub const OSINET: Self = Self(libc::AF_OSINET);
    /// Represents `AF_PACKET`.
    #[cfg(any(linux_android, solarish, target_os = "fuchsia"))]
    pub const PACKET: Self = Self(libc::AF_PACKET);
    /// Represents `AF_PHONET`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const PHONET: Self = Self(libc::AF_PHONET);
    /// Represents `AF_POLICY`.
    #[cfg(solarish)]
    pub const POLICY: Self = Self(libc::AF_POLICY);
    /// Represents `AF_PPP`.
    #[cfg(apple_targets)]
    pub const PPP: Self = Self(libc::AF_PPP);
    /// Represents `AF_PPPOX`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const PPPOX: Self = Self(libc::AF_PPPOX);
    /// Represents `AF_PUP`.
    #[cfg(any(apple_targets, freebsdlike, netbsdlike, solarish))]
    pub const PUP: Self = Self(libc::AF_PUP);
    /// Represents `AF_RDS`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const RDS: Self = Self(libc::AF_RDS);
    /// Represents `AF_ROSE`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const ROSE: Self = Self(libc::AF_ROSE);
    /// Represents `AF_ROUTE`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        linux_android,
        solarish,
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    pub const ROUTE: Self = Self(libc::AF_ROUTE);
    /// Represents `AF_RXRPC`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const RXRPC: Self = Self(libc::AF_RXRPC);
    /// Represents `AF_SCLUSTER`.
    #[cfg(target_os = "freebsd")]
    pub const SCLUSTER: Self = Self(libc::AF_SCLUSTER);
    /// Represents `AF_SECURITY`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const SECURITY: Self = Self(libc::AF_SECURITY);
    /// Represents `AF_SIP`.
    #[cfg(any(apple_targets, freebsdlike, target_os = "openbsd"))]
    pub const SIP: Self = Self(libc::AF_SIP);
    /// Represents `AF_SLOW`.
    #[cfg(target_os = "freebsd")]
    pub const SLOW: Self = Self(libc::AF_SLOW);
    /// Represents `AF_SNA`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        linux_android,
        netbsdlike,
        solarish,
        target_os = "fuchsia",
    ))]
    pub const SNA: Self = Self(libc::AF_SNA);
    /// Represents `AF_SYSTEM`.
    #[cfg(apple_targets)]
    pub const SYSTEM: Self = Self(libc::AF_SYSTEM);
    /// Represents `AF_SYS_CONTROL`.
    #[cfg(apple_targets)]
    pub const SYS_CONTROL: Self = Self(libc::AF_SYS_CONTROL);
    /// Represents `AF_TIPC`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const TIPC: Self = Self(libc::AF_TIPC);
    /// Represents `AF_TRILL`.
    #[cfg(solarish)]
    pub const TRILL: Self = Self(libc::AF_TRILL);
    /// Represents `AF_UNIX`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        linux_android,
        netbsdlike,
        solarish,
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "redox",
    ))]
    pub const UNIX: Self = Self(libc::AF_UNIX);
    /// Represents `AF_UNSPEC`.
    #[cfg(any(
        apple_targets,
        freebsdlike,
        linux_android,
        netbsdlike,
        solarish,
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "redox",
    ))]
    pub const UNSPEC: Self = Self(libc::AF_UNSPEC);
    /// Represents `AF_UTUN`.
    #[cfg(apple_targets)]
    pub const UTUN: Self = Self(libc::AF_UTUN);
    /// Represents `AF_VSOCK`.
    #[cfg(any(apple_targets, target_os = "android", target_os = "linux"))]
    pub const VSOCK: Self = Self(libc::AF_VSOCK);
    /// Represents `AF_WANPIPE`.
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub const WANPIPE: Self = Self(libc::AF_WANPIPE);
    /// Represents `AF_X25`.
    #[cfg(any(linux_android, solarish, target_os = "fuchsia"))]
    pub const X25: Self = Self(libc::AF_X25);
    /// Represents `AF_XDP`.
    #[cfg(all(target_os = "linux", not(target_env = "uclibc")))]
    pub const XDP: Self = Self(libc::AF_XDP);
}

/// A wrapper around `sockaddr_un`.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct UnixAddr {
    // INVARIANT: sun & sun_len are valid as defined by docs for from_raw_parts
    sun: libc::sockaddr_un,
    /// The length of the valid part of `sun`, including the sun_family field
    /// but excluding any trailing nul.
    // On the BSDs, this field is built into sun
    #[cfg(any(
        linux_android,
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "redox",
    ))]
    sun_len: u8,
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
    #[cfg(linux_android)]
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
        #[cfg(linux_android)]
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

impl UnixAddr {
    /// Create a new sockaddr_un representing a filesystem path.
    #[allow(clippy::unnecessary_cast)] // Not unnecessary on all platforms
    pub fn new<P: ?Sized + NixPath>(path: &P) -> Result<UnixAddr> {
        path.with_nix_path(|cstr| unsafe {
            let mut ret = libc::sockaddr_un {
                sun_family: AddressFamily::UNIX.family() as sa_family_t,
                ..mem::zeroed()
            };

            let bytes = cstr.to_bytes();

            if bytes.len() >= ret.sun_path.len() {
                return Err(Errno::ENAMETOOLONG);
            }

            let sun_len = (bytes.len()
                + offset_of!(libc::sockaddr_un, sun_path))
            .try_into()
            .unwrap();

            #[cfg(bsd)]
            {
                ret.sun_len = sun_len;
            }
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                ret.sun_path.as_mut_ptr().cast(),
                bytes.len(),
            );

            Ok(UnixAddr::from_raw_parts(ret, sun_len))
        })?
    }

    /// Create a new `sockaddr_un` representing an address in the "abstract namespace".
    ///
    /// The leading nul byte for the abstract namespace is automatically added;
    /// thus the input `path` is expected to be the bare name, not NUL-prefixed.
    /// This is a Linux-specific extension, primarily used to allow chrooted
    /// processes to communicate with processes having a different filesystem view.
    #[cfg(linux_android)]
    #[allow(clippy::unnecessary_cast)] // Not unnecessary on all platforms
    pub fn new_abstract(path: &[u8]) -> Result<UnixAddr> {
        unsafe {
            let mut ret = libc::sockaddr_un {
                sun_family: AddressFamily::UNIX.family() as sa_family_t,
                ..mem::zeroed()
            };

            if path.len() >= ret.sun_path.len() {
                return Err(Errno::ENAMETOOLONG);
            }
            let sun_len =
                (path.len() + 1 + offset_of!(libc::sockaddr_un, sun_path))
                    .try_into()
                    .unwrap();

            // Abstract addresses are represented by sun_path[0] ==
            // b'\0', so copy starting one byte in.
            ptr::copy_nonoverlapping(
                path.as_ptr(),
                ret.sun_path.as_mut_ptr().offset(1).cast(),
                path.len(),
            );

            Ok(UnixAddr::from_raw_parts(ret, sun_len))
        }
    }

    /// Create a new `sockaddr_un` representing an "unnamed" unix socket address.
    #[cfg(linux_android)]
    pub fn new_unnamed() -> UnixAddr {
        let ret = libc::sockaddr_un {
            sun_family: AddressFamily::UNIX.family() as sa_family_t,
            ..unsafe { mem::zeroed() }
        };

        let sun_len: u8 =
            offset_of!(libc::sockaddr_un, sun_path).try_into().unwrap();

        unsafe { UnixAddr::from_raw_parts(ret, sun_len) }
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
        sun_len: u8,
    ) -> UnixAddr {
        cfg_if! {
            if #[cfg(any(linux_android,
                     target_os = "fuchsia",
                     target_os = "illumos",
                     target_os = "redox",
                ))]
            {
                UnixAddr { sun, sun_len }
            } else {
                assert_eq!(sun_len, sun.sun_len);
                UnixAddr {sun}
            }
        }
    }

    fn kind(&self) -> UnixAddrKind<'_> {
        // SAFETY: our sockaddr is always valid because of the invariant on the struct
        unsafe { UnixAddrKind::get(&self.sun, self.sun_len()) }
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
    #[cfg(linux_android)]
    pub fn as_abstract(&self) -> Option<&[u8]> {
        match self.kind() {
            UnixAddrKind::Abstract(name) => Some(name),
            _ => None,
        }
    }

    /// Check if this address is an "unnamed" unix socket address.
    #[cfg(linux_android)]
    #[inline]
    pub fn is_unnamed(&self) -> bool {
        matches!(self.kind(), UnixAddrKind::Unnamed)
    }

    /// Returns the addrlen of this socket - `offsetof(struct sockaddr_un, sun_path)`
    #[inline]
    pub fn path_len(&self) -> usize {
        self.sun_len() as usize - offset_of!(libc::sockaddr_un, sun_path)
    }
    /// Returns a pointer to the raw `sockaddr_un` struct
    #[inline]
    pub fn as_ptr(&self) -> *const libc::sockaddr_un {
        &self.sun
    }
    /// Returns a mutable pointer to the raw `sockaddr_un` struct
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut libc::sockaddr_un {
        &mut self.sun
    }

    fn sun_len(&self) -> u8 {
        cfg_if! {
            if #[cfg(any(linux_android,
                     target_os = "fuchsia",
                     target_os = "illumos",
                     target_os = "redox",
                ))]
            {
                self.sun_len
            } else {
                self.sun.sun_len
            }
        }
    }
}

impl private::SockaddrLikePriv for UnixAddr {}
impl SockaddrLike for UnixAddr {
    #[cfg(any(linux_android, target_os = "fuchsia", target_os = "illumos"))]
    fn len(&self) -> libc::socklen_t {
        self.sun_len.into()
    }

    unsafe fn from_raw(
        addr: *const libc::sockaddr,
        len: Option<libc::socklen_t>,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(l) = len {
            if (l as usize) < offset_of!(libc::sockaddr_un, sun_path)
                || l > u8::MAX as libc::socklen_t
            {
                return None;
            }
        }
        if unsafe { (*addr).sa_family as i32 != libc::AF_UNIX } {
            return None;
        }
        let mut su: libc::sockaddr_un = unsafe { mem::zeroed() };
        let sup = &mut su as *mut libc::sockaddr_un as *mut u8;
        cfg_if! {
            if #[cfg(any(linux_android,
                         target_os = "fuchsia",
                         target_os = "illumos",
                         target_os = "redox",
                ))] {
                let su_len = len.unwrap_or(
                    mem::size_of::<libc::sockaddr_un>() as libc::socklen_t
                );
            } else {
                let su_len = unsafe { len.unwrap_or((*addr).sa_len as libc::socklen_t) };
            }
        }
        unsafe { ptr::copy(addr as *const u8, sup, su_len as usize) };
        Some(unsafe { Self::from_raw_parts(su, su_len as u8) })
    }

    fn size() -> libc::socklen_t
    where
        Self: Sized,
    {
        mem::size_of::<libc::sockaddr_un>() as libc::socklen_t
    }

    unsafe fn set_length(
        &mut self,
        new_length: usize,
    ) -> std::result::Result<(), SocketAddressLengthNotDynamic> {
        // `new_length` is only used on some platforms, so it must be provided even when not used
        #![allow(unused_variables)]
        cfg_if! {
            if #[cfg(any(linux_android,
                         target_os = "fuchsia",
                         target_os = "illumos",
                         target_os = "redox",
                ))] {
                self.sun_len = new_length as u8;
            }
        };
        Ok(())
    }
}

impl AsRef<libc::sockaddr_un> for UnixAddr {
    fn as_ref(&self) -> &libc::sockaddr_un {
        &self.sun
    }
}

#[cfg(linux_android)]
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

impl fmt::Display for UnixAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind() {
            UnixAddrKind::Pathname(path) => path.display().fmt(f),
            UnixAddrKind::Unnamed => f.pad("<unbound UNIX socket>"),
            #[cfg(linux_android)]
            UnixAddrKind::Abstract(name) => fmt_abstract(name, f),
        }
    }
}

impl PartialEq for UnixAddr {
    fn eq(&self, other: &UnixAddr) -> bool {
        self.kind() == other.kind()
    }
}

impl Eq for UnixAddr {}

impl Hash for UnixAddr {
    fn hash<H: Hasher>(&self, s: &mut H) {
        self.kind().hash(s)
    }
}

/// Anything that, in C, can be cast back and forth to `sockaddr`.
///
/// Most implementors also implement `AsRef<libc::XXX>` to access their
/// inner type read-only.
#[allow(clippy::len_without_is_empty)]
pub trait SockaddrLike: private::SockaddrLikePriv {
    /// Returns a raw pointer to the inner structure.  Useful for FFI.
    fn as_ptr(&self) -> *const libc::sockaddr {
        self as *const Self as *const libc::sockaddr
    }

    /// Unsafe constructor from a variable length source
    ///
    /// Some C APIs from provide `len`, and others do not.  If it's provided it
    /// will be validated.  If not, it will be guessed based on the family.
    ///
    /// # Arguments
    ///
    /// - `addr`:   raw pointer to something that can be cast to a
    ///             `libc::sockaddr`. For example, `libc::sockaddr_in`,
    ///             `libc::sockaddr_in6`, etc.
    /// - `len`:    For fixed-width types like `sockaddr_in`, it will be
    ///             validated if present and ignored if not.  For variable-width
    ///             types it is required and must be the total length of valid
    ///             data.  For example, if `addr` points to a
    ///             named `sockaddr_un`, then `len` must be the length of the
    ///             structure up to but not including the trailing NUL.
    ///
    /// # Safety
    ///
    /// `addr` must be valid for the specific type of sockaddr.  `len`, if
    /// present, must not exceed the length of valid data in `addr`.
    unsafe fn from_raw(
        addr: *const libc::sockaddr,
        len: Option<libc::socklen_t>,
    ) -> Option<Self>
    where
        Self: Sized;

    /// Return the address family of this socket
    ///
    /// # Examples
    /// One common use is to match on the family of a union type, like this:
    /// ```
    /// # use nix::sys::socket::*;
    /// # use std::os::unix::io::AsRawFd;
    /// let fd = socket(AddressFamily::INET, SockType::Stream,
    ///     SockFlag::empty(), None).unwrap();
    /// let ss: SockaddrStorage = getsockname(fd.as_raw_fd()).unwrap();
    /// match ss.family() {
    ///     AddressFamily::INET => println!("{}", ss.as_sockaddr_in().unwrap()),
    ///     AddressFamily::INET6 => println!("{}", ss.as_sockaddr_in6().unwrap()),
    ///     _ => println!("Unexpected address family")
    /// }
    /// ```
    fn family(&self) -> AddressFamily {
        // SAFETY: `&self` must be castable to a `libc::sockaddr` pointer by contract.
        AddressFamily::of(unsafe { &*self.as_ptr() })
    }

    cfg_if! {
        if #[cfg(bsd)] {
            /// Return the length of valid data in the sockaddr structure.
            ///
            /// For fixed-size sockaddrs, this should be the size of the
            /// structure.  But for variable-sized types like [`UnixAddr`] it
            /// may be less.
            fn len(&self) -> libc::socklen_t {
                // Safe since all implementors have a sa_len field at the same
                // address, and they're all repr(transparent).
                // Robust for all implementors.
                unsafe {
                    (*(self as *const Self as *const libc::sockaddr)).sa_len
                }.into()
            }
        } else {
            /// Return the length of valid data in the sockaddr structure.
            ///
            /// For fixed-size sockaddrs, this should be the size of the
            /// structure.  But for variable-sized types like [`UnixAddr`] it
            /// may be less.
            fn len(&self) -> libc::socklen_t {
                // No robust default implementation is possible without an
                // sa_len field.  Implementors with a variable size must
                // override this method.
                mem::size_of_val(self) as libc::socklen_t
            }
        }
    }

    /// Return the available space in the structure
    fn size() -> libc::socklen_t
    where
        Self: Sized,
    {
        mem::size_of::<Self>() as libc::socklen_t
    }

    /// Set the length of this socket address
    ///
    /// This method may only be called on socket addresses whose lengths are dynamic, and it
    /// returns an error if called on a type whose length is static.
    ///
    /// # Safety
    ///
    /// `new_length` must be a valid length for this type of address. Specifically, reads of that
    /// length from `self` must be valid.
    #[doc(hidden)]
    unsafe fn set_length(
        &mut self,
        _new_length: usize,
    ) -> std::result::Result<(), SocketAddressLengthNotDynamic> {
        Err(SocketAddressLengthNotDynamic)
    }
}

/// The error returned by [`SockaddrLike::set_length`] on an address whose length is statically
/// fixed.
#[derive(Copy, Clone, Debug)]
pub struct SocketAddressLengthNotDynamic;
impl fmt::Display for SocketAddressLengthNotDynamic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Attempted to set length on socket whose length is statically fixed")
    }
}
impl std::error::Error for SocketAddressLengthNotDynamic {}

impl private::SockaddrLikePriv for () {
    fn as_mut_ptr(&mut self) -> *mut libc::sockaddr {
        ptr::null_mut()
    }
}

/// `()` can be used in place of a real Sockaddr when no address is expected,
/// for example for a field of `Option<S> where S: SockaddrLike`.
// If this RFC ever stabilizes, then ! will be a better choice.
// https://github.com/rust-lang/rust/issues/35121
impl SockaddrLike for () {
    fn as_ptr(&self) -> *const libc::sockaddr {
        ptr::null()
    }

    unsafe fn from_raw(
        _: *const libc::sockaddr,
        _: Option<libc::socklen_t>,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    fn family(&self) -> AddressFamily {
        AddressFamily::UNSPEC
    }

    fn len(&self) -> libc::socklen_t {
        0
    }
}

/// An IPv4 socket address
#[cfg(feature = "net")]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SockaddrIn(libc::sockaddr_in);

#[cfg(feature = "net")]
impl SockaddrIn {
    /// Returns the IP address associated with this socket address, in native
    /// endian.
    pub const fn ip(&self) -> net::Ipv4Addr {
        let bytes = self.0.sin_addr.s_addr.to_ne_bytes();
        let (a, b, c, d) = (bytes[0], bytes[1], bytes[2], bytes[3]);
        Ipv4Addr::new(a, b, c, d)
    }

    /// Creates a new socket address from IPv4 octets and a port number.
    pub fn new(a: u8, b: u8, c: u8, d: u8, port: u16) -> Self {
        Self(libc::sockaddr_in {
            #[cfg(any(bsd, target_os = "aix", target_os = "haiku"))]
            sin_len: Self::size() as u8,
            sin_family: AddressFamily::INET.family() as sa_family_t,
            sin_port: u16::to_be(port),
            sin_addr: libc::in_addr {
                s_addr: u32::from_ne_bytes([a, b, c, d]),
            },
            sin_zero: unsafe { mem::zeroed() },
        })
    }

    /// Returns the port number associated with this socket address, in native
    /// endian.
    pub const fn port(&self) -> u16 {
        u16::from_be(self.0.sin_port)
    }
}

#[cfg(feature = "net")]
impl private::SockaddrLikePriv for SockaddrIn {}
#[cfg(feature = "net")]
impl SockaddrLike for SockaddrIn {
    unsafe fn from_raw(
        addr: *const libc::sockaddr,
        len: Option<libc::socklen_t>,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(l) = len {
            if l != mem::size_of::<libc::sockaddr_in>() as libc::socklen_t {
                return None;
            }
        }
        if unsafe { (*addr).sa_family as i32 != libc::AF_INET } {
            return None;
        }
        Some(Self(unsafe { ptr::read_unaligned(addr as *const _) }))
    }
}

#[cfg(feature = "net")]
impl AsRef<libc::sockaddr_in> for SockaddrIn {
    fn as_ref(&self) -> &libc::sockaddr_in {
        &self.0
    }
}

#[cfg(feature = "net")]
impl fmt::Display for SockaddrIn {
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
impl From<net::SocketAddrV4> for SockaddrIn {
    fn from(addr: net::SocketAddrV4) -> Self {
        Self(libc::sockaddr_in {
            #[cfg(any(bsd, target_os = "haiku", target_os = "hermit"))]
            sin_len: mem::size_of::<libc::sockaddr_in>() as u8,
            sin_family: AddressFamily::INET.family() as sa_family_t,
            sin_port: addr.port().to_be(), // network byte order
            sin_addr: ipv4addr_to_libc(*addr.ip()),
            ..unsafe { mem::zeroed() }
        })
    }
}

#[cfg(feature = "net")]
impl From<SockaddrIn> for net::SocketAddrV4 {
    fn from(addr: SockaddrIn) -> Self {
        net::SocketAddrV4::new(
            net::Ipv4Addr::from(addr.0.sin_addr.s_addr.to_ne_bytes()),
            u16::from_be(addr.0.sin_port),
        )
    }
}

#[cfg(feature = "net")]
impl std::str::FromStr for SockaddrIn {
    type Err = net::AddrParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        net::SocketAddrV4::from_str(s).map(SockaddrIn::from)
    }
}

/// An IPv6 socket address
#[cfg(feature = "net")]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SockaddrIn6(libc::sockaddr_in6);

#[cfg(feature = "net")]
impl SockaddrIn6 {
    /// Returns the flow information associated with this address.
    pub const fn flowinfo(&self) -> u32 {
        self.0.sin6_flowinfo
    }

    /// Returns the IP address associated with this socket address.
    pub const fn ip(&self) -> net::Ipv6Addr {
        let bytes = self.0.sin6_addr.s6_addr;
        let (a, b, c, d, e, f, g, h) = (
            ((bytes[0] as u16) << 8) | bytes[1] as u16,
            ((bytes[2] as u16) << 8) | bytes[3] as u16,
            ((bytes[4] as u16) << 8) | bytes[5] as u16,
            ((bytes[6] as u16) << 8) | bytes[7] as u16,
            ((bytes[8] as u16) << 8) | bytes[9] as u16,
            ((bytes[10] as u16) << 8) | bytes[11] as u16,
            ((bytes[12] as u16) << 8) | bytes[13] as u16,
            ((bytes[14] as u16) << 8) | bytes[15] as u16,
        );
        Ipv6Addr::new(a, b, c, d, e, f, g, h)
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
}

#[cfg(feature = "net")]
impl private::SockaddrLikePriv for SockaddrIn6 {}
#[cfg(feature = "net")]
impl SockaddrLike for SockaddrIn6 {
    unsafe fn from_raw(
        addr: *const libc::sockaddr,
        len: Option<libc::socklen_t>,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(l) = len {
            if l != mem::size_of::<libc::sockaddr_in6>() as libc::socklen_t {
                return None;
            }
        }
        if unsafe { (*addr).sa_family as i32 != libc::AF_INET6 } {
            return None;
        }
        Some(Self(unsafe { ptr::read_unaligned(addr as *const _) }))
    }
}

#[cfg(feature = "net")]
impl AsRef<libc::sockaddr_in6> for SockaddrIn6 {
    fn as_ref(&self) -> &libc::sockaddr_in6 {
        &self.0
    }
}

#[cfg(feature = "net")]
impl fmt::Display for SockaddrIn6 {
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
impl From<net::SocketAddrV6> for SockaddrIn6 {
    fn from(addr: net::SocketAddrV6) -> Self {
        #[allow(clippy::needless_update)] // It isn't needless on Illumos
        Self(libc::sockaddr_in6 {
            #[cfg(any(bsd, target_os = "haiku", target_os = "hermit"))]
            sin6_len: mem::size_of::<libc::sockaddr_in6>() as u8,
            sin6_family: AddressFamily::INET6.family() as sa_family_t,
            sin6_port: addr.port().to_be(), // network byte order
            sin6_addr: ipv6addr_to_libc(addr.ip()),
            sin6_flowinfo: addr.flowinfo(), // host byte order
            sin6_scope_id: addr.scope_id(), // host byte order
            ..unsafe { mem::zeroed() }
        })
    }
}

#[cfg(feature = "net")]
impl From<SockaddrIn6> for net::SocketAddrV6 {
    fn from(addr: SockaddrIn6) -> Self {
        net::SocketAddrV6::new(
            net::Ipv6Addr::from(addr.0.sin6_addr.s6_addr),
            u16::from_be(addr.0.sin6_port),
            addr.0.sin6_flowinfo,
            addr.0.sin6_scope_id,
        )
    }
}

#[cfg(feature = "net")]
impl std::str::FromStr for SockaddrIn6 {
    type Err = net::AddrParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        net::SocketAddrV6::from_str(s).map(SockaddrIn6::from)
    }
}

/// A container for any sockaddr type
///
/// Just like C's `sockaddr_storage`, this type is large enough to hold any type
/// of sockaddr.  It can be used as an argument with functions like
/// [`bind`](super::bind) and [`getsockname`](super::getsockname).  Though it is
/// a union, it can be safely accessed through the `as_*` methods.
///
/// # Example
/// ```
/// # use nix::sys::socket::*;
/// # use std::str::FromStr;
/// # use std::os::unix::io::AsRawFd;
/// let localhost = SockaddrIn::from_str("127.0.0.1:8081").unwrap();
/// let fd = socket(AddressFamily::INET, SockType::Stream, SockFlag::empty(),
///     None).unwrap();
/// bind(fd.as_raw_fd(), &localhost).expect("bind");
/// let ss: SockaddrStorage = getsockname(fd.as_raw_fd()).expect("getsockname");
/// assert_eq!(&localhost, ss.as_sockaddr_in().unwrap());
/// ```
#[derive(Clone, Copy, Eq)]
#[repr(C)]
pub union SockaddrStorage {
    #[cfg(linux_android)]
    alg: AlgAddr,
    #[cfg(all(feature = "net", not(target_os = "redox")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "net")))]
    dl: LinkAddr,
    #[cfg(linux_android)]
    nl: NetlinkAddr,
    #[cfg(all(feature = "ioctl", apple_targets))]
    #[cfg_attr(docsrs, doc(cfg(feature = "ioctl")))]
    sctl: SysControlAddr,
    #[cfg(feature = "net")]
    sin: SockaddrIn,
    #[cfg(feature = "net")]
    sin6: SockaddrIn6,
    ss: libc::sockaddr_storage,
    su: UnixAddr,
    #[cfg(any(linux_android, target_os = "macos"))]
    vsock: VsockAddr,
}
impl private::SockaddrLikePriv for SockaddrStorage {}
impl SockaddrLike for SockaddrStorage {
    unsafe fn from_raw(
        addr: *const libc::sockaddr,
        l: Option<libc::socklen_t>,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        if addr.is_null() {
            return None;
        }
        if let Some(len) = l {
            let ulen = len as usize;
            if ulen < offset_of!(libc::sockaddr, sa_data)
                || ulen > mem::size_of::<libc::sockaddr_storage>()
            {
                None
            } else {
                let mut ss: libc::sockaddr_storage = unsafe { mem::zeroed() };
                let ssp = &mut ss as *mut libc::sockaddr_storage as *mut u8;
                unsafe { ptr::copy(addr as *const u8, ssp, len as usize) };
                #[cfg(any(
                    linux_android,
                    target_os = "fuchsia",
                    target_os = "illumos",
                ))]
                if i32::from(ss.ss_family) == libc::AF_UNIX {
                    // Safe because we UnixAddr is strictly smaller than
                    // SockaddrStorage, and we just initialized the structure.
                    unsafe {
                        (*(&mut ss as *mut libc::sockaddr_storage
                            as *mut UnixAddr))
                            .sun_len = len as u8;
                    }
                }
                Some(Self { ss })
            }
        } else {
            // If length is not available and addr is of a fixed-length type,
            // copy it.  If addr is of a variable length type and len is not
            // available, then there's nothing we can do.
            match unsafe { (*addr).sa_family as i32 } {
                #[cfg(linux_android)]
                libc::AF_ALG => unsafe {
                    AlgAddr::from_raw(addr, l).map(|alg| Self { alg })
                },
                #[cfg(feature = "net")]
                libc::AF_INET => unsafe {
                    SockaddrIn::from_raw(addr, l).map(|sin| Self { sin })
                },
                #[cfg(feature = "net")]
                libc::AF_INET6 => unsafe {
                    SockaddrIn6::from_raw(addr, l).map(|sin6| Self { sin6 })
                },
                #[cfg(any(bsd, target_os = "illumos", target_os = "haiku"))]
                #[cfg(feature = "net")]
                libc::AF_LINK => unsafe {
                    LinkAddr::from_raw(addr, l).map(|dl| Self { dl })
                },
                #[cfg(linux_android)]
                libc::AF_NETLINK => unsafe {
                    NetlinkAddr::from_raw(addr, l).map(|nl| Self { nl })
                },
                #[cfg(any(linux_android, target_os = "fuchsia"))]
                #[cfg(feature = "net")]
                libc::AF_PACKET => unsafe {
                    LinkAddr::from_raw(addr, l).map(|dl| Self { dl })
                },
                #[cfg(all(feature = "ioctl", apple_targets))]
                libc::AF_SYSTEM => unsafe {
                    SysControlAddr::from_raw(addr, l).map(|sctl| Self { sctl })
                },
                #[cfg(any(linux_android, target_os = "macos"))]
                libc::AF_VSOCK => unsafe {
                    VsockAddr::from_raw(addr, l).map(|vsock| Self { vsock })
                },
                _ => None,
            }
        }
    }

    #[cfg(any(linux_android, target_os = "fuchsia", target_os = "illumos"))]
    fn len(&self) -> libc::socklen_t {
        match self.as_unix_addr() {
            // The UnixAddr type knows its own length
            Some(ua) => ua.len(),
            // For all else, we're just a boring SockaddrStorage
            None => mem::size_of_val(self) as libc::socklen_t,
        }
    }

    unsafe fn set_length(
        &mut self,
        new_length: usize,
    ) -> std::result::Result<(), SocketAddressLengthNotDynamic> {
        match self.as_unix_addr_mut() {
            Some(addr) => unsafe { addr.set_length(new_length) },
            None => Err(SocketAddressLengthNotDynamic),
        }
    }
}

macro_rules! accessors {
    (
        $fname:ident,
        $fname_mut:ident,
        $sockty:ty,
        $family:expr,
        $libc_ty:ty,
        $field:ident) => {
        /// Safely and falliably downcast to an immutable reference
        pub fn $fname(&self) -> Option<&$sockty> {
            if self.family() == $family
                && self.len() >= mem::size_of::<$libc_ty>() as libc::socklen_t
            {
                // Safe because family and len are validated
                Some(unsafe { &self.$field })
            } else {
                None
            }
        }

        /// Safely and falliably downcast to a mutable reference
        pub fn $fname_mut(&mut self) -> Option<&mut $sockty> {
            if self.family() == $family
                && self.len() >= mem::size_of::<$libc_ty>() as libc::socklen_t
            {
                // Safe because family and len are validated
                Some(unsafe { &mut self.$field })
            } else {
                None
            }
        }
    };
}

impl SockaddrStorage {
    /// Downcast to an immutable `[UnixAddr]` reference.
    pub fn as_unix_addr(&self) -> Option<&UnixAddr> {
        cfg_if! {
            if #[cfg(any(linux_android,
                     target_os = "fuchsia",
                     target_os = "illumos",
                ))]
            {
                let p = unsafe{ &self.ss as *const libc::sockaddr_storage };
                // Safe because UnixAddr is strictly smaller than
                // sockaddr_storage, and we're fully initialized
                let len = unsafe {
                    (*(p as *const UnixAddr )).sun_len as usize
                };
            } else {
                let len = self.len() as usize;
            }
        }
        // Sanity checks
        if self.family() != AddressFamily::UNIX
            || len < offset_of!(libc::sockaddr_un, sun_path)
            || len > mem::size_of::<libc::sockaddr_un>()
        {
            None
        } else {
            Some(unsafe { &self.su })
        }
    }

    /// Downcast to a mutable `[UnixAddr]` reference.
    pub fn as_unix_addr_mut(&mut self) -> Option<&mut UnixAddr> {
        cfg_if! {
            if #[cfg(any(linux_android,
                     target_os = "fuchsia",
                     target_os = "illumos",
                ))]
            {
                let p = unsafe{ &self.ss as *const libc::sockaddr_storage };
                // Safe because UnixAddr is strictly smaller than
                // sockaddr_storage, and we're fully initialized
                let len = unsafe {
                    (*(p as *const UnixAddr )).sun_len as usize
                };
            } else {
                let len = self.len() as usize;
            }
        }
        // Sanity checks
        if self.family() != AddressFamily::UNIX
            || len < offset_of!(libc::sockaddr_un, sun_path)
            || len > mem::size_of::<libc::sockaddr_un>()
        {
            None
        } else {
            Some(unsafe { &mut self.su })
        }
    }

    #[cfg(linux_android)]
    accessors! {as_alg_addr, as_alg_addr_mut, AlgAddr,
    AddressFamily::ALG, libc::sockaddr_alg, alg}

    #[cfg(any(linux_android, target_os = "fuchsia"))]
    #[cfg(feature = "net")]
    accessors! {
    as_link_addr, as_link_addr_mut, LinkAddr,
    AddressFamily::PACKET, libc::sockaddr_ll, dl}

    #[cfg(any(bsd, target_os = "illumos"))]
    #[cfg(feature = "net")]
    accessors! {
    as_link_addr, as_link_addr_mut, LinkAddr,
    AddressFamily::LINK, libc::sockaddr_dl, dl}

    #[cfg(feature = "net")]
    accessors! {
    as_sockaddr_in, as_sockaddr_in_mut, SockaddrIn,
    AddressFamily::INET, libc::sockaddr_in, sin}

    #[cfg(feature = "net")]
    accessors! {
    as_sockaddr_in6, as_sockaddr_in6_mut, SockaddrIn6,
    AddressFamily::INET6, libc::sockaddr_in6, sin6}

    #[cfg(linux_android)]
    accessors! {as_netlink_addr, as_netlink_addr_mut, NetlinkAddr,
    AddressFamily::NETLINK, libc::sockaddr_nl, nl}

    #[cfg(all(feature = "ioctl", apple_targets))]
    #[cfg_attr(docsrs, doc(cfg(feature = "ioctl")))]
    accessors! {as_sys_control_addr, as_sys_control_addr_mut, SysControlAddr,
    AddressFamily::SYSTEM, libc::sockaddr_ctl, sctl}

    #[cfg(any(linux_android, target_os = "macos"))]
    accessors! {as_vsock_addr, as_vsock_addr_mut, VsockAddr,
    AddressFamily::VSOCK, libc::sockaddr_vm, vsock}
}

impl fmt::Debug for SockaddrStorage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SockaddrStorage")
            // Safe because sockaddr_storage has the least specific
            // field types
            .field("ss", unsafe { &self.ss })
            .finish()
    }
}

impl fmt::Display for SockaddrStorage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            match self.ss.ss_family as i32 {
                #[cfg(linux_android)]
                libc::AF_ALG => self.alg.fmt(f),
                #[cfg(feature = "net")]
                libc::AF_INET => self.sin.fmt(f),
                #[cfg(feature = "net")]
                libc::AF_INET6 => self.sin6.fmt(f),
                #[cfg(any(bsd, target_os = "illumos"))]
                #[cfg(feature = "net")]
                libc::AF_LINK => self.dl.fmt(f),
                #[cfg(linux_android)]
                libc::AF_NETLINK => self.nl.fmt(f),
                #[cfg(any(linux_android, target_os = "fuchsia"))]
                #[cfg(feature = "net")]
                libc::AF_PACKET => self.dl.fmt(f),
                #[cfg(apple_targets)]
                #[cfg(feature = "ioctl")]
                libc::AF_SYSTEM => self.sctl.fmt(f),
                libc::AF_UNIX => self.su.fmt(f),
                #[cfg(any(linux_android, target_os = "macos"))]
                libc::AF_VSOCK => self.vsock.fmt(f),
                _ => "<Address family unspecified>".fmt(f),
            }
        }
    }
}

#[cfg(feature = "net")]
impl From<net::SocketAddrV4> for SockaddrStorage {
    fn from(s: net::SocketAddrV4) -> Self {
        unsafe {
            let mut ss: Self = mem::zeroed();
            ss.sin = SockaddrIn::from(s);
            ss
        }
    }
}

#[cfg(feature = "net")]
impl From<net::SocketAddrV6> for SockaddrStorage {
    fn from(s: net::SocketAddrV6) -> Self {
        unsafe {
            let mut ss: Self = mem::zeroed();
            ss.sin6 = SockaddrIn6::from(s);
            ss
        }
    }
}

#[cfg(feature = "net")]
impl From<net::SocketAddr> for SockaddrStorage {
    fn from(s: net::SocketAddr) -> Self {
        match s {
            net::SocketAddr::V4(sa4) => Self::from(sa4),
            net::SocketAddr::V6(sa6) => Self::from(sa6),
        }
    }
}

impl Hash for SockaddrStorage {
    fn hash<H: Hasher>(&self, s: &mut H) {
        unsafe {
            match self.ss.ss_family as i32 {
                #[cfg(linux_android)]
                libc::AF_ALG => self.alg.hash(s),
                #[cfg(feature = "net")]
                libc::AF_INET => self.sin.hash(s),
                #[cfg(feature = "net")]
                libc::AF_INET6 => self.sin6.hash(s),
                #[cfg(any(bsd, target_os = "illumos"))]
                #[cfg(feature = "net")]
                libc::AF_LINK => self.dl.hash(s),
                #[cfg(linux_android)]
                libc::AF_NETLINK => self.nl.hash(s),
                #[cfg(any(linux_android, target_os = "fuchsia"))]
                #[cfg(feature = "net")]
                libc::AF_PACKET => self.dl.hash(s),
                #[cfg(apple_targets)]
                #[cfg(feature = "ioctl")]
                libc::AF_SYSTEM => self.sctl.hash(s),
                libc::AF_UNIX => self.su.hash(s),
                #[cfg(any(linux_android, target_os = "macos"))]
                libc::AF_VSOCK => self.vsock.hash(s),
                _ => self.ss.hash(s),
            }
        }
    }
}

impl PartialEq for SockaddrStorage {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            match (self.ss.ss_family as i32, other.ss.ss_family as i32) {
                #[cfg(linux_android)]
                (libc::AF_ALG, libc::AF_ALG) => self.alg == other.alg,
                #[cfg(feature = "net")]
                (libc::AF_INET, libc::AF_INET) => self.sin == other.sin,
                #[cfg(feature = "net")]
                (libc::AF_INET6, libc::AF_INET6) => self.sin6 == other.sin6,
                #[cfg(any(bsd, target_os = "illumos"))]
                #[cfg(feature = "net")]
                (libc::AF_LINK, libc::AF_LINK) => self.dl == other.dl,
                #[cfg(linux_android)]
                (libc::AF_NETLINK, libc::AF_NETLINK) => self.nl == other.nl,
                #[cfg(any(linux_android, target_os = "fuchsia"))]
                #[cfg(feature = "net")]
                (libc::AF_PACKET, libc::AF_PACKET) => self.dl == other.dl,
                #[cfg(apple_targets)]
                #[cfg(feature = "ioctl")]
                (libc::AF_SYSTEM, libc::AF_SYSTEM) => self.sctl == other.sctl,
                (libc::AF_UNIX, libc::AF_UNIX) => self.su == other.su,
                #[cfg(any(linux_android, target_os = "macos"))]
                (libc::AF_VSOCK, libc::AF_VSOCK) => self.vsock == other.vsock,
                _ => false,
            }
        }
    }
}

pub(super) mod private {
    pub trait SockaddrLikePriv {
        /// Returns a mutable raw pointer to the inner structure.
        ///
        /// # Safety
        ///
        /// This method is technically safe, but modifying the inner structure's
        /// `family` or `len` fields may result in violating Nix's invariants.
        /// It is best to use this method only with foreign functions that do
        /// not change the sockaddr type.
        fn as_mut_ptr(&mut self) -> *mut libc::sockaddr {
            self as *mut Self as *mut libc::sockaddr
        }
    }
}

#[cfg(linux_android)]
pub mod netlink {
    use super::*;
    use crate::sys::socket::addr::AddressFamily;
    use libc::{sa_family_t, sockaddr_nl};
    use std::{fmt, mem};

    /// Address for the Linux kernel user interface device.
    ///
    /// # References
    ///
    /// [netlink(7)](https://man7.org/linux/man-pages/man7/netlink.7.html)
    #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
    #[repr(transparent)]
    pub struct NetlinkAddr(pub(in super::super) sockaddr_nl);

    impl NetlinkAddr {
        /// Construct a new socket address from its port ID and multicast groups
        /// mask.
        pub fn new(pid: u32, groups: u32) -> NetlinkAddr {
            let mut addr: sockaddr_nl = unsafe { mem::zeroed() };
            addr.nl_family = AddressFamily::NETLINK.family() as sa_family_t;
            addr.nl_pid = pid;
            addr.nl_groups = groups;

            NetlinkAddr(addr)
        }

        /// Return the socket's port ID.
        pub const fn pid(&self) -> u32 {
            self.0.nl_pid
        }

        /// Return the socket's multicast groups mask
        pub const fn groups(&self) -> u32 {
            self.0.nl_groups
        }
    }

    impl private::SockaddrLikePriv for NetlinkAddr {}
    impl SockaddrLike for NetlinkAddr {
        unsafe fn from_raw(
            addr: *const libc::sockaddr,
            len: Option<libc::socklen_t>,
        ) -> Option<Self>
        where
            Self: Sized,
        {
            if let Some(l) = len {
                if l != mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t {
                    return None;
                }
            }
            if unsafe { (*addr).sa_family as i32 != libc::AF_NETLINK } {
                return None;
            }
            Some(Self(unsafe { ptr::read_unaligned(addr as *const _) }))
        }
    }

    impl AsRef<libc::sockaddr_nl> for NetlinkAddr {
        fn as_ref(&self) -> &libc::sockaddr_nl {
            &self.0
        }
    }

    impl fmt::Display for NetlinkAddr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "pid: {} groups: {}", self.pid(), self.groups())
        }
    }
}

#[cfg(linux_android)]
pub mod alg {
    use super::*;
    use libc::{sockaddr_alg, AF_ALG};
    use std::ffi::CStr;
    use std::hash::{Hash, Hasher};
    use std::{fmt, mem, str};

    /// Socket address for the Linux kernel crypto API
    #[derive(Copy, Clone)]
    #[repr(transparent)]
    pub struct AlgAddr(pub(in super::super) sockaddr_alg);

    impl private::SockaddrLikePriv for AlgAddr {}
    impl SockaddrLike for AlgAddr {
        unsafe fn from_raw(
            addr: *const libc::sockaddr,
            l: Option<libc::socklen_t>,
        ) -> Option<Self>
        where
            Self: Sized,
        {
            if let Some(l) = l {
                if l != mem::size_of::<libc::sockaddr_alg>() as libc::socklen_t
                {
                    return None;
                }
            }
            if unsafe { (*addr).sa_family as i32 != libc::AF_ALG } {
                return None;
            }
            Some(Self(unsafe { ptr::read_unaligned(addr as *const _) }))
        }
    }

    impl AsRef<libc::sockaddr_alg> for AlgAddr {
        fn as_ref(&self) -> &libc::sockaddr_alg {
            &self.0
        }
    }

    // , PartialEq, Eq, Debug, Hash
    impl PartialEq for AlgAddr {
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

    impl Eq for AlgAddr {}

    impl Hash for AlgAddr {
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

    impl AlgAddr {
        /// Construct an `AF_ALG` socket from its cipher name and type.
        pub fn new(alg_type: &str, alg_name: &str) -> AlgAddr {
            let mut addr: sockaddr_alg = unsafe { mem::zeroed() };
            addr.salg_family = AF_ALG as u16;
            addr.salg_type[..alg_type.len()]
                .copy_from_slice(alg_type.to_string().as_bytes());
            addr.salg_name[..alg_name.len()]
                .copy_from_slice(alg_name.to_string().as_bytes());

            AlgAddr(addr)
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

    impl fmt::Display for AlgAddr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "type: {} alg: {}",
                self.alg_name().to_string_lossy(),
                self.alg_type().to_string_lossy()
            )
        }
    }

    impl fmt::Debug for AlgAddr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(self, f)
        }
    }
}

feature! {
#![feature = "ioctl"]
#[cfg(apple_targets)]
pub mod sys_control {
    use crate::sys::socket::addr::AddressFamily;
    use libc::{self, c_uchar};
    use std::{fmt, mem, ptr};
    use std::os::unix::io::RawFd;
    use crate::{Errno, Result};
    use super::{private, SockaddrLike};

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
    pub struct SysControlAddr(pub(in super::super) libc::sockaddr_ctl);

    impl private::SockaddrLikePriv for SysControlAddr {}
    impl SockaddrLike for SysControlAddr {
        unsafe fn from_raw(addr: *const libc::sockaddr, len: Option<libc::socklen_t>)
            -> Option<Self> where Self: Sized
        {
            if let Some(l) = len {
                if l != mem::size_of::<libc::sockaddr_ctl>() as libc::socklen_t {
                    return None;
                }
            }
            if unsafe { (*addr).sa_family as i32 != libc::AF_SYSTEM } {
                return None;
            }
            Some(Self(unsafe { ptr::read_unaligned(addr as *const _) } ))
        }
    }

    impl AsRef<libc::sockaddr_ctl> for SysControlAddr {
        fn as_ref(&self) -> &libc::sockaddr_ctl {
            &self.0
        }
    }

    impl SysControlAddr {
        /// Construct a new `SysControlAddr` from its kernel unique identifier
        /// and unit number.
        pub const fn new(id: u32, unit: u32) -> SysControlAddr {
            let addr = libc::sockaddr_ctl {
                sc_len: mem::size_of::<libc::sockaddr_ctl>() as c_uchar,
                sc_family: AddressFamily::SYSTEM.family() as c_uchar,
                ss_sysaddr: libc::AF_SYS_CONTROL as u16,
                sc_id: id,
                sc_unit: unit,
                sc_reserved: [0; 5]
            };

            SysControlAddr(addr)
        }

        /// Construct a new `SysControlAddr` from its human readable name and
        /// unit number.
        pub fn from_name(sockfd: RawFd, name: &str, unit: u32) -> Result<SysControlAddr> {
            if name.len() > MAX_KCTL_NAME {
                return Err(Errno::ENAMETOOLONG);
            }

            let mut ctl_name = [0; MAX_KCTL_NAME];
            ctl_name[..name.len()].clone_from_slice(name.as_bytes());
            let mut info = ctl_ioc_info { ctl_id: 0, ctl_name };

            unsafe { ctl_info(sockfd, &mut info)?; }

            Ok(SysControlAddr::new(info.ctl_id, unit))
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

    impl fmt::Display for SysControlAddr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Debug::fmt(self, f)
        }
    }
}
}

#[cfg(any(linux_android, target_os = "fuchsia"))]
mod datalink {
    feature! {
    #![feature = "net"]
    use super::{fmt, mem, private, ptr, SockaddrLike};

    /// Hardware Address
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    #[repr(transparent)]
    pub struct LinkAddr(pub(in super::super) libc::sockaddr_ll);

    impl LinkAddr {
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
    }

    impl fmt::Display for LinkAddr {
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
    impl private::SockaddrLikePriv for LinkAddr {}
    impl SockaddrLike for LinkAddr {
        unsafe fn from_raw(addr: *const libc::sockaddr,
                           len: Option<libc::socklen_t>)
            -> Option<Self> where Self: Sized
        {
            if let Some(l) = len {
                if l != mem::size_of::<libc::sockaddr_ll>() as libc::socklen_t {
                    return None;
                }
            }
            if unsafe { (*addr).sa_family as i32 != libc::AF_PACKET } {
                return None;
            }
            Some(Self(unsafe { ptr::read_unaligned(addr as *const _) }))
        }
    }

    impl AsRef<libc::sockaddr_ll> for LinkAddr {
        fn as_ref(&self) -> &libc::sockaddr_ll {
            &self.0
        }
    }

    }
}

#[cfg(any(bsd, target_os = "illumos", target_os = "haiku", target_os = "aix"))]
mod datalink {
    feature! {
    #![feature = "net"]
    use super::{fmt, mem, private, ptr, SockaddrLike};

    /// Hardware Address
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    #[repr(transparent)]
    pub struct LinkAddr(pub(in super::super) libc::sockaddr_dl);

    impl LinkAddr {
        /// interface index, if != 0, system given index for interface
        #[cfg(not(target_os = "haiku"))]
        pub fn ifindex(&self) -> usize {
            self.0.sdl_index as usize
        }

        /// Datalink type
        #[cfg(not(target_os = "haiku"))]
        pub fn datalink_type(&self) -> u8 {
            self.0.sdl_type
        }

        /// MAC address start position
        pub fn nlen(&self) -> usize {
            self.0.sdl_nlen as usize
        }

        /// link level address length
        pub fn alen(&self) -> usize {
            self.0.sdl_alen as usize
        }

        /// link layer selector length
        #[cfg(not(target_os = "haiku"))]
        pub fn slen(&self) -> usize {
            self.0.sdl_slen as usize
        }

        /// if link level address length == 0,
        /// or `sdl_data` not be larger.
        pub fn is_empty(&self) -> bool {
            let nlen = self.nlen();
            let alen = self.alen();
            let data_len = self.0.sdl_data.len();

            alen == 0 || nlen + alen >= data_len
        }

        /// Physical-layer address (MAC)
        // The cast is not unnecessary on all platforms.
        #[allow(clippy::unnecessary_cast)]
        pub fn addr(&self) -> Option<[u8; 6]> {
            let nlen = self.nlen();
            let data = self.0.sdl_data;

            if self.is_empty() {
                None
            } else {
                Some([
                    data[nlen] as u8,
                    data[nlen + 1] as u8,
                    data[nlen + 2] as u8,
                    data[nlen + 3] as u8,
                    data[nlen + 4] as u8,
                    data[nlen + 5] as u8,
                ])
            }
        }
    }

    impl fmt::Display for LinkAddr {
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
    impl private::SockaddrLikePriv for LinkAddr {}
    impl SockaddrLike for LinkAddr {
        unsafe fn from_raw(addr: *const libc::sockaddr,
                           len: Option<libc::socklen_t>)
            -> Option<Self> where Self: Sized
        {
            if let Some(l) = len {
                if l != mem::size_of::<libc::sockaddr_dl>() as libc::socklen_t {
                    return None;
                }
            }
            if unsafe { (*addr).sa_family as i32 != libc::AF_LINK } {
                return None;
            }
            Some(Self(unsafe { ptr::read_unaligned(addr as *const _) }))
        }
    }

    impl AsRef<libc::sockaddr_dl> for LinkAddr {
        fn as_ref(&self) -> &libc::sockaddr_dl {
            &self.0
        }
    }
    }
}

#[cfg(any(linux_android, target_os = "macos"))]
pub mod vsock {
    use super::*;
    use crate::sys::socket::addr::AddressFamily;
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
    pub struct VsockAddr(pub(in super::super) sockaddr_vm);

    impl private::SockaddrLikePriv for VsockAddr {}
    impl SockaddrLike for VsockAddr {
        unsafe fn from_raw(
            addr: *const libc::sockaddr,
            len: Option<libc::socklen_t>,
        ) -> Option<Self>
        where
            Self: Sized,
        {
            if let Some(l) = len {
                if l != mem::size_of::<libc::sockaddr_vm>() as libc::socklen_t {
                    return None;
                }
            }
            if unsafe { (*addr).sa_family as i32 != libc::AF_VSOCK } {
                return None;
            }
            unsafe { Some(Self(ptr::read_unaligned(addr as *const _))) }
        }
    }

    impl AsRef<libc::sockaddr_vm> for VsockAddr {
        fn as_ref(&self) -> &libc::sockaddr_vm {
            &self.0
        }
    }

    impl PartialEq for VsockAddr {
        #[cfg(linux_android)]
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

    impl Eq for VsockAddr {}

    impl Hash for VsockAddr {
        #[cfg(linux_android)]
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
    impl VsockAddr {
        /// Construct a `VsockAddr` from its raw fields.
        pub fn new(cid: u32, port: u32) -> VsockAddr {
            let mut addr: sockaddr_vm = unsafe { mem::zeroed() };
            addr.svm_family = AddressFamily::VSOCK.family() as sa_family_t;
            addr.svm_cid = cid;
            addr.svm_port = port;

            #[cfg(target_os = "macos")]
            {
                addr.svm_len = std::mem::size_of::<sockaddr_vm>() as u8;
            }
            VsockAddr(addr)
        }

        /// Context Identifier (CID)
        pub fn cid(&self) -> u32 {
            self.0.svm_cid
        }

        /// Port number
        pub fn port(&self) -> u32 {
            self.0.svm_port
        }
    }

    impl fmt::Display for VsockAddr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "cid: {} port: {}", self.cid(), self.port())
        }
    }

    impl fmt::Debug for VsockAddr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(self, f)
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

        #[cfg(any(apple_targets, target_os = "illumos"))]
        use super::super::super::socklen_t;
        use super::*;

        /// Don't panic when trying to display an empty datalink address
        #[cfg(bsd)]
        #[test]
        fn test_datalink_display() {
            use super::super::LinkAddr;
            use std::mem;

            let la = LinkAddr(libc::sockaddr_dl {
                sdl_len: 56,
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

        #[cfg(all(
            any(linux_android, target_os = "fuchsia"),
            target_endian = "little"
        ))]
        #[test]
        fn linux_loopback() {
            #[repr(align(2))]
            struct Raw([u8; 20]);

            let bytes = Raw([
                17u8, 0, 0, 0, 1, 0, 0, 0, 4, 3, 0, 6, 1, 2, 3, 4, 5, 6, 0, 0,
            ]);
            let sa = bytes.0.as_ptr().cast();
            let len = None;
            let sock_addr =
                unsafe { SockaddrStorage::from_raw(sa, len) }.unwrap();
            assert_eq!(sock_addr.family(), AddressFamily::PACKET);
            match sock_addr.as_link_addr() {
                Some(dl) => assert_eq!(dl.addr(), Some([1, 2, 3, 4, 5, 6])),
                None => panic!("Can't unwrap sockaddr storage"),
            }
        }

        #[cfg(apple_targets)]
        #[test]
        fn macos_loopback() {
            let bytes =
                [20i8, 18, 1, 0, 24, 3, 0, 0, 108, 111, 48, 0, 0, 0, 0, 0];
            let sa = bytes.as_ptr().cast();
            let len = Some(bytes.len() as socklen_t);
            let sock_addr =
                unsafe { SockaddrStorage::from_raw(sa, len) }.unwrap();
            assert_eq!(sock_addr.family(), AddressFamily::LINK);
            match sock_addr.as_link_addr() {
                Some(dl) => {
                    assert!(dl.addr().is_none());
                }
                None => panic!("Can't unwrap sockaddr storage"),
            }
        }

        #[cfg(apple_targets)]
        #[test]
        fn macos_tap() {
            let bytes = [
                20i8, 18, 7, 0, 6, 3, 6, 0, 101, 110, 48, 24, 101, -112, -35,
                76, -80,
            ];
            let ptr = bytes.as_ptr();
            let sa = ptr as *const libc::sockaddr;
            let len = Some(bytes.len() as socklen_t);

            let sock_addr =
                unsafe { SockaddrStorage::from_raw(sa, len).unwrap() };
            assert_eq!(sock_addr.family(), AddressFamily::LINK);
            match sock_addr.as_link_addr() {
                Some(dl) => {
                    assert_eq!(dl.addr(), Some([24u8, 101, 144, 221, 76, 176]))
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
            let len = Some(bytes.len() as socklen_t);
            let _sock_addr = unsafe { SockaddrStorage::from_raw(sa, len) };

            assert!(_sock_addr.is_some());

            let sock_addr = _sock_addr.unwrap();

            assert_eq!(sock_addr.family(), AddressFamily::LINK);

            assert_eq!(
                sock_addr.as_link_addr().unwrap().addr(),
                Some([24u8, 101, 144, 221, 76, 176])
            );
        }

        #[test]
        fn size() {
            #[cfg(any(
                bsd,
                target_os = "aix",
                target_os = "illumos",
                target_os = "haiku"
            ))]
            let l = mem::size_of::<libc::sockaddr_dl>();
            #[cfg(any(linux_android, target_os = "fuchsia"))]
            let l = mem::size_of::<libc::sockaddr_ll>();
            assert_eq!(LinkAddr::size() as usize, l);
        }
    }

    mod sockaddr_in {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn display() {
            let s = "127.0.0.1:8080";
            let addr = SockaddrIn::from_str(s).unwrap();
            assert_eq!(s, format!("{addr}"));
        }

        #[test]
        fn size() {
            assert_eq!(
                mem::size_of::<libc::sockaddr_in>(),
                SockaddrIn::size() as usize
            );
        }

        #[test]
        fn ip() {
            let s = "127.0.0.1:8080";
            let ip = SockaddrIn::from_str(s).unwrap().ip();
            assert_eq!("127.0.0.1", format!("{ip}"));
        }
    }

    mod sockaddr_in6 {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn display() {
            let s = "[1234:5678:90ab:cdef::1111:2222]:8080";
            let addr = SockaddrIn6::from_str(s).unwrap();
            assert_eq!(s, format!("{addr}"));
        }

        #[test]
        fn size() {
            assert_eq!(
                mem::size_of::<libc::sockaddr_in6>(),
                SockaddrIn6::size() as usize
            );
        }

        #[test]
        fn ip() {
            let s = "[1234:5678:90ab:cdef::1111:2222]:8080";
            let ip = SockaddrIn6::from_str(s).unwrap().ip();
            assert_eq!("1234:5678:90ab:cdef::1111:2222", format!("{ip}"));
        }

        #[test]
        // Ensure that we can convert to-and-from std::net variants without change.
        fn to_and_from() {
            let s = "[1234:5678:90ab:cdef::1111:2222]:8080";
            let mut nix_sin6 = SockaddrIn6::from_str(s).unwrap();
            nix_sin6.0.sin6_flowinfo = 0x12345678;
            nix_sin6.0.sin6_scope_id = 0x9abcdef0;

            let std_sin6: std::net::SocketAddrV6 = nix_sin6.into();
            assert_eq!(nix_sin6, std_sin6.into());
        }
    }

    mod sockaddr_storage {
        use super::*;

        #[test]
        fn from_sockaddr_un_named() {
            let ua = UnixAddr::new("/var/run/mysock").unwrap();
            let ptr = ua.as_ptr().cast();
            let ss = unsafe { SockaddrStorage::from_raw(ptr, Some(ua.len())) }
                .unwrap();
            assert_eq!(ss.len(), ua.len());
        }

        #[cfg(linux_android)]
        #[test]
        fn from_sockaddr_un_abstract_named() {
            let name = String::from("nix\0abstract\0test");
            let ua = UnixAddr::new_abstract(name.as_bytes()).unwrap();
            let ptr = ua.as_ptr().cast();
            let ss = unsafe { SockaddrStorage::from_raw(ptr, Some(ua.len())) }
                .unwrap();
            assert_eq!(ss.len(), ua.len());
        }

        #[cfg(linux_android)]
        #[test]
        fn from_sockaddr_un_abstract_unnamed() {
            let ua = UnixAddr::new_unnamed();
            let ptr = ua.as_ptr().cast();
            let ss = unsafe { SockaddrStorage::from_raw(ptr, Some(ua.len())) }
                .unwrap();
            assert_eq!(ss.len(), ua.len());
        }
    }

    mod unixaddr {
        use super::*;

        #[cfg(linux_android)]
        #[test]
        fn abstract_sun_path() {
            let name = String::from("nix\0abstract\0test");
            let addr = UnixAddr::new_abstract(name.as_bytes()).unwrap();

            let sun_path1 =
                unsafe { &(*addr.as_ptr()).sun_path[..addr.path_len()] };
            let sun_path2 = [
                0, 110, 105, 120, 0, 97, 98, 115, 116, 114, 97, 99, 116, 0,
                116, 101, 115, 116,
            ];
            assert_eq!(sun_path1, sun_path2);
        }

        #[test]
        fn size() {
            assert_eq!(
                mem::size_of::<libc::sockaddr_un>(),
                UnixAddr::size() as usize
            );
        }
    }
}
