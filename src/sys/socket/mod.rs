//! Socket interface functions
//!
//! # Addresses
//!
//! All socket addresses consist of three parts:
//!
//! - An [`AddressFamily`], which specifies the format of the address.
//!
//! - A length, even if isn't embedded into the libc address type. It indicates the number of bytes
//! of the address that should be read when interpreting it.
//!
//! - The address itself.
//!
//! ## `Address` vs `Addr`
//!
//! Socket addresses come in two flavors:
//!
//! - Addresses ending with `Address` are sized and safely convertible to libc's address types, regardless
//! of their length. Their address length is bounded by `[2, SIZE]` with `SIZE` being the libc address type's size.
//! The only exception is [`Address`], where the length can also be zero, indicating an empty, uninitialized address.
//!
//! - Addresses ending with `Addr` are unsized. They are generally *not* directly convertible to libc's address
//! types (they are convertible if and only if the address length matches the libc address type's size). Their address
//! length is bounded by `[2, MAX]` with `MAX` being the maximum address length, i.e. the size of [`sockaddr_storage`].
//! Similar to above, [`Addr`] can also have a length of zero. Trying to instantiate or using an address with a length
//! greater than `MAX` can result in a panic.
//!
//! **Note**: Not all addresses have a dyn-sized variant, in fact, only a few do. That's because most addresses are fixed in
//! size and wouldn't benefit from a dyn-sized alternative. Only those with a variable length have a dyn-sized variant.
//!
//! [Further reading](https://man7.org/linux/man-pages/man7/socket.7.html)
#[cfg(any(target_os = "android", target_os = "linux"))]
#[cfg(feature = "uio")]
use crate::sys::time::TimeSpec;
#[cfg(not(target_os = "redox"))]
#[cfg(feature = "uio")]
use crate::sys::time::TimeVal;
use crate::{errno::Errno, Result};
use cfg_if::cfg_if;
use libc::{self, c_int, size_t, socklen_t};
#[cfg(all(feature = "uio", not(target_os = "redox")))]
use libc::{CMSG_DATA, CMSG_FIRSTHDR, CMSG_LEN, CMSG_NXTHDR, CMSG_SPACE};
#[cfg(not(target_os = "redox"))]
use std::io::{IoSlice, IoSliceMut};
#[allow(unused)]
use std::mem::MaybeUninit;
#[cfg(feature = "net")]
use std::net;
use std::os::unix::io::{AsFd, AsRawFd, FromRawFd, OwnedFd, RawFd};
#[cfg(not(target_os = "redox"))]
use std::ptr::addr_of_mut;
use std::{mem, ptr};

#[deny(missing_docs)]
mod addr;
#[deny(missing_docs)]
pub mod sockopt;

/*
 *
 * ===== Re-exports =====
 *
 */

pub use self::addr::{
    Addr, Address, AddressFamily, InvalidAddressFamilyError, RawAddr, UnixAddr,
    UnixAddress,
};

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
#[cfg(feature = "net")]
pub use self::addr::LinkAddr;
#[cfg(any(target_os = "solaris", target_os = "redox",))]
#[cfg(feature = "net")]
pub use self::addr::{Ipv4Address, Ipv6Address};
#[cfg(not(any(target_os = "solaris", target_os = "redox",)))]
#[cfg(feature = "net")]
pub use self::addr::{Ipv4Address, Ipv6Address, LinkAddress};

#[cfg(any(target_os = "android", target_os = "linux"))]
pub use crate::sys::socket::addr::alg::AlgAddress;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use crate::sys::socket::addr::netlink::NetlinkAddress;
#[cfg(any(target_os = "ios", target_os = "macos"))]
#[cfg(feature = "ioctl")]
pub use crate::sys::socket::addr::sys_control::SysControlAddress;
#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "macos"
))]
pub use crate::sys::socket::addr::vsock::VsockAddress;

#[cfg(all(feature = "uio", not(target_os = "redox")))]
pub use libc::{cmsghdr, msghdr};
pub use libc::{sa_family_t, sockaddr, sockaddr_storage, sockaddr_un};
#[cfg(feature = "net")]
pub use libc::{sockaddr_in, sockaddr_in6};

#[cfg(feature = "net")]
use crate::sys::socket::addr::{ipv4addr_to_libc, ipv6addr_to_libc};

/// These constants are used to specify the communication semantics
/// when creating a socket with [`socket()`](fn.socket.html)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
#[non_exhaustive]
pub enum SockType {
    /// Provides sequenced, reliable, two-way, connection-
    /// based byte streams.  An out-of-band data transmission
    /// mechanism may be supported.
    Stream = libc::SOCK_STREAM,
    /// Supports datagrams (connectionless, unreliable
    /// messages of a fixed maximum length).
    Datagram = libc::SOCK_DGRAM,
    /// Provides a sequenced, reliable, two-way connection-
    /// based data transmission path for datagrams of fixed
    /// maximum length; a consumer is required to read an
    /// entire packet with each input system call.
    SeqPacket = libc::SOCK_SEQPACKET,
    /// Provides raw network protocol access.
    #[cfg(not(target_os = "redox"))]
    Raw = libc::SOCK_RAW,
    /// Provides a reliable datagram layer that does not
    /// guarantee ordering.
    #[cfg(not(any(target_os = "haiku", target_os = "redox")))]
    Rdm = libc::SOCK_RDM,
}
// The TryFrom impl could've been derived using libc_enum!.  But for
// backwards-compatibility with Nix-0.25.0 we manually implement it, so as to
// keep the old variant names.
impl TryFrom<i32> for SockType {
    type Error = crate::Error;

    fn try_from(x: i32) -> Result<Self> {
        match x {
            libc::SOCK_STREAM => Ok(Self::Stream),
            libc::SOCK_DGRAM => Ok(Self::Datagram),
            libc::SOCK_SEQPACKET => Ok(Self::SeqPacket),
            #[cfg(not(target_os = "redox"))]
            libc::SOCK_RAW => Ok(Self::Raw),
            #[cfg(not(any(target_os = "haiku", target_os = "redox")))]
            libc::SOCK_RDM => Ok(Self::Rdm),
            _ => Err(Errno::EINVAL),
        }
    }
}

/// Constants used in [`socket`](fn.socket.html) and [`socketpair`](fn.socketpair.html)
/// to specify the protocol to use.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum SockProtocol {
    /// TCP protocol ([ip(7)](https://man7.org/linux/man-pages/man7/ip.7.html))
    Tcp = libc::IPPROTO_TCP,
    /// UDP protocol ([ip(7)](https://man7.org/linux/man-pages/man7/ip.7.html))
    Udp = libc::IPPROTO_UDP,
    /// Raw sockets ([raw(7)](https://man7.org/linux/man-pages/man7/raw.7.html))
    Raw = libc::IPPROTO_RAW,
    /// Allows applications to configure and control a KEXT
    /// ([ref](https://developer.apple.com/library/content/documentation/Darwin/Conceptual/NKEConceptual/control/control.html))
    #[cfg(apple_targets)]
    KextControl = libc::SYSPROTO_CONTROL,
    /// Receives routing and link updates and may be used to modify the routing tables (both IPv4 and IPv6), IP addresses, link
    // parameters, neighbor setups, queueing disciplines, traffic classes and packet classifiers
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkRoute = libc::NETLINK_ROUTE,
    /// Reserved for user-mode socket protocols
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkUserSock = libc::NETLINK_USERSOCK,
    /// Query information about sockets of various protocol families from the kernel
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkSockDiag = libc::NETLINK_SOCK_DIAG,
    /// Netfilter/iptables ULOG.
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkNFLOG = libc::NETLINK_NFLOG,
    /// SELinux event notifications.
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkSELinux = libc::NETLINK_SELINUX,
    /// Open-iSCSI
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkISCSI = libc::NETLINK_ISCSI,
    /// Auditing
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkAudit = libc::NETLINK_AUDIT,
    /// Access to FIB lookup from user space
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkFIBLookup = libc::NETLINK_FIB_LOOKUP,
    /// Netfilter subsystem
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkNetFilter = libc::NETLINK_NETFILTER,
    /// SCSI Transports
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkSCSITransport = libc::NETLINK_SCSITRANSPORT,
    /// Infiniband RDMA
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkRDMA = libc::NETLINK_RDMA,
    /// Transport IPv6 packets from netfilter to user space.  Used by ip6_queue kernel module.
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkIPv6Firewall = libc::NETLINK_IP6_FW,
    /// DECnet routing messages
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkDECNetRoutingMessage = libc::NETLINK_DNRTMSG,
    /// Kernel messages to user space
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkKObjectUEvent = libc::NETLINK_KOBJECT_UEVENT,
    /// Generic netlink family for simplified netlink usage.
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkGeneric = libc::NETLINK_GENERIC,
    /// Netlink interface to request information about ciphers registered with the kernel crypto API as well as allow
    /// configuration of the kernel crypto API.
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkCrypto = libc::NETLINK_CRYPTO,
    /// Non-DIX type protocol number defined for the Ethernet IEEE 802.3 interface that allows packets of all protocols
    /// defined in the interface to be received.
    /// ([ref](https://man7.org/linux/man-pages/man7/packet.7.html))
    // The protocol number is fed into the socket syscall in network byte order.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    EthAll = (libc::ETH_P_ALL as u16).to_be() as i32,
    /// ICMP protocol ([icmp(7)](https://man7.org/linux/man-pages/man7/icmp.7.html))
    Icmp = libc::IPPROTO_ICMP,
    /// ICMPv6 protocol (ICMP over IPv6)
    IcmpV6 = libc::IPPROTO_ICMPV6,
}

impl SockProtocol {
    /// The Controller Area Network raw socket protocol
    /// ([ref](https://docs.kernel.org/networking/can.html#how-to-use-socketcan))
    #[cfg(target_os = "linux")]
    #[allow(non_upper_case_globals)]
    pub const CanRaw: SockProtocol = SockProtocol::Icmp; // Matches libc::CAN_RAW

    /// The Controller Area Network broadcast manager protocol
    /// ([ref](https://docs.kernel.org/networking/can.html#how-to-use-socketcan))
    #[cfg(target_os = "linux")]
    #[allow(non_upper_case_globals)]
    pub const CanBcm: SockProtocol = SockProtocol::NetlinkUserSock; // Matches libc::CAN_BCM

    /// Allows applications and other KEXTs to be notified when certain kernel events occur
    /// ([ref](https://developer.apple.com/library/content/documentation/Darwin/Conceptual/NKEConceptual/control/control.html))
    #[cfg(apple_targets)]
    #[allow(non_upper_case_globals)]
    pub const KextEvent: SockProtocol = SockProtocol::Icmp; // Matches libc::SYSPROTO_EVENT
}
#[cfg(any(target_os = "android", target_os = "linux"))]
libc_bitflags! {
    /// Configuration flags for `SO_TIMESTAMPING` interface
    ///
    /// For use with [`Timestamping`][sockopt::Timestamping].
    /// [Further reading](https://www.kernel.org/doc/html/latest/networking/timestamping.html)
    pub struct TimestampingFlag: libc::c_uint {
        /// Report any software timestamps when available.
        SOF_TIMESTAMPING_SOFTWARE;
        /// Report hardware timestamps as generated by SOF_TIMESTAMPING_TX_HARDWARE when available.
        SOF_TIMESTAMPING_RAW_HARDWARE;
        /// Collect transmitting timestamps as reported by hardware
        SOF_TIMESTAMPING_TX_HARDWARE;
        /// Collect transmitting timestamps as reported by software
        SOF_TIMESTAMPING_TX_SOFTWARE;
        /// Collect receiving timestamps as reported by hardware
        SOF_TIMESTAMPING_RX_HARDWARE;
        /// Collect receiving timestamps as reported by software
        SOF_TIMESTAMPING_RX_SOFTWARE;
        /// Generate a unique identifier along with each transmitted packet
        SOF_TIMESTAMPING_OPT_ID;
        /// Return transmit timestamps alongside an empty packet instead of the original packet
        SOF_TIMESTAMPING_OPT_TSONLY;
    }
}

libc_bitflags! {
    /// Additional socket options
    pub struct SockFlag: c_int {
        /// Set non-blocking mode on the new socket
        #[cfg(any(target_os = "android",
                  target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "illumos",
                  target_os = "linux",
                  target_os = "netbsd",
                  target_os = "openbsd"))]
        SOCK_NONBLOCK;
        /// Set close-on-exec on the new descriptor
        #[cfg(any(target_os = "android",
                  target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "illumos",
                  target_os = "linux",
                  target_os = "netbsd",
                  target_os = "openbsd"))]
        SOCK_CLOEXEC;
        /// Return `EPIPE` instead of raising `SIGPIPE`
        #[cfg(target_os = "netbsd")]
        SOCK_NOSIGPIPE;
        /// For domains `AF_INET(6)`, only allow `connect(2)`, `sendto(2)`, or `sendmsg(2)`
        /// to the DNS port (typically 53)
        #[cfg(target_os = "openbsd")]
        SOCK_DNS;
    }
}

libc_bitflags! {
    /// Flags for send/recv and their relatives
    pub struct MsgFlags: c_int {
        /// Sends or requests out-of-band data on sockets that support this notion
        /// (e.g., of type [`Stream`](enum.SockType.html)); the underlying protocol must also
        /// support out-of-band data.
        MSG_OOB;
        /// Peeks at an incoming message. The data is treated as unread and the next
        /// [`recv()`](fn.recv.html)
        /// or similar function shall still return this data.
        MSG_PEEK;
        /// Receive operation blocks until the full amount of data can be
        /// returned. The function may return smaller amount of data if a signal
        /// is caught, an error or disconnect occurs.
        MSG_WAITALL;
        /// Enables nonblocking operation; if the operation would block,
        /// `EAGAIN` or `EWOULDBLOCK` is returned.  This provides similar
        /// behavior to setting the `O_NONBLOCK` flag
        /// (via the [`fcntl`](../../fcntl/fn.fcntl.html)
        /// `F_SETFL` operation), but differs in that `MSG_DONTWAIT` is a per-
        /// call option, whereas `O_NONBLOCK` is a setting on the open file
        /// description (see [open(2)](https://man7.org/linux/man-pages/man2/open.2.html)),
        /// which will affect all threads in
        /// the calling process and as well as other processes that hold
        /// file descriptors referring to the same open file description.
        #[cfg(not(target_os = "aix"))]
        MSG_DONTWAIT;
        /// Receive flags: Control Data was discarded (buffer too small)
        MSG_CTRUNC;
        /// For raw (`AF_PACKET`), Internet datagram
        /// (since Linux 2.4.27/2.6.8),
        /// netlink (since Linux 2.6.22) and UNIX datagram (since Linux 3.4)
        /// sockets: return the real length of the packet or datagram, even
        /// when it was longer than the passed buffer. Not implemented for UNIX
        /// domain ([unix(7)](https://linux.die.net/man/7/unix)) sockets.
        ///
        /// For use with Internet stream sockets, see [tcp(7)](https://linux.die.net/man/7/tcp).
        MSG_TRUNC;
        /// Terminates a record (when this notion is supported, as for
        /// sockets of type [`SeqPacket`](enum.SockType.html)).
        MSG_EOR;
        /// This flag specifies that queued errors should be received from
        /// the socket error queue. (For more details, see
        /// [recvfrom(2)](https://linux.die.net/man/2/recvfrom))
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MSG_ERRQUEUE;
        /// Set the `close-on-exec` flag for the file descriptor received via a UNIX domain
        /// file descriptor using the `SCM_RIGHTS` operation (described in
        /// [unix(7)](https://linux.die.net/man/7/unix)).
        /// This flag is useful for the same reasons as the `O_CLOEXEC` flag of
        /// [open(2)](https://pubs.opengroup.org/onlinepubs/9699919799/functions/open.html).
        ///
        /// Only used in [`recvmsg`](fn.recvmsg.html) function.
        #[cfg(any(target_os = "android",
                  target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "linux",
                  target_os = "netbsd",
                  target_os = "openbsd"))]
        MSG_CMSG_CLOEXEC;
        /// Requests not to send `SIGPIPE` errors when the other end breaks the connection.
        /// (For more details, see [send(2)](https://linux.die.net/man/2/send)).
        #[cfg(any(target_os = "android",
                  target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "fuchsia",
                  target_os = "haiku",
                  target_os = "illumos",
                  target_os = "linux",
                  target_os = "netbsd",
                  target_os = "openbsd",
                  target_os = "solaris"))]
        MSG_NOSIGNAL;
        /// Turns on [`MSG_DONTWAIT`] after the first message has been received (only for
        /// `recvmmsg()`).
        #[cfg(any(target_os = "android",
                  target_os = "fuchsia",
                  target_os = "linux",
                  target_os = "netbsd",
                  target_os = "freebsd",
                  target_os = "openbsd",
                  target_os = "solaris"))]
        MSG_WAITFORONE;
    }
}

cfg_if! {
    if #[cfg(any(target_os = "android", target_os = "linux"))] {
        /// Unix credentials of the sending process.
        ///
        /// This struct is used with the `SO_PEERCRED` ancillary message
        /// and the `SCM_CREDENTIALS` control message for UNIX sockets.
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct UnixCredentials(libc::ucred);

        impl UnixCredentials {
            /// Creates a new instance with the credentials of the current process
            pub fn new() -> Self {
                // Safe because these FFI functions are inherently safe
                unsafe {
                    UnixCredentials(libc::ucred {
                        pid: libc::getpid(),
                        uid: libc::getuid(),
                        gid: libc::getgid()
                    })
                }
            }

            /// Returns the process identifier
            pub fn pid(&self) -> libc::pid_t {
                self.0.pid
            }

            /// Returns the user identifier
            pub fn uid(&self) -> libc::uid_t {
                self.0.uid
            }

            /// Returns the group identifier
            pub fn gid(&self) -> libc::gid_t {
                self.0.gid
            }
        }

        impl Default for UnixCredentials {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<libc::ucred> for UnixCredentials {
            fn from(cred: libc::ucred) -> Self {
                UnixCredentials(cred)
            }
        }

        impl From<UnixCredentials> for libc::ucred {
            fn from(uc: UnixCredentials) -> Self {
                uc.0
            }
        }
    } else if #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))] {
        /// Unix credentials of the sending process.
        ///
        /// This struct is used with the `SCM_CREDS` ancillary message for UNIX sockets.
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct UnixCredentials(libc::cmsgcred);

        impl UnixCredentials {
            /// Returns the process identifier
            pub fn pid(&self) -> libc::pid_t {
                self.0.cmcred_pid
            }

            /// Returns the real user identifier
            pub fn uid(&self) -> libc::uid_t {
                self.0.cmcred_uid
            }

            /// Returns the effective user identifier
            pub fn euid(&self) -> libc::uid_t {
                self.0.cmcred_euid
            }

            /// Returns the real group identifier
            pub fn gid(&self) -> libc::gid_t {
                self.0.cmcred_gid
            }

            /// Returns a list group identifiers (the first one being the effective GID)
            pub fn groups(&self) -> &[libc::gid_t] {
                unsafe {
                    std::slice::from_raw_parts(
                        self.0.cmcred_groups.as_ptr(),
                        self.0.cmcred_ngroups as _
                    )
                }
            }
        }

        impl From<libc::cmsgcred> for UnixCredentials {
            fn from(cred: libc::cmsgcred) -> Self {
                UnixCredentials(cred)
            }
        }
    }
}

cfg_if! {
    if #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "macos",
                target_os = "ios"
        ))] {
        /// Return type of [`LocalPeerCred`](crate::sys::socket::sockopt::LocalPeerCred)
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct XuCred(libc::xucred);

        impl XuCred {
            /// Structure layout version
            pub fn version(&self) -> u32 {
                self.0.cr_version
            }

            /// Effective user ID
            pub fn uid(&self) -> libc::uid_t {
                self.0.cr_uid
            }

            /// Returns a list of group identifiers (the first one being the
            /// effective GID)
            pub fn groups(&self) -> &[libc::gid_t] {
                &self.0.cr_groups
            }
        }
    }
}

feature! {
#![feature = "net"]
/// Request for multicast socket operations
///
/// This is a wrapper type around `ip_mreq`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IpMembershipRequest(libc::ip_mreq);

impl IpMembershipRequest {
    /// Instantiate a new `IpMembershipRequest`
    ///
    /// If `interface` is `None`, then `Ipv4Addr::any()` will be used for the interface.
    pub fn new(group: net::Ipv4Addr, interface: Option<net::Ipv4Addr>)
        -> Self
    {
        let imr_addr = match interface {
            None => net::Ipv4Addr::UNSPECIFIED,
            Some(addr) => addr
        };
        IpMembershipRequest(libc::ip_mreq {
            imr_multiaddr: ipv4addr_to_libc(group),
            imr_interface: ipv4addr_to_libc(imr_addr)
        })
    }
}

/// Request for ipv6 multicast socket operations
///
/// This is a wrapper type around `ipv6_mreq`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ipv6MembershipRequest(libc::ipv6_mreq);

impl Ipv6MembershipRequest {
    /// Instantiate a new `Ipv6MembershipRequest`
    pub const fn new(group: net::Ipv6Addr) -> Self {
        Ipv6MembershipRequest(libc::ipv6_mreq {
            ipv6mr_multiaddr: ipv6addr_to_libc(&group),
            ipv6mr_interface: 0,
        })
    }
}
}

#[cfg(not(target_os = "redox"))]
feature! {
#![feature = "uio"]

/// Calculates the space needed for the provided arguments.
///
/// The arguments are the names of the variants of [`ControlMessageOwnedSpace`]. This macro
/// is const-evaluable.
#[macro_export]
macro_rules! cmsg_space {
    ($($x:ident $(($arg:expr))? ),* $(,)?) => {{
        0usize $(
            + <$crate::sys::socket::ControlMessageOwnedSpace>::$x $(($arg))?.space()
        )*
    }};
}

/// Creates a [`CmsgBuf`] with the capacity needed for **receiving**
/// the provided arguments.
///
/// The arguments are the names of the variants of [`ControlMessageOwnedSpace`].
///
/// # Example
///
/// ```
/// # use nix::{cmsg_space, cmsg_buf};
/// let cmsg = cmsg_buf![ScmRights(2), ScmTimestamp];
///
/// assert_eq!(cmsg.capacity(), cmsg_space![ScmRights(2), ScmTimestamp]);
/// ```
#[macro_export]
macro_rules! cmsg_buf {
    ($($x:ident $(($arg:expr))? ),* $(,)?) => {{
        const SPACE: usize = $crate::cmsg_space![$($x $(($arg))? ),*];

        <$crate::sys::socket::CmsgBuf>::with_capacity(SPACE)
    }};
}

// FIXME (2023-11-13): the module-internal test `recvmmsg2` requires a version of the macro without
// an absolute path to `cmsg_space!`. This workaround is necessary until the macro resolution
// of the compiler isn't as horrendous as it is currently.
macro_rules! cmsg_vec_internal {
    ($($x:ident $(($arg:expr))? ),* $(,)?) => {{
        const SPACE: usize = cmsg_space![$($x $(($arg))? ),*];

        <$crate::sys::socket::CmsgBuf>::with_capacity(SPACE)
    }};
}

/// An iterator created by [`CmsgBuf::iter`], yielding control messages of type
/// [`ControlMessageOwned`].
#[derive(Clone, Copy, Debug)]
pub struct CmsgIterator<'a> {
    /// Control message buffer to decode from. Must adhere to cmsg alignment.
    cmsghdr: Option<&'a cmsghdr>,
    // SAFETY: `msg_control` and `msg_controllen` must be initialized.
    mhdr: MaybeUninit<msghdr>,
}

impl<'a> Iterator for CmsgIterator<'a> {
    type Item = ControlMessageOwned;

    fn next(&mut self) -> Option<ControlMessageOwned> {
        match self.cmsghdr {
            None => None,   // No more messages
            Some(hdr) => {
                // Get the data.
                // Safe if cmsghdr points to valid data returned by recvmsg(2)
                let cm = unsafe { Some(ControlMessageOwned::decode_from(hdr))};
                // Advance the internal pointer.  Safe if mhdr and cmsghdr point
                // to valid data returned by recvmsg(2)
                self.cmsghdr = unsafe {
                    let p = CMSG_NXTHDR(self.mhdr.as_ptr(), hdr as *const _);
                    p.as_ref()
                };
                cm
            }
        }
    }
}

/// A type-safe wrapper around a single control message, as used with
/// [`recvmsg`](#fn.recvmsg).
///
/// **Note**: This is *not* the owned version of [`ControlMessage`] as they don't
/// necessarily have the same variants.
///
/// [Further reading](https://man7.org/linux/man-pages/man3/cmsg.3.html)

//  Nix version 0.13.0 and earlier used ControlMessage for both recvmsg and
//  sendmsg.  However, on some platforms the messages returned by recvmsg may be
//  unaligned.  ControlMessageOwned takes those messages by copy, obviating any
//  alignment issues.
//
//  See https://github.com/nix-rust/nix/issues/999
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ControlMessageOwned {
    /// Received version of [`ControlMessage::ScmRights`]
    ScmRights(Vec<RawFd>),
    /// Received version of [`ControlMessage::ScmCredentials`]
    #[cfg(any(target_os = "android", target_os = "linux"))]
    ScmCredentials(UnixCredentials),
    /// Received version of [`ControlMessage::ScmCreds`]
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    ScmCreds(UnixCredentials),
    /// A message of type `SCM_TIMESTAMP`, containing the time the
    /// packet was received by the kernel.
    ///
    /// See the kernel's explanation in "SO_TIMESTAMP" of
    /// [networking/timestamping](https://www.kernel.org/doc/Documentation/networking/timestamping.txt).
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate nix;
    /// # use nix::sys::socket::*;
    /// # use nix::sys::time::*;
    /// # use std::io::{IoSlice, IoSliceMut};
    /// # use std::time::*;
    /// # use std::str::FromStr;
    /// # use std::os::unix::io::AsRawFd;
    /// # fn main() {
    /// // Set up
    /// let message = "Ohayō!".as_bytes();
    /// let in_socket = socket(
    ///     AddressFamily::INET,
    ///     SockType::Datagram,
    ///     SockFlag::empty(),
    ///     None).unwrap();
    /// setsockopt(&in_socket, sockopt::ReceiveTimestamp, &true).unwrap();
    /// let localhost = Ipv4Address::from_str("127.0.0.1:0").unwrap();
    /// bind(in_socket.as_raw_fd(), &localhost).unwrap();
    /// let address = getsockname(in_socket.as_raw_fd()).unwrap();
    /// // Get initial time
    /// let time0 = SystemTime::now();
    /// // Send the message
    /// let iov = [IoSlice::new(message)];
    /// let flags = MsgFlags::empty();
    /// let l = sendmsg(
    ///     in_socket.as_raw_fd(),
    ///     &address,
    ///     &iov,
    ///     CmsgStr::empty(),
    ///     flags,
    /// ).unwrap().bytes();
    /// assert_eq!(message.len(), l);
    /// // Receive the message
    /// let mut buffer = vec![0u8; message.len()];
    /// let mut cmsg = cmsg_buf![ScmTimestamp];
    /// let mut iov = [IoSliceMut::new(&mut buffer)];
    /// let _ = recvmsg(
    ///     in_socket.as_raw_fd(),
    ///     &mut iov,
    ///     cmsg.handle(),
    ///     flags,
    /// ).unwrap();
    /// let rtime = match cmsg.iter().next() {
    ///     Some(ControlMessageOwned::ScmTimestamp(rtime)) => rtime,
    ///     Some(_) => panic!("Unexpected control message"),
    ///     None => panic!("No control message")
    /// };
    /// // Check the final time
    /// let time1 = SystemTime::now();
    /// // the packet's received timestamp should lie in-between the two system
    /// // times, unless the system clock was adjusted in the meantime.
    /// let rduration = Duration::new(rtime.tv_sec() as u64,
    ///                               rtime.tv_usec() as u32 * 1000);
    /// assert!(time0.duration_since(UNIX_EPOCH).unwrap() <= rduration);
    /// assert!(rduration <= time1.duration_since(UNIX_EPOCH).unwrap());
    /// // Close socket
    /// # }
    /// ```
    ScmTimestamp(TimeVal),
    /// A set of nanosecond resolution timestamps
    ///
    /// [Further reading](https://www.kernel.org/doc/html/latest/networking/timestamping.html)
    #[cfg(any(target_os = "android", target_os = "linux"))]
    ScmTimestampsns(Timestamps),
    /// Nanoseconds resolution timestamp
    ///
    /// [Further reading](https://www.kernel.org/doc/html/latest/networking/timestamping.html)
    #[cfg(any(target_os = "android", target_os = "linux"))]
    ScmTimestampns(TimeSpec),
    #[cfg(any(
        target_os = "android",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
    ))]
    #[cfg(feature = "net")]
    Ipv4PacketInfo(libc::in_pktinfo),
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "netbsd",
    ))]
    #[cfg(feature = "net")]
    Ipv6PacketInfo(libc::in6_pktinfo),
    #[cfg(any(
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    #[cfg(feature = "net")]
    Ipv4RecvIf(libc::sockaddr_dl),
    #[cfg(any(
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    #[cfg(feature = "net")]
    Ipv4RecvDstAddr(libc::in_addr),
    #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
    #[cfg(feature = "net")]
    Ipv4OrigDstAddr(libc::sockaddr_in),
    #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
    #[cfg(feature = "net")]
    Ipv6OrigDstAddr(libc::sockaddr_in6),

    /// UDP Generic Receive Offload (GRO) allows receiving multiple UDP
    /// packets from a single sender.
    /// Fixed-size payloads are following one by one in a receive buffer.
    /// This Control Message indicates the size of all smaller packets,
    /// except, maybe, the last one.
    ///
    /// `UdpGroSegment` socket option should be enabled on a socket
    /// to allow receiving GRO packets.
    #[cfg(target_os = "linux")]
    #[cfg(feature = "net")]
    UdpGroSegments(u16),

    /// SO_RXQ_OVFL indicates that an unsigned 32 bit value
    /// ancilliary msg (cmsg) should be attached to recieved
    /// skbs indicating the number of packets dropped by the
    /// socket between the last recieved packet and this
    /// received packet.
    ///
    /// `RxqOvfl` socket option should be enabled on a socket
    /// to allow receiving the drop counter.
    #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
    RxqOvfl(u32),

    /// Socket error queue control messages read with the `MSG_ERRQUEUE` flag.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[cfg(feature = "net")]
    Ipv4RecvErr(libc::sock_extended_err, Option<sockaddr_in>),
    /// Socket error queue control messages read with the `MSG_ERRQUEUE` flag.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[cfg(feature = "net")]
    Ipv6RecvErr(libc::sock_extended_err, Option<sockaddr_in6>),

    /// `SOL_TLS` messages of type `TLS_GET_RECORD_TYPE`
    #[cfg(target_os = "linux")]
    TlsGetRecordType(TlsGetRecordType),

    /// Catch-all variant for unimplemented cmsg types.
    #[doc(hidden)]
    Unknown(UnknownCmsg),
}

/// For representing packet timestamps via `SO_TIMESTAMPING` interface
#[cfg(any(target_os = "android", target_os = "linux"))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Timestamps {
    /// software based timestamp, usually one containing data
    pub system: TimeSpec,
    /// legacy timestamp, usually empty
    pub hw_trans: TimeSpec,
    /// hardware based timestamp
    pub hw_raw: TimeSpec,
}

/// These constants correspond to TLS 1.2 message types, as defined in
/// RFC 5246, Appendix A.1
#[cfg(target_os = "linux")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
#[non_exhaustive]
pub enum TlsGetRecordType {
    ChangeCipherSpec ,
    Alert,
    Handshake,
    ApplicationData,
    Unknown(u8),
}

#[cfg(any(target_os = "linux"))]
impl From<u8> for TlsGetRecordType {
    fn from(x: u8) -> Self {
        match x {
            20 => TlsGetRecordType::ChangeCipherSpec,
            21 => TlsGetRecordType::Alert,
            22 => TlsGetRecordType::Handshake,
            23 => TlsGetRecordType::ApplicationData,
            _ => TlsGetRecordType::Unknown(x),
        }
    }
}

impl ControlMessageOwned {
    /// Decodes a `ControlMessageOwned` from raw bytes.
    ///
    /// This is only safe to call if the data is correct for the message type
    /// specified in the header. Normally, the kernel ensures that this is the
    /// case. "Correct" in this case includes correct length, alignment and
    /// actual content.
    // Clippy complains about the pointer alignment of `p`, not understanding
    // that it's being fed to a function that can handle that.
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn decode_from(header: &cmsghdr) -> ControlMessageOwned
    {
        let p = unsafe { CMSG_DATA(header) };

        // The cast is not unnecessary on all platforms.
        #[allow(clippy::unnecessary_cast)]
        let len = header as *const _ as usize + header.cmsg_len as usize
            - p as usize;
        match (header.cmsg_level, header.cmsg_type) {
            (libc::SOL_SOCKET, libc::SCM_RIGHTS) => {
                let n = len / mem::size_of::<RawFd>();
                let mut fds = Vec::with_capacity(n);
                for i in 0..n {
                    unsafe {
                        let fdp = (p as *const RawFd).add(i);
                        fds.push(ptr::read_unaligned(fdp));
                    }
                }
                ControlMessageOwned::ScmRights(fds)
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            (libc::SOL_SOCKET, libc::SCM_CREDENTIALS) => {
                let cred: libc::ucred = unsafe { ptr::read_unaligned(p as *const _) };
                ControlMessageOwned::ScmCredentials(cred.into())
            }
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            (libc::SOL_SOCKET, libc::SCM_CREDS) => {
                let cred: libc::cmsgcred = unsafe { ptr::read_unaligned(p as *const _) };
                ControlMessageOwned::ScmCreds(cred.into())
            }
            #[cfg(not(any(target_os = "aix", target_os = "haiku")))]
            (libc::SOL_SOCKET, libc::SCM_TIMESTAMP) => {
                let tv: libc::timeval = unsafe { ptr::read_unaligned(p as *const _) };
                ControlMessageOwned::ScmTimestamp(TimeVal::from(tv))
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            (libc::SOL_SOCKET, libc::SCM_TIMESTAMPNS) => {
                let ts: libc::timespec = unsafe { ptr::read_unaligned(p as *const _) };
                ControlMessageOwned::ScmTimestampns(TimeSpec::from(ts))
            }
            #[cfg(any(target_os = "android", target_os = "linux"))]
            (libc::SOL_SOCKET, libc::SCM_TIMESTAMPING) => {
                let tp = p as *const libc::timespec;
                let ts: libc::timespec = unsafe { ptr::read_unaligned(tp) };
                let system = TimeSpec::from(ts);
                let ts: libc::timespec = unsafe { ptr::read_unaligned(tp.add(1)) };
                let hw_trans = TimeSpec::from(ts);
                let ts: libc::timespec = unsafe { ptr::read_unaligned(tp.add(2)) };
                let hw_raw = TimeSpec::from(ts);
                let timestamping = Timestamps { system, hw_trans, hw_raw };
                ControlMessageOwned::ScmTimestampsns(timestamping)
            }
            #[cfg(any(
                target_os = "android",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "linux",
                target_os = "macos"
            ))]
            #[cfg(feature = "net")]
            (libc::IPPROTO_IPV6, libc::IPV6_PKTINFO) => {
                let info = unsafe { ptr::read_unaligned(p as *const libc::in6_pktinfo) };
                ControlMessageOwned::Ipv6PacketInfo(info)
            }
            #[cfg(any(
                target_os = "android",
                target_os = "ios",
                target_os = "linux",
                target_os = "macos",
                target_os = "netbsd",
            ))]
            #[cfg(feature = "net")]
            (libc::IPPROTO_IP, libc::IP_PKTINFO) => {
                let info = unsafe { ptr::read_unaligned(p as *const libc::in_pktinfo) };
                ControlMessageOwned::Ipv4PacketInfo(info)
            }
            #[cfg(any(
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            #[cfg(feature = "net")]
            (libc::IPPROTO_IP, libc::IP_RECVIF) => {
                let dl = unsafe { ptr::read_unaligned(p as *const libc::sockaddr_dl) };
                ControlMessageOwned::Ipv4RecvIf(dl)
            },
            #[cfg(any(
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            #[cfg(feature = "net")]
            (libc::IPPROTO_IP, libc::IP_RECVDSTADDR) => {
                let dl = unsafe { ptr::read_unaligned(p as *const libc::in_addr) };
                ControlMessageOwned::Ipv4RecvDstAddr(dl)
            },
            #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
            #[cfg(feature = "net")]
            (libc::IPPROTO_IP, libc::IP_ORIGDSTADDR) => {
                let dl = unsafe { ptr::read_unaligned(p as *const libc::sockaddr_in) };
                ControlMessageOwned::Ipv4OrigDstAddr(dl)
            },
            #[cfg(target_os = "linux")]
            #[cfg(feature = "net")]
            (libc::SOL_UDP, libc::UDP_GRO) => {
                let gso_size: u16 = unsafe { ptr::read_unaligned(p as *const _) };
                ControlMessageOwned::UdpGroSegments(gso_size)
            },
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            (libc::SOL_SOCKET, libc::SO_RXQ_OVFL) => {
                let drop_counter = unsafe { ptr::read_unaligned(p as *const u32) };
                ControlMessageOwned::RxqOvfl(drop_counter)
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            #[cfg(feature = "net")]
            (libc::IPPROTO_IP, libc::IP_RECVERR) => {
                let (err, addr) = unsafe { Self::recv_err_helper::<sockaddr_in>(p, len) };
                ControlMessageOwned::Ipv4RecvErr(err, addr)
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            #[cfg(feature = "net")]
            (libc::IPPROTO_IPV6, libc::IPV6_RECVERR) => {
                let (err, addr) = unsafe { Self::recv_err_helper::<sockaddr_in6>(p, len) };
                ControlMessageOwned::Ipv6RecvErr(err, addr)
            },
            #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
            #[cfg(feature = "net")]
            (libc::IPPROTO_IPV6, libc::IPV6_ORIGDSTADDR) => {
                let dl = unsafe { ptr::read_unaligned(p as *const libc::sockaddr_in6) };
                ControlMessageOwned::Ipv6OrigDstAddr(dl)
            },
            #[cfg(any(target_os = "linux"))]
            (libc::SOL_TLS, libc::TLS_GET_RECORD_TYPE) => {
                let content_type = unsafe { ptr::read_unaligned(p as *const u8) };
                ControlMessageOwned::TlsGetRecordType(content_type.into())
            },
            (_, _) => {
                let sl = unsafe { std::slice::from_raw_parts(p, len) };
                let ucmsg = UnknownCmsg(*header, Vec::<u8>::from(sl));
                ControlMessageOwned::Unknown(ucmsg)
            }
        }
    }

    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[cfg(feature = "net")]
    #[allow(clippy::cast_ptr_alignment)]    // False positive
    unsafe fn recv_err_helper<T>(p: *mut libc::c_uchar, len: usize) -> (libc::sock_extended_err, Option<T>) {
        let ee = p as *const libc::sock_extended_err;
        let err = unsafe { ptr::read_unaligned(ee) };

        // For errors originating on the network, SO_EE_OFFENDER(ee) points inside the p[..len]
        // CMSG_DATA buffer.  For local errors, there is no address included in the control
        // message, and SO_EE_OFFENDER(ee) points beyond the end of the buffer.  So, we need to
        // validate that the address object is in-bounds before we attempt to copy it.
        let addrp = unsafe { libc::SO_EE_OFFENDER(ee) as *const T };

        if unsafe { addrp.offset(1) } as usize - (p as usize) > len {
            (err, None)
        } else {
            (err, Some(unsafe { ptr::read_unaligned(addrp) }))
        }
    }
}

/// A type-safe zero-copy wrapper around a single control message, as used wih
/// [`sendmsg`](#fn.sendmsg).  More types may be added to this enum; do not
/// exhaustively pattern-match it.
///
/// [Further reading](https://man7.org/linux/man-pages/man3/cmsg.3.html)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ControlMessage<'a> {
    /// A message of type `SCM_RIGHTS`, containing an array of file
    /// descriptors passed between processes.
    ///
    /// See the description in the "Ancillary messages" section of the
    /// [unix(7) man page](https://man7.org/linux/man-pages/man7/unix.7.html).
    ///
    /// Using multiple `ScmRights` messages for a single `sendmsg` call isn't
    /// recommended since it causes platform-dependent behaviour: It might
    /// swallow all but the first `ScmRights` message or fail with `EINVAL`.
    /// Instead, you can put all fds to be passed into a single `ScmRights`
    /// message.
    ScmRights(&'a [RawFd]),
    /// A message of type `SCM_CREDENTIALS`, containing the pid, uid and gid of
    /// a process connected to the socket.
    ///
    /// This is similar to the socket option `SO_PEERCRED`, but requires a
    /// process to explicitly send its credentials. A process running as root is
    /// allowed to specify any credentials, while credentials sent by other
    /// processes are verified by the kernel.
    ///
    /// For further information, please refer to the
    /// [`unix(7)`](https://man7.org/linux/man-pages/man7/unix.7.html) man page.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    ScmCredentials(&'a UnixCredentials),
    /// A message of type `SCM_CREDS`, containing the pid, uid, euid, gid and groups of
    /// a process connected to the socket.
    ///
    /// This is similar to the socket options `LOCAL_CREDS` and `LOCAL_PEERCRED`, but
    /// requires a process to explicitly send its credentials.
    ///
    /// Credentials are always overwritten by the kernel, so this variant does have
    /// any data, unlike the receive-side
    /// [`ControlMessageOwned::ScmCreds`].
    ///
    /// For further information, please refer to the
    /// [`unix(4)`](https://www.freebsd.org/cgi/man.cgi?query=unix) man page.
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    ScmCreds,

    /// Set IV for `AF_ALG` crypto API.
    ///
    /// For further information, please refer to the
    /// [`documentation`](https://docs.kernel.org/crypto/userspace-if.html)
    #[cfg(any(
        target_os = "android",
        target_os = "linux",
    ))]
    AlgSetIv(&'a [u8]),
    /// Set crypto operation for `AF_ALG` crypto API. It may be one of
    /// `ALG_OP_ENCRYPT` or `ALG_OP_DECRYPT`
    ///
    /// For further information, please refer to the
    /// [`documentation`](https://docs.kernel.org/crypto/userspace-if.html)
    #[cfg(any(
        target_os = "android",
        target_os = "linux",
    ))]
    AlgSetOp(&'a libc::c_int),
    /// Set the length of associated authentication data (AAD) (applicable only to AEAD algorithms)
    /// for `AF_ALG` crypto API.
    ///
    /// For further information, please refer to the
    /// [`documentation`](https://docs.kernel.org/crypto/userspace-if.html)
    #[cfg(any(
        target_os = "android",
        target_os = "linux",
    ))]
    AlgSetAeadAssoclen(&'a u32),

    /// UDP GSO makes it possible for applications to generate network packets
    /// for a virtual MTU much greater than the real one.
    /// The length of the send data no longer matches the expected length on
    /// the wire.
    /// The size of the datagram payload as it should appear on the wire may be
    /// passed through this control message.
    /// Send buffer should consist of multiple fixed-size wire payloads
    /// following one by one, and the last, possibly smaller one.
    #[cfg(target_os = "linux")]
    #[cfg(feature = "net")]
    UdpGsoSegments(&'a u16),

    /// Configure the sending addressing and interface for v4.
    ///
    /// For further information, please refer to the
    /// [`ip(7)`](https://man7.org/linux/man-pages/man7/ip.7.html) man page.
    #[cfg(any(target_os = "linux",
              target_os = "macos",
              target_os = "netbsd",
              target_os = "android",
              target_os = "ios",))]
    #[cfg(feature = "net")]
    Ipv4PacketInfo(&'a libc::in_pktinfo),

    /// Configure the sending addressing and interface for v6.
    ///
    /// For further information, please refer to the
    /// [`ipv6(7)`](https://man7.org/linux/man-pages/man7/ipv6.7.html) man page.
    #[cfg(any(target_os = "linux",
              target_os = "macos",
              target_os = "netbsd",
              target_os = "freebsd",
              target_os = "android",
              target_os = "ios",))]
    #[cfg(feature = "net")]
    Ipv6PacketInfo(&'a libc::in6_pktinfo),

    /// Configure the IPv4 source address with `IP_SENDSRCADDR`.
    #[cfg(any(
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "dragonfly",
    ))]
    #[cfg(feature = "net")]
    Ipv4SendSrcAddr(&'a libc::in_addr),

    /// Configure the hop limit for v6 multicast traffic.
    ///
    /// Set the IPv6 hop limit for this message. The argument is an integer
    /// between 0 and 255. A value of -1 will set the hop limit to the route
    /// default if possible on the interface. Without this cmsg,  packets sent
    /// with sendmsg have a hop limit of 1 and will not leave the local network.
    /// For further information, please refer to the
    /// [`ipv6(7)`](https://man7.org/linux/man-pages/man7/ipv6.7.html) man page.
    #[cfg(any(target_os = "linux", target_os = "macos",
              target_os = "freebsd", target_os = "dragonfly",
              target_os = "android", target_os = "ios",
              target_os = "haiku"))]
    #[cfg(feature = "net")]
    Ipv6HopLimit(&'a libc::c_int),

    /// SO_RXQ_OVFL indicates that an unsigned 32 bit value
    /// ancilliary msg (cmsg) should be attached to recieved
    /// skbs indicating the number of packets dropped by the
    /// socket between the last recieved packet and this
    /// received packet.
    #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
    RxqOvfl(&'a u32),

    /// Configure the transmission time of packets.
    ///
    /// For further information, please refer to the
    /// [`tc-etf(8)`](https://man7.org/linux/man-pages/man8/tc-etf.8.html) man
    /// page.
    #[cfg(target_os = "linux")]
    TxTime(&'a u64),
}

// An opaque structure used to prevent cmsghdr from being a public type
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownCmsg(cmsghdr, Vec<u8>);

impl<'a> ControlMessage<'a> {
    /// The value of CMSG_SPACE on this message.
    /// Safe because CMSG_SPACE is always safe
    fn space(&self) -> usize {
        unsafe{CMSG_SPACE(self.len() as libc::c_uint) as usize}
    }

    /// The value of CMSG_LEN on this message.
    /// Safe because CMSG_LEN is always safe
    #[cfg(any(target_os = "android",
              all(target_os = "linux", not(target_env = "musl"))))]
    fn cmsg_len(&self) -> usize {
        unsafe{CMSG_LEN(self.len() as libc::c_uint) as usize}
    }

    #[cfg(not(any(target_os = "android",
                  all(target_os = "linux", not(target_env = "musl")))))]
    fn cmsg_len(&self) -> libc::c_uint {
        unsafe{CMSG_LEN(self.len() as libc::c_uint)}
    }

    /// Return a reference to the payload data as a byte pointer
    fn copy_to_cmsg_data(&self, cmsg_data: *mut u8) {
        let data_ptr = match *self {
            ControlMessage::ScmRights(fds) => {
                fds as *const _ as *const u8
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::ScmCredentials(creds) => {
                &creds.0 as *const libc::ucred as *const u8
            }
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            ControlMessage::ScmCreds => {
                // The kernel overwrites the data, we just zero it
                // to make sure it's not uninitialized memory
                unsafe { ptr::write_bytes(cmsg_data, 0, self.len()) };
                return
            }
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetIv(iv) => {
                #[allow(deprecated)] // https://github.com/rust-lang/libc/issues/1501
                let af_alg_iv = libc::af_alg_iv {
                    ivlen: iv.len() as u32,
                    iv: [0u8; 0],
                };

                let size = mem::size_of_val(&af_alg_iv);

                unsafe {
                    ptr::copy_nonoverlapping(
                        &af_alg_iv as *const _ as *const u8,
                        cmsg_data,
                        size,
                    );
                    ptr::copy_nonoverlapping(
                        iv.as_ptr(),
                        cmsg_data.add(size),
                        iv.len()
                    );
                };

                return
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetOp(op) => {
                op as *const _ as *const u8
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetAeadAssoclen(len) => {
                len as *const _ as *const u8
            },
            #[cfg(target_os = "linux")]
            #[cfg(feature = "net")]
            ControlMessage::UdpGsoSegments(gso_size) => {
                gso_size as *const _ as *const u8
            },
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "netbsd", target_os = "android",
                      target_os = "ios",))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv4PacketInfo(info) => info as *const _ as *const u8,
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "netbsd", target_os = "freebsd",
                      target_os = "android", target_os = "ios",))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv6PacketInfo(info) => info as *const _ as *const u8,
            #[cfg(any(target_os = "netbsd", target_os = "freebsd",
                      target_os = "openbsd", target_os = "dragonfly"))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv4SendSrcAddr(addr) => addr as *const _ as *const u8,
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "freebsd", target_os = "dragonfly",
                      target_os = "android", target_os = "ios",
                      target_os = "haiku"))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv6HopLimit(limit) => limit as *const _ as *const u8,
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            ControlMessage::RxqOvfl(drop_count) => {
                drop_count as *const _ as *const u8
            },
            #[cfg(target_os = "linux")]
            ControlMessage::TxTime(tx_time) => {
                tx_time as *const _ as *const u8
            },
        };
        unsafe {
            ptr::copy_nonoverlapping(
                data_ptr,
                cmsg_data,
                self.len()
            )
        };
    }

    /// The size of the payload, excluding its cmsghdr
    fn len(&self) -> usize {
        match *self {
            ControlMessage::ScmRights(fds) => {
                mem::size_of_val(fds)
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::ScmCredentials(creds) => {
                mem::size_of_val(creds)
            }
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            ControlMessage::ScmCreds => {
                mem::size_of::<libc::cmsgcred>()
            }
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetIv(iv) => {
                mem::size_of::<&[u8]>() + iv.len()
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetOp(op) => {
                mem::size_of_val(op)
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetAeadAssoclen(len) => {
                mem::size_of_val(len)
            },
            #[cfg(target_os = "linux")]
            #[cfg(feature = "net")]
            ControlMessage::UdpGsoSegments(gso_size) => {
                mem::size_of_val(gso_size)
            },
            #[cfg(any(target_os = "linux", target_os = "macos",
              target_os = "netbsd", target_os = "android",
              target_os = "ios",))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv4PacketInfo(info) => mem::size_of_val(info),
            #[cfg(any(target_os = "linux", target_os = "macos",
              target_os = "netbsd", target_os = "freebsd",
              target_os = "android", target_os = "ios",))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv6PacketInfo(info) => mem::size_of_val(info),
            #[cfg(any(target_os = "netbsd", target_os = "freebsd",
                      target_os = "openbsd", target_os = "dragonfly"))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv4SendSrcAddr(addr) => mem::size_of_val(addr),
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "freebsd", target_os = "dragonfly",
                      target_os = "android", target_os = "ios",
                      target_os = "haiku"))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv6HopLimit(limit) => {
                mem::size_of_val(limit)
            },
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            ControlMessage::RxqOvfl(drop_count) => {
                mem::size_of_val(drop_count)
            },
            #[cfg(target_os = "linux")]
            ControlMessage::TxTime(tx_time) => {
                mem::size_of_val(tx_time)
            },
        }
    }

    /// Returns the value to put into the `cmsg_level` field of the header.
    fn cmsg_level(&self) -> libc::c_int {
        match *self {
            ControlMessage::ScmRights(_) => libc::SOL_SOCKET,
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::ScmCredentials(_) => libc::SOL_SOCKET,
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            ControlMessage::ScmCreds => libc::SOL_SOCKET,
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetIv(_) | ControlMessage::AlgSetOp(_) |
                ControlMessage::AlgSetAeadAssoclen(_) => libc::SOL_ALG,
            #[cfg(target_os = "linux")]
            #[cfg(feature = "net")]
            ControlMessage::UdpGsoSegments(_) => libc::SOL_UDP,
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "netbsd", target_os = "android",
                      target_os = "ios",))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv4PacketInfo(_) => libc::IPPROTO_IP,
            #[cfg(any(target_os = "linux", target_os = "macos",
              target_os = "netbsd", target_os = "freebsd",
              target_os = "android", target_os = "ios",))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv6PacketInfo(_) => libc::IPPROTO_IPV6,
            #[cfg(any(target_os = "netbsd", target_os = "freebsd",
                      target_os = "openbsd", target_os = "dragonfly"))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv4SendSrcAddr(_) => libc::IPPROTO_IP,
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "freebsd", target_os = "dragonfly",
                      target_os = "android", target_os = "ios",
                      target_os = "haiku"))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv6HopLimit(_) => libc::IPPROTO_IPV6,
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            ControlMessage::RxqOvfl(_) => libc::SOL_SOCKET,
            #[cfg(target_os = "linux")]
            ControlMessage::TxTime(_) => libc::SOL_SOCKET,
        }
    }

    /// Returns the value to put into the `cmsg_type` field of the header.
    fn cmsg_type(&self) -> libc::c_int {
        match *self {
            ControlMessage::ScmRights(_) => libc::SCM_RIGHTS,
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::ScmCredentials(_) => libc::SCM_CREDENTIALS,
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            ControlMessage::ScmCreds => libc::SCM_CREDS,
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetIv(_) => {
                libc::ALG_SET_IV
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetOp(_) => {
                libc::ALG_SET_OP
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetAeadAssoclen(_) => {
                libc::ALG_SET_AEAD_ASSOCLEN
            },
            #[cfg(target_os = "linux")]
            #[cfg(feature = "net")]
            ControlMessage::UdpGsoSegments(_) => {
                libc::UDP_SEGMENT
            },
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "netbsd", target_os = "android",
                      target_os = "ios",))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv4PacketInfo(_) => libc::IP_PKTINFO,
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "netbsd", target_os = "freebsd",
                      target_os = "android", target_os = "ios",))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv6PacketInfo(_) => libc::IPV6_PKTINFO,
            #[cfg(any(target_os = "netbsd", target_os = "freebsd",
                      target_os = "openbsd", target_os = "dragonfly"))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv4SendSrcAddr(_) => libc::IP_SENDSRCADDR,
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "freebsd", target_os = "dragonfly",
                      target_os = "android", target_os = "ios",
                      target_os = "haiku"))]
            #[cfg(feature = "net")]
            ControlMessage::Ipv6HopLimit(_) => libc::IPV6_HOPLIMIT,
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            ControlMessage::RxqOvfl(_) => {
                libc::SO_RXQ_OVFL
            },
            #[cfg(target_os = "linux")]
            ControlMessage::TxTime(_) => {
                libc::SCM_TXTIME
            },
        }
    }

    // Unsafe: cmsg must point to a valid cmsghdr with enough space to
    // encode self.
    unsafe fn encode_into(&self, cmsg: *mut cmsghdr) {
        unsafe {
            (*cmsg).cmsg_level = self.cmsg_level();
            (*cmsg).cmsg_type = self.cmsg_type();
            (*cmsg).cmsg_len = self.cmsg_len();
            self.copy_to_cmsg_data( CMSG_DATA(cmsg) );
        }
    }
}

/// Variants to be used with [`cmsg_space!`].
///
/// You shouldn't need to use this type directly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ControlMessageOwnedSpace {
    /// See [`ControlMessageOwned::ScmRights`].
    ///
    /// Argument is the number of file descriptors.
    ScmRights(usize),
    /// See [`ControlMessageOwned::ScmCredentials`].
    #[cfg(any(target_os = "android", target_os = "linux"))]
    ScmCredentials,
    /// See [`ControlMessageOwned::ScmCreds`].
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    ScmCreds,
    /// See [`ControlMessageOwned::ScmTimestamp`].
    ScmTimestamp,
    /// See [`ControlMessageOwned::ScmTimestampns`].
    #[cfg(any(target_os = "android", target_os = "linux"))]
    ScmTimestampsns,
    /// See [`ControlMessageOwned::ScmTimestampns`].
    #[cfg(any(target_os = "android", target_os = "linux"))]
    ScmTimestampns,
    /// See [`ControlMessageOwned::Ipv4PacketInfo`].
    #[cfg(any(
        target_os = "android",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
    ))]
    #[cfg(feature = "net")]
    Ipv4PacketInfo,
    /// See [`ControlMessageOwned::Ipv6PacketInfo`].
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "netbsd",
    ))]
    #[cfg(feature = "net")]
    Ipv6PacketInfo,
    /// See [`ControlMessageOwned::Ipv4RecvIf`].
    #[cfg(any(
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    #[cfg(feature = "net")]
    Ipv4RecvIf,
    /// See [`ControlMessageOwned::Ipv4RecvDstAddr`].
    #[cfg(any(
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    #[cfg(feature = "net")]
    Ipv4RecvDstAddr,
    /// See [`ControlMessageOwned::Ipv4OrigDstAddr`].
    #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
    #[cfg(feature = "net")]
    Ipv4OrigDstAddr,
    /// See [`ControlMessageOwned::Ipv6OrigDstAddr`].
    #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
    #[cfg(feature = "net")]
    Ipv6OrigDstAddr,
    /// See [`ControlMessageOwned::UdpGroSegments`].
    #[cfg(target_os = "linux")]
    #[cfg(feature = "net")]
    UdpGroSegments,
    /// See [`ControlMessageOwned::RxqOvfl`].
    #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
    RxqOvfl,
    /// See [`ControlMessageOwned::Ipv4RecvErr`].
    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[cfg(feature = "net")]
    Ipv4RecvErr,
    /// See [`ControlMessageOwned::Ipv6RecvErr`].
    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[cfg(feature = "net")]
    Ipv6RecvErr,
    /// See [`ControlMessageOwned::TlsGetRecordType`].
    #[cfg(target_os = "linux")]
    TlsGetRecordType,
}

impl ControlMessageOwnedSpace {
    const fn len(self) -> usize {
        match self {
            Self::ScmRights(n) => n * mem::size_of::<RawFd>(),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            Self::ScmCredentials => mem::size_of::<UnixCredentials>(),
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            Self::ScmCreds => mem::size_of::<UnixCredentials>(),
            Self::ScmTimestamp => mem::size_of::<TimeVal>(),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            Self::ScmTimestampsns => mem::size_of::<Timestamps>(),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            Self::ScmTimestampns => mem::size_of::<TimeSpec>(),
            #[cfg(any(
                target_os = "android",
                target_os = "ios",
                target_os = "linux",
                target_os = "macos",
                target_os = "netbsd",
            ))]
            #[cfg(feature = "net")]
            Self::Ipv4PacketInfo => mem::size_of::<libc::in_pktinfo>(),
            #[cfg(any(
                target_os = "android",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "linux",
                target_os = "macos",
                target_os = "openbsd",
                target_os = "netbsd",
            ))]
            #[cfg(feature = "net")]
            Self::Ipv6PacketInfo => mem::size_of::<libc::in6_pktinfo>(),
            #[cfg(any(
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            #[cfg(feature = "net")]
            Self::Ipv4RecvIf => mem::size_of::<libc::sockaddr_dl>(),
            #[cfg(any(
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            #[cfg(feature = "net")]
            Self::Ipv4RecvDstAddr => mem::size_of::<libc::in_addr>(),
            #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
            #[cfg(feature = "net")]
            Self::Ipv4OrigDstAddr => mem::size_of::<libc::sockaddr_in>(),
            #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
            #[cfg(feature = "net")]
            Self::Ipv6OrigDstAddr => mem::size_of::<libc::sockaddr_in6>(),
            #[cfg(target_os = "linux")]
            #[cfg(feature = "net")]
            Self::UdpGroSegments => mem::size_of::<u16>(),
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            Self::RxqOvfl => mem::size_of::<u32>(),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            #[cfg(feature = "net")]
            Self::Ipv4RecvErr => {
                mem::size_of::<libc::sock_extended_err>() + mem::size_of::<libc::sockaddr_in>()
            }
            #[cfg(any(target_os = "android", target_os = "linux"))]
            #[cfg(feature = "net")]
            Self::Ipv6RecvErr => {
                mem::size_of::<libc::sock_extended_err>() + mem::size_of::<libc::sockaddr_in6>()
            }
            #[cfg(target_os = "linux")]
            Self::TlsGetRecordType => mem::size_of::<TlsGetRecordType>(),
        }
    }

    #[doc(hidden)]
    pub const fn space(self) -> usize {
        // SAFETY: CMSG_SPACE has no sideeffects and is always safe.
        unsafe { CMSG_SPACE(self.len() as libc::c_uint) as usize }
    }
}

/// Sends a message through a connection-mode or connectionless-mode socket.
///
/// If the socket is a connectionless-mode socket, the message will *usually* be sent
/// to the address passed in `addr`. Click [here] for more information.
///
/// If the socket is connection-mode, `addr` will be ignored. In that case, using
/// [`Addr::empty`] for `addr` is recommended.
///
/// Additionally to [`sendto`], it also allows to send control messages.
///
/// [Further reading]
///
/// # Examples
///
/// See [`recvmsg`] for an example using both functions.
///
/// [here]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendmsg.html
/// [Further reading]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendmsg.html
pub fn sendmsg<'a, A, I>(
    fd: RawFd,
    addr: A,
    iov: &I,
    cmsgs: &CmsgStr,
    flags: MsgFlags,
) -> Result<SendMsgResult>
where
    A: AsRef<Addr>,
    I: AsRef<[IoSlice<'a>]>,
{
    let header = sendmsg_header(addr.as_ref(), iov.as_ref(), cmsgs);

    let ret = unsafe { libc::sendmsg(fd, &header, flags.bits()) };

    let bytes = Errno::result(ret).map(|x| x as usize)?;

    Ok(SendMsgResult { bytes })
}

/// Receives a message from a connection-mode or connectionless-mode socket.
///
/// It is normally used with connectionless-mode sockets because it permits the application to
/// retrieve the source address of received data.
///
/// Additionally to [`recvfrom`], it also allows to receive control messages.
///
/// [Further reading]
///
/// # Examples
///
/// The following example runs on Linux and Android only.
///
#[cfg_attr(any(target_os = "linux", target_os = "android"), doc = "```")]
#[cfg_attr(not(any(target_os = "linux", target_os = "android")), doc = "```ignore")]
/// # use nix::sys::socket::*;
/// # use nix::cmsg_buf;
/// # use nix::sys::socket::sockopt::Timestamping;
/// # use std::os::fd::AsRawFd;
/// # use std::io::{IoSlice, IoSliceMut};
/// // We use connectionless UDP sockets.
/// let send = socket(
///     AddressFamily::INET,
///     SockType::Datagram,
///     SockFlag::empty(),
///     Some(SockProtocol::Udp),
/// )?;
///
/// let recv = socket(
///     AddressFamily::INET,
///     SockType::Datagram,
///     SockFlag::empty(),
///     Some(SockProtocol::Udp),
/// )?;
///
/// // We enable timestamping on the receiving socket. They will be sent as
/// // control messages by the kernel.
/// setsockopt(&recv, Timestamping, &TimestampingFlag::all())?;
///
/// // This is the address we are going to send the message.
/// let addr = "127.0.0.1:6069".parse::<Ipv4Address>().unwrap();
///
/// bind(recv.as_raw_fd(), &addr)?;
///
/// // The message we are trying to send: [0, 1, 2, ...].
/// let msg: [u8; 1500] = std::array::from_fn(|i| i as u8);
///
/// // Send `msg` on `send` without control messages.
/// //
/// // On connectionless sockets like UDP, the destination address is required.
/// // On connection-oriented sockets like TCP, `addr` would be ignored
/// // and `None` is usually passed instead.
/// let send_res = sendmsg(
///     send.as_raw_fd(),
///     &addr,
///     &[IoSlice::new(&msg)],
///     CmsgStr::empty(),
///     MsgFlags::empty(),
/// )?;
///
/// // We have actually sent 1500 bytes.
/// assert_eq!(send_res.bytes(), 1500);
///
/// // Initialize a buffer to receive `msg`.
/// let mut buf = [0u8; 1500];
///
/// // The timestamps will land here. The control message type is `ScmTimestampsns`.
/// let mut cmsg = cmsg_buf![ScmTimestampsns];
///
/// // Receive `msg` on `recv`.
/// let recv_res = recvmsg(
///     recv.as_raw_fd(),
///     &mut [IoSliceMut::new(&mut buf)],
///     cmsg.handle(),
///     MsgFlags::empty(),
/// )?;
///
/// // We have actually received 1500 bytes.
/// assert_eq!(recv_res.bytes(), 1500);
///
/// // Since this is a connectionless socket, the sender address is returned.
/// // On connection-oriented sockets like TCP, this would be `None`.
/// assert!(recv_res.address().family() == AddressFamily::INET);
///
/// // The received message is identical to the sent one.
/// assert_eq!(buf, msg);
///
/// // We have received a control message containing a timestamp.
/// assert!(matches!(
///     cmsg.iter().next(),
///     Some(ControlMessageOwned::ScmTimestampsns(_)),
/// ));
/// # Ok::<(), nix::Error>(())
/// ```
///
/// [Further reading]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/recvmsg.html
pub fn recvmsg(
    fd: RawFd,
    iov: &mut [IoSliceMut<'_>],
    mut cmsg_buffer: CmsgBufHandle<'_>,
    flags: MsgFlags,
) -> Result<RecvMsgResult> {
    let mut addr_buf = Address::default();

    let mut header = recvmsg_header(addr_buf.as_mut_ptr(), iov, &mut cmsg_buffer);

    let ret = unsafe { libc::recvmsg(fd, &mut header, flags.bits()) };

    let bytes = Errno::result(ret).map(|x| x as usize)?;

    if let Some(len) = cmsg_buffer.len {
        *len = header.msg_controllen as _;
    }

    cfg_if! {
        if #[cfg(any(
            target_os = "android",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "linux",
            target_os = "redox",
        ))] {
            addr_buf.len = header.msg_namelen as _;
        }
    }

    Ok(RecvMsgResult { bytes, hdr: header, addr_buf })
}

/// Primitive for encoded control messages that can be **sent**.
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(not(doc), repr(transparent))]
pub struct CmsgStr {
    slice: [u8],
}

impl CmsgStr {
    /// Creates an empty [`CmsgStr`] with zero length.
    pub const fn empty() -> &'static Self {
        unsafe { Self::from_bytes_unchecked(&[]) }
    }

    /// Creates a new [`CmsgStr`] from the given bytes.
    ///
    /// [`write_cmsg_into`] can be used to encode an iterator of [`ControlMessage`]s
    /// into a buffer.
    ///
    /// # Safety
    ///
    /// The given bytes must contain valid encoded control messages.
    pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { &*(bytes as *const [u8] as *const Self) }
    }

    /// Returns a raw pointer to the buffer.
    pub const fn as_ptr(&self) -> *const u8 {
        self.slice.as_ptr()
    }

    /// Returns the length of the buffer.
    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    /// Returns whether the buffer is empty.
    pub const fn is_empty(&self) -> bool {
        self.slice.len() == 0
    }
}

impl Default for &CmsgStr {
    fn default() -> Self {
        CmsgStr::empty()
    }
}

/// Mutable handle to an control message buffer to be used with [`recvmsg`].
///
/// The handle contains a mutable reference to the (uninitialized) buffer and a
/// mutable reference to the length of the buffer, which is used to update the
/// length of valid control messages.
#[derive(Debug)]
pub struct CmsgBufHandle<'a> {
    buf: &'a mut [MaybeUninit<u8>],
    // Invariant: `len` is always `Some` if `buf` is not empty.
    len: Option<&'a mut usize>,
}

impl<'a> CmsgBufHandle<'a> {
    /// An empty handle.
    ///
    /// Use this if no control messages are needed.
    pub fn empty() -> Self {
        Self { buf: &mut [], len: None }
    }

    /// Creates a new handle for the given buffer and length.
    ///
    /// # Safety
    ///
    /// Normally casting a `&mut [u8]` to `&mut [MaybeUninit<u8>]` would be unsound,
    /// as that allows us to write uninitialised bytes to the buffer. However this
    /// implementation promises to not write uninitialised bytes to the buffer and
    /// passes it directly to the system call. This promise ensures that this
    /// function can be called using a buffer of type `&mut [u8]`.

    // Safety doc based on https://docs.rs/socket2/latest/socket2/struct.Socket.html#safety.
    pub fn new(buf: &'a mut [MaybeUninit<u8>], len: &'a mut usize) -> Self {
        Self { buf, len: Some(len) }
    }

    fn capacity(&self) -> usize {
        self.buf.len()
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr().cast()
    }
}

impl Default for CmsgBufHandle<'_> {
    fn default() -> Self {
        Self::empty()
    }
}

/// Writes the given control messages into the given buffer.
///
/// Buffers are zero-initialized before writing. If the buffer is already zero-initialized,
/// consider using [`write_cmsg_into_unchecked`] instead.
///
/// Returns the number of bytes written into the buffer. If the buffer was too small,
/// the first control message that didn't fit in is returned additionally.
pub fn write_cmsg_into<'a, I>(
    buf: &mut [u8],
    cmsg: I,
) -> std::result::Result<usize, (usize, ControlMessage<'a>)>
where
    I: IntoIterator<Item = ControlMessage<'a>>,
{
    buf.iter_mut().for_each(|b| *b = 0);

    // SAFETY: `buf` has been zero-initialized.
    unsafe {
        write_cmsg_into_unchecked(buf, cmsg)
    }
}

/// Writes the given control messages into the given buffer.
///
/// Returns the number of bytes written into the buffer. If the buffer was too small,
/// the first control message that didn't fit in is returned additionally.
///
/// # Safety
///
/// `buf` must be zero-initialized before calling this function.
pub unsafe fn write_cmsg_into_unchecked<'a, I>(
    buf: &mut [u8],
    cmsg: I,
) -> std::result::Result<usize, (usize, ControlMessage<'a>)>
where
    I: IntoIterator<Item = ControlMessage<'a>>,
{
    let mut mhdr = cmsg_dummy_mhdr(buf.as_mut_ptr(), buf.len());

    // SAFETY: call to extern function without sideeffects. We need to start from a mutable
    // reference before casting it to a `*const` as we want to use the resulting pointer mutably.
    let mut cmsg_ptr = unsafe { CMSG_FIRSTHDR(mhdr.as_mut_ptr().cast_const()) };

    let mut written = 0;

    let mut cmsg = cmsg.into_iter();

    for c in cmsg.by_ref() {
        if cmsg_ptr.is_null() || c.space() > buf.len() - written {
            return Err((written, c));
        }

        written += c.space();

        // SAFETY: we checked that there is enough space in `buf`.
        // Additionally, relies on `CMSG_FIRSTHDR` and `CMSG_NXTHDR` for safety.
        // `CMSG_FIRSTHDR` and `CMSG_NXTHDR` shouldn't care about the other
        // uninitialized fields of `mhdr`.
        //
        // See https://man7.org/linux/man-pages/man3/cmsg.3.html.
        unsafe {
            c.encode_into(cmsg_ptr.cast());
        }

        // SAFETY: call to extern function without sideeffects. We need to start from a mutable
        // reference before casting it to a `*const` as we want to use the resulting pointer mutably.
        cmsg_ptr = unsafe { CMSG_NXTHDR(mhdr.as_mut_ptr().cast_const(), cmsg_ptr) };
    }

    Ok(written)
}

// FIXME: make `const` once possible in stable rust. Last checked: 1.73.0.
fn cmsg_dummy_mhdr(buf: *mut u8, len: usize) -> MaybeUninit<libc::msghdr> {
    let mut mhdr = MaybeUninit::<libc::msghdr>::zeroed();

    // SAFETY: using `ptr::write` to not drop the old uninitialized value and `addr_of_mut`
    // to not create references of `libc::msghdr` along the way.
    unsafe {
        addr_of_mut!((*mhdr.as_mut_ptr()).msg_control).write(buf.cast());
        addr_of_mut!((*mhdr.as_mut_ptr()).msg_controllen).write(len as _);
    }

    mhdr
}

/// Returns the exact number of bytes required to hold the given control messages.
pub fn cmsg_space_iter<'a, I>(cmsg: I) -> usize
where
    I: IntoIterator<Item = ControlMessage<'a>>,
{
    cmsg.into_iter().map(|c| c.space()).sum()
}

/// Non-extendable heap-allocated container for holding control messages
/// that can be **sent**.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CmsgVec {
    inner: Vec<u8>,
}

impl CmsgVec {
    /// Returns an empty [`CmsgVec`].
    ///
    /// No allocations are performed. Use this if no control messages are needed.
    pub const fn empty() -> Self {
        Self { inner: Vec::new() }
    }

    /// Returns an empty [`CmsgVec`] with the given capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self { inner: Vec::with_capacity(cap) }
    }

    /// Allocates a buffer that contains the given control messages.
    ///
    /// The `size` parameter determines the size of the allocation in bytes.
    /// [`cmsg_space_iter`] can be used to calculate the exact number of bytes required to
    /// hold the control messages.
    ///
    /// If `size` is too small, the first control message that didn't fit in is returned additionally.
    pub fn from_iter<'a, I>(
        cmsg: I,
        size: usize,
    ) -> std::result::Result<Self, (Self, ControlMessage<'a>)>
    where
        I: IntoIterator<Item = ControlMessage<'a>>,
    {
        let mut cmsg_buf = vec![0; size];

        // SAFETY: `cmsg_buf` is zero-initialized.
        match unsafe { write_cmsg_into_unchecked(&mut cmsg_buf, cmsg) } {
            Ok(written) => {
                cmsg_buf.truncate(written);

                Ok(Self {
                    inner: cmsg_buf,
                })
            }
            Err((written, i)) => {
                cmsg_buf.truncate(written);

                Err((
                    Self {
                        inner: cmsg_buf,
                    },
                    i,
                ))
            }
        }
    }

    /// Allocates a buffer that contains the given control messages.
    ///
    /// This is a shorthand for calling [`cmsg_space_iter`] with the cloned iterator,
    /// followed by [`Self::from_iter`].
    pub fn from_iter_clone<'a, I>(cmsg: I) -> Self
    where
        I: IntoIterator<Item = ControlMessage<'a>>,
        I::IntoIter: Clone,
    {
        let cmsg = cmsg.into_iter();

        let len = cmsg_space_iter(cmsg.clone());

        Self::from_iter(cmsg, len).unwrap()
    }

    /// Writes the given control messages into the buffer, replacing the previous contents.
    ///
    /// The `size` parameter determines the minimum size of the allocation in bytes, but the internal
    /// storage might allocate more. [`cmsg_space_iter`] can be used to calculate the exact number of
    /// bytes required to hold the control messages.
    ///
    /// If `size` is too small, the first control message that didn't fit in is returned additionally.
    ///
    /// This function does not allocate if `size` is smaller than the current capacity.
    pub fn write_iter<'a, I>(
        &mut self,
        cmsg: I,
        size: usize,
    ) -> std::result::Result<(), ControlMessage<'a>>
    where
        I: IntoIterator<Item = ControlMessage<'a>>,
    {
        self.inner.clear();
        self.inner.reserve(size);

        (0..size).for_each(|_| self.inner.push(0));

        match unsafe { write_cmsg_into_unchecked(&mut self.inner, cmsg) } {
            Ok(written) => {
                self.inner.truncate(written);

                Ok(())
            }
            Err((written, i)) => {
                self.inner.truncate(written);

                Err(i)
            }
        }
    }

    /// Writes the given control messages into the buffer, replacing the previous contents.
    ///
    /// This is a shorthand for calling [`cmsg_space_iter`] with the cloned iterator,
    /// followed by [`Self::write_iter`].
    ///
    /// This function does not allocate if the calculated size is smaller than the current capacity.
    pub fn write_iter_clone<'a, I>(&mut self, cmsg: I)
    where
        I: IntoIterator<Item = ControlMessage<'a>>,
        I::IntoIter: Clone,
    {
        let cmsg = cmsg.into_iter();

        let len = cmsg_space_iter(cmsg.clone());

        self.write_iter(cmsg, len).unwrap();
    }

    /// Writes the given control messages into the buffer, replacing the previous contents.
    ///
    /// This function does not allocate. If the current allocation can't hold all control messages,
    /// the first control message that didn't fit in is returned additionally.
    pub fn write_iter_in_place<'a, I>(&mut self, cmsg: I) -> std::result::Result<(), ControlMessage<'a>>
    where
        I: IntoIterator<Item = ControlMessage<'a>>,
    {
        self.write_iter(cmsg, self.inner.capacity())
    }

    /// Returns the length of the buffer.
    ///
    /// This is number of bytes that contain valid control messages.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the buffer contains no control messages.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the capacity of the buffer.
    ///
    /// This is the number of bytes that can be written to the buffer.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Clears the buffer, removing all control messages.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Reserves extra capacity for the buffer.
    ///
    /// The buffer will be able to hold at least `additional` more bytes
    /// than its current length.
    /// If there is already sufficient space, nothing happens.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX`.
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Shrinks the capacity of the buffer to at least the maximum of its length
    /// and the given minimum capacity.
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity);
    }

    /// Shrinks the capacity of the buffer as close as possible to its length.
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }
}

impl std::ops::Deref for CmsgVec {
    type Target = CmsgStr;

    fn deref(&self) -> &Self::Target {
        // SAFETY: `CmsgVecWrite` is a newtype wrapper around `Vec<u8>`.
        // `self.inner[..]` is guaranteed to be a valid `CmsgStr`.
        unsafe { CmsgStr::from_bytes_unchecked(&self.inner) }
    }
}

/// Heap-allocated buffer for **receiving** control messages.
///
/// # Cloning
///
/// In the current implementation, when cloning the buffer, the capacity
/// is cloned as well, meaning that the cloned allocation will have the same
/// size as the original one. **This could change in future versions of nix**,
/// but not without increasing its major version.
#[derive(Debug, Clone)]
pub struct CmsgBuf {
    inner: Vec<MaybeUninit<u8>>,
    len: usize,
}

impl CmsgBuf {
    /// Returns an empty [`CmsgBuf`].
    ///
    /// This function doesn't allocate.
    pub const fn empty() -> Self {
        Self { inner: Vec::new(), len: 0 }
    }

    /// Returns an empty [`CmsgBuf`] with the given capacity.
    pub fn with_capacity(cap: usize) -> Self {
        let mut inner = Vec::with_capacity(cap);

        // SAFETY: `MaybeUninit` doesn't require initialization, and the length matches
        // the capacity.
        unsafe {
            inner.set_len(cap);
        }

        Self { inner, len: 0 }
    }

    /// Returns the capacity of the buffer.
    pub fn capacity(&self) -> usize {
        debug_assert_eq!(self.inner.len(), self.inner.capacity());

        self.inner.capacity()
    }

    /// Returns the length of the buffer with valid control messages.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the buffer contains no control messages.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a mutable handle to the buffer to be used with [`recvmsg`].
    pub fn handle(&mut self) -> CmsgBufHandle<'_> {
        CmsgBufHandle::new(&mut self.inner, &mut self.len)
    }

    /// Returns an iterator over the control messages in the buffer.
    pub fn iter(&self) -> CmsgIterator<'_> {
        if self.len == 0 {
            let mhdr = cmsg_dummy_mhdr(ptr::null_mut(), 0);

            return CmsgIterator { cmsghdr: None, mhdr };
        }

        let mhdr = cmsg_dummy_mhdr(self.inner.as_ptr().cast_mut().cast(), self.len);

        CmsgIterator {
            cmsghdr: unsafe { CMSG_FIRSTHDR(mhdr.as_ptr()).as_ref() },
            mhdr,
        }
    }
}

impl Default for CmsgBuf {
    fn default() -> Self {
        Self::empty()
    }
}

fn sendmsg_header(
    addr: &Addr,
    iov: &[IoSlice<'_>],
    cmsg: &CmsgStr,
) -> libc::msghdr {
    let (addr_ptr, addr_len) = if addr.len() == 0 {
        (ptr::null(), 0)
    } else {
        (addr.as_ptr(), addr.len())
    };

    let (iov_ptr, iov_len) = (iov.as_ptr(), iov.len());
    let (cmsg_ptr, cmsg_len) = if cmsg.is_empty() {
        (ptr::null(), 0)
    } else {
        (cmsg.as_ptr(), cmsg.len())
    };

    let mut msg_hdr = MaybeUninit::<libc::msghdr>::zeroed();
    let msg_hdr_ptr = msg_hdr.as_mut_ptr();

    unsafe {
        addr_of_mut!((*msg_hdr_ptr).msg_name).write(addr_ptr.cast_mut().cast());
        addr_of_mut!((*msg_hdr_ptr).msg_namelen).write(addr_len as _);
        addr_of_mut!((*msg_hdr_ptr).msg_iov).write(iov_ptr.cast_mut().cast());
        addr_of_mut!((*msg_hdr_ptr).msg_iovlen).write(iov_len as _);
        addr_of_mut!((*msg_hdr_ptr).msg_control).write(cmsg_ptr.cast_mut().cast());
        addr_of_mut!((*msg_hdr_ptr).msg_controllen).write(cmsg_len as _);
    }

    unsafe { msg_hdr.assume_init() }
}

fn recvmsg_header(
    addr: *mut libc::sockaddr_storage,
    iov: &mut [IoSliceMut<'_>],
    cmsg: &mut CmsgBufHandle<'_>,
) -> libc::msghdr {
    let (iov_ptr, iov_len) = (iov.as_mut().as_mut_ptr(), iov.as_mut().len());
    let (cmsg_ptr, cmsg_len) = if cmsg.capacity() == 0 {
        (ptr::null_mut(), 0)
    } else {
        (cmsg.as_mut_ptr(), cmsg.capacity())
    };

    let addr_size = mem::size_of::<libc::sockaddr_storage>();

    let (addr_ptr, addr_len) = if addr_size == 0 {
        (ptr::null_mut(), 0)
    } else {
        (addr.cast(), addr_size)
    };

    let mut msg_hdr = MaybeUninit::<libc::msghdr>::zeroed();
    let msg_hdr_ptr = msg_hdr.as_mut_ptr();

    unsafe {
        addr_of_mut!((*msg_hdr_ptr).msg_name).write(addr_ptr);
        addr_of_mut!((*msg_hdr_ptr).msg_namelen).write(addr_len as _);
        addr_of_mut!((*msg_hdr_ptr).msg_iov).write(iov_ptr.cast());
        addr_of_mut!((*msg_hdr_ptr).msg_iovlen).write(iov_len as _);
        addr_of_mut!((*msg_hdr_ptr).msg_control).write(cmsg_ptr.cast());
        addr_of_mut!((*msg_hdr_ptr).msg_controllen).write(cmsg_len as _);
    }

    unsafe { msg_hdr.assume_init() }
}

/// Growable container holding the headers for [`sendmmsg`].
///
/// This allocation can be reused when calling [`sendmmsg`] multiple times,
/// which can be beneficial for performance.
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
#[derive(Debug, Clone, Default)]
pub struct SendMmsgHeaders {
    mmsghdrs: Vec<libc::mmsghdr>,
    sent: usize,
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
impl SendMmsgHeaders {
    /// Creates a new container for the mmsg-headers.
    ///
    /// No allocations are performed.
    pub const fn new() -> Self {
        Self {
            mmsghdrs: Vec::new(),
            sent: 0,
        }
    }

    /// Creates a new container and reserves space for `cap` mmsg-headers.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            mmsghdrs: Vec::with_capacity(cap),
            sent: 0,
        }
    }

    /// Returns an iterator over [`SendMsgResult`], which contains the
    /// metadata of the sent messages.
    pub fn iter(&self) -> SendMmsgIter<'_> {
        SendMmsgIter { hdrs: self.mmsghdrs[..self.sent].iter() }
    }

    fn fill_send<'a, I>(&mut self, mut items: I)
    where
        I: Iterator<Item = (&'a Addr, &'a [IoSlice<'a>], &'a CmsgStr)> + ExactSizeIterator,
    {
        // For panic-safety
        self.sent = 0;

        let len = items.len();

        self.mmsghdrs.clear();
        self.mmsghdrs.reserve(len);

        let mut total = 0;

        for (i, (addr, iov, cmsg)) in items.by_ref().take(len).enumerate() {
            let mmsg_hdr = libc::mmsghdr {
                msg_hdr: sendmsg_header(addr, iov, cmsg),
                msg_len: 0,
            };

            self.mmsghdrs.push(mmsg_hdr);

            total = i + 1;
        }


        if total != len || items.next().is_some() {
            panic!("Len returned by exact size iterator was not accurate");
        }
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
unsafe impl Send for SendMmsgHeaders {}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
unsafe impl Sync for SendMmsgHeaders {}

/// An iterator returned by [`SendMmsgHeaders::iter`].
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
#[derive(Debug, Clone)]
pub struct SendMmsgIter<'a> {
    hdrs: std::slice::Iter<'a, libc::mmsghdr>,
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
impl<'a> Iterator for SendMmsgIter<'a> {
    type Item = SendMsgResult;

    fn next(&mut self) -> Option<Self::Item> {
        self.hdrs.next().map(|hdr| SendMsgResult { bytes: hdr.msg_len as _ })
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
unsafe impl Send for SendMmsgIter<'_> {}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
unsafe impl Sync for SendMmsgIter<'_> {}

/// Growable container holding the headers for [`recvmmsg`].
///
/// This allocation can be reused when calling [`recvmmsg`] multiple times,
/// which can be beneficial for performance.
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
#[derive(Debug, Default)]
pub struct RecvMmsgHeaders {
    mmsghdrs: Vec<libc::mmsghdr>,
    addresses: Vec<Address>,
    cmsg_len_ptrs: Vec<*mut usize>,
    recv: usize,
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
impl RecvMmsgHeaders {
    /// Creates a new container for the mmsg-headers.
    ///
    /// No allocations are performed.
    pub const fn new() -> Self {
        Self {
            mmsghdrs: Vec::new(),
            addresses: Vec::new(),
            cmsg_len_ptrs: Vec::new(),
            recv: 0,
        }
    }

    /// Creates a new container and reserves space for `cap` mmsg-headers.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            mmsghdrs: Vec::with_capacity(cap),
            addresses: Vec::with_capacity(cap),
            cmsg_len_ptrs: Vec::with_capacity(cap),
            recv: 0,
        }
    }

    /// Returns an iterator over [`RecvMsgResult`], which contains the
    /// metadata of the received messages.
    pub fn iter(&self) -> RecvMmsgIter<'_> {
        RecvMmsgIter {
            hdrs: self.mmsghdrs[..self.recv].iter(),
            addr: self.addresses[..self.recv].iter(),
        }
    }

    fn fill_recv<'a, 'b, I>(&mut self, mut items: I)
    where
        'b: 'a,
        I: Iterator<Item = (&'a mut [IoSliceMut<'b>], CmsgBufHandle<'a>)> + ExactSizeIterator,
    {
        // For panic-safety
        self.recv = 0;

        let len = items.len();

        self.mmsghdrs.clear();
        self.mmsghdrs.reserve(len);

        self.addresses.clear();
        self.addresses.reserve(len);

        self.cmsg_len_ptrs.clear();
        self.cmsg_len_ptrs.reserve(len);

        for _ in 0..len {
            // FIXME: maybe mem-setting the address buffers to zero is faster?
            self.addresses.push(Address::default());
        }

        let mut addresses = self.addresses.iter_mut().map(Address::as_mut_ptr);

        let mut total = 0;

        for (i, (iov, mut cmsg)) in items.by_ref().take(len).enumerate() {
            let mmsg_hdr = libc::mmsghdr {
                msg_hdr: recvmsg_header(addresses.next().unwrap(), iov, &mut cmsg),
                msg_len: 0,
            };

            self.mmsghdrs.push(mmsg_hdr);

            self.cmsg_len_ptrs.push(cmsg.len.map_or(ptr::null_mut(), |l| l as *mut _));

            total = i + 1;
        }

        if total != len || items.next().is_some() {
            panic!("Len returned by exact size iterator was not accurate");
        }
    }

    unsafe fn update_lens(&mut self) {
        for ((mhdr, len), _addr) in self
            .mmsghdrs
            .iter()
            .zip(self.cmsg_len_ptrs.iter_mut())
            .zip(self.addresses.iter_mut())
            .take(self.recv)
        {
            unsafe {
                if let Some(len) = len.as_mut() {
                    *len = mhdr.msg_hdr.msg_controllen as _;
                }
            }

            cfg_if! {
                if #[cfg(any(
                    target_os = "android",
                    target_os = "fuchsia",
                    target_os = "illumos",
                    target_os = "linux",
                    target_os = "redox",
                ))] {
                    _addr.len = mhdr.msg_hdr.msg_namelen as _;
                }
            }
        }
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
unsafe impl Send for RecvMmsgHeaders {}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
unsafe impl Sync for RecvMmsgHeaders {}

/// An iterator returned by [`RecvMmsgHeaders::iter`].
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
#[derive(Debug, Clone)]
pub struct RecvMmsgIter<'a> {
    addr: std::slice::Iter<'a, Address>,
    hdrs: std::slice::Iter<'a, libc::mmsghdr>,
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
impl Iterator for RecvMmsgIter<'_> {
    type Item = RecvMsgResult;

    fn next(&mut self) -> Option<Self::Item> {
        let hdr = *self.hdrs.next()?;
        let bytes = hdr.msg_len as _;
        let addr_buf = *self.addr.next().unwrap();

        Some(RecvMsgResult { bytes, hdr: hdr.msg_hdr, addr_buf })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.hdrs.size_hint()
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
unsafe impl Send for RecvMmsgIter<'_> {}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
unsafe impl Sync for RecvMmsgIter<'_> {}

/// An extension of [`sendmsg`] that allows the caller to transmit multiple messages on a socket
/// using a single system call.
///
/// This has performance benefits for some applications.
///
/// Returns an iterator producing [`SendMsgResult`], one per sent message.
///
/// # Panics
///
/// This function panics if:
///
/// - The length of the [`ExactSizeIterator`] is not accurate.
/// - The number of messages exceeds `u32::MAX` (not applicable for FreeBSD).
///
/// # Bugs (in underlying implementation, at least in Linux)
///
/// If an error occurs after at least one message has been sent, the
/// call succeeds, and returns the number of messages sent.  The
/// error code is lost.  The caller can retry the transmission,
/// starting at the first failed message, but there is no guarantee
/// that, if an error is returned, it will be the same as the one
/// that was lost on the previous call.
///
/// # Examples
///
/// See [`recvmmsg`] for an example using both functions.
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
pub fn sendmmsg<'a, J, A, I>(
    fd: RawFd,
    headers: &mut SendMmsgHeaders,
    items: J,
    flags: MsgFlags,
) -> crate::Result<usize>
where
    J: IntoIterator<Item = (&'a A, &'a I, &'a CmsgStr)>,
    J::IntoIter: ExactSizeIterator,
    A: AsRef<Addr> + ?Sized + 'a,
    I: AsRef<[IoSlice<'a>]> + ?Sized + 'a,
{
    headers.fill_send(items.into_iter().map(|(addr, iov, cmsg)| (addr.as_ref(), iov.as_ref(), cmsg)));

    #[cfg(not(target_os = "freebsd"))]
    let mmsghdrs_len = headers.mmsghdrs.len().try_into().unwrap();

    #[cfg(target_os = "freebsd")]
    let mmsghdrs_len = headers.mmsghdrs.len() as _;

    let sent = Errno::result(unsafe {
        libc::sendmmsg(
            fd,
            headers.mmsghdrs.as_mut_ptr(),
            mmsghdrs_len,
            flags.bits() as _,
        )
    })? as usize;

    headers.sent = sent;

    Ok(sent)
}

/// An extension of [`recvmsg`] that allows the caller to receive multiple messages from a socket
/// using a single system call.
///
/// This has performance benefits for some applications. A further extension over [`recvmsg`] is
/// support for a timeout on the receive operation.
///
/// Returns the number of messages received. The metadata, such as the sender address and length
/// of the messages, can be accessed by calling [`RecvMmsgHeaders::iter`].
///
/// # Panics
///
/// This function panics if:
///
/// - The length of the [`ExactSizeIterator`] is not accurate.
/// - The number of messages exceeds `u32::MAX` (not applicable for FreeBSD).
///
/// # Bugs (in underlying implementation, at least in Linux)
///
/// The timeout argument does not work as intended.  The timeout is
/// checked only after the receipt of each datagram, so that if not all
/// datagrams are received before the timeout expires, but
/// then no further datagrams are received, the call will block
/// forever.
///
/// If an error occurs after at least one message has been received,
/// the call succeeds, and returns the number of messages received.
/// The error code is expected to be returned on a subsequent call to
/// [`recvmmsg`].  In the current implementation, however, the error
/// code can be overwritten in the meantime by an unrelated network
/// event on a socket, for example an incoming ICMP packet.
///
/// # Examples
///
/// ```
/// # use nix::sys::socket::*;
/// # use std::os::fd::AsRawFd;
/// # use std::io::{IoSlice, IoSliceMut};
/// // We use connectionless UDP sockets.
/// let send = socket(
///     AddressFamily::INET,
///     SockType::Datagram,
///     SockFlag::empty(),
///     Some(SockProtocol::Udp),
/// )?;
///
/// let recv = socket(
///     AddressFamily::INET,
///     SockType::Datagram,
///     SockFlag::empty(),
///     Some(SockProtocol::Udp),
/// )?;
///
/// // This is the address we are going to send the message.
/// let addr = "127.0.0.1:6069".parse::<Ipv4Address>().unwrap();
///
/// bind(recv.as_raw_fd(), &addr)?;
///
/// // The two messages we are trying to send: [0, 1, 2, ...] and [0, 2, 4, ...].
/// let msg_1: [u8; 1500] = std::array::from_fn(|i| i as u8);
/// let send_iov_1 = [IoSlice::new(&msg_1)];
///
/// let msg_2: [u8; 1500] = std::array::from_fn(|i| (i * 2) as u8);
/// let send_iov_2 = [IoSlice::new(&msg_2)];
///
/// // We preallocate headers for 2 messages.
/// let mut send_headers = SendMmsgHeaders::with_capacity(2);
///
/// // Zip everything together.
/// //
/// // On connectionless sockets like UDP, destination addresses are required.
/// // Each message can be sent to a different address.
/// let send_items = [
///     (&addr, &send_iov_1, CmsgStr::empty()),
///     (&addr, &send_iov_2, CmsgStr::empty()),
/// ];
///
/// // Send the messages on the send socket.
/// let mut sent = sendmmsg(
///     send.as_raw_fd(),
///     &mut send_headers,
///     send_items,
///     MsgFlags::empty(),
/// )?;
///
/// // We have actually sent 2 messages.
/// assert_eq!(sent, 2);
///
/// // We have actually sent 1500 bytes per message.
/// assert!(send_headers.iter().all(|res| res.bytes() == 1500));
///
/// // Initialize buffers to receive the messages.
/// let mut buf_1 = [0u8; 1500];
/// let mut recv_iov_1 = [IoSliceMut::new(&mut buf_1)];
///
/// let mut buf_2 = [0u8; 1500];
/// let mut recv_iov_2 = [IoSliceMut::new(&mut buf_2)];
///
/// // We preallocate headers for 2 messages.
/// let mut recv_headers = RecvMmsgHeaders::new();
///
/// // Zip everything together.
/// let mut recv_items = [
///     (&mut recv_iov_1, Default::default()),
///     (&mut recv_iov_2, Default::default()),
/// ];
///
/// // Receive `msg` on `recv`.
/// let mut recv = recvmmsg(
///     recv.as_raw_fd(),
///     &mut recv_headers,
///     recv_items,
///     MsgFlags::empty(),
///     None,
/// )?;
///
/// // We have actually received two messages.
/// assert_eq!(recv, 2);
///
/// // We have actually received 1500 bytes per message.
/// // Since this is a connectionless socket, the sender address is returned as well.
/// assert!(recv_headers.iter().all(|res| {
///     res.bytes() == 1500 && res.address().family() == AddressFamily::INET
/// }));
///
/// // The received messages are identical to the sent ones.
/// assert_eq!(buf_1, msg_1);
/// assert_eq!(buf_2, msg_2);
///
/// # Ok::<(), nix::Error>(())
/// ```
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
pub fn recvmmsg<'a, 'b, J, I>(
    fd: RawFd,
    headers: &mut RecvMmsgHeaders,
    items: J,
    flags: MsgFlags,
    mut timeout: Option<crate::sys::time::TimeSpec>,
) -> crate::Result<usize>
where
    J: IntoIterator<Item = (&'a mut I, CmsgBufHandle<'a>)>,
    J::IntoIter: ExactSizeIterator,
    I: AsMut<[IoSliceMut<'b>]> + ?Sized + 'a,
{
    headers.fill_recv(items.into_iter().map(|(iov, cmsg)| (iov.as_mut(), cmsg)));

    let timeout_ptr = timeout
        .as_mut()
        .map_or_else(std::ptr::null_mut, |t| t as *mut _ as *mut libc::timespec);

    #[cfg(not(target_os = "freebsd"))]
    let mmsghdrs_len = headers.mmsghdrs.len().try_into().unwrap();

    #[cfg(target_os = "freebsd")]
    let mmsghdrs_len = headers.mmsghdrs.len() as _;

    let recv = Errno::result(unsafe {
        libc::recvmmsg(
            fd,
            headers.mmsghdrs.as_mut_ptr(),
            mmsghdrs_len,
            flags.bits() as _,
            timeout_ptr,
        )
    })? as usize;

    headers.recv = recv;

    unsafe {
        headers.update_lens();
    }

    Ok(recv)
}

/// Contains the metadata for the sent message.
#[derive(Debug, Clone, Copy)]
pub struct SendMsgResult {
    bytes: usize,
}

impl SendMsgResult {
    /// Returns the number of bytes sent.
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

unsafe impl Send for SendMsgResult {}

unsafe impl Sync for SendMsgResult {}

/// Contains the metadata for the received message.
#[derive(Debug, Clone, Copy)]
pub struct RecvMsgResult {
    bytes: usize,
    hdr: libc::msghdr,
    addr_buf: Address,
}

impl RecvMsgResult {
    /// Returns the number of bytes received.
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    /// Returns the address of the sender.
    ///
    /// **Note**: This method should only be called if the socket is connectionless.
    /// When using connection-mode sockets (like TCP), the address of the peer is already known,
    /// thus the kernel will not return it.
    pub fn address(&self) -> Address {
        self.addr_buf
    }

    /// Returns the received flags of the message from the kernel.
    pub fn flags(&self) -> MsgFlags {
        MsgFlags::from_bits_truncate(self.hdr.msg_flags as _)
    }
}

unsafe impl Send for RecvMsgResult {}

unsafe impl Sync for RecvMsgResult {}

// test contains both recvmmsg and timestaping which is linux only
// there are existing tests for recvmmsg only in tests/
#[cfg(target_os = "linux")]
#[cfg(test)]
mod test {
    use crate::*;
    use std::str::FromStr;

    #[cfg_attr(qemu, ignore)]
    #[test]
    fn test_recvmm_2() -> crate::Result<()> {
        use crate::sys::socket::sockopt::Timestamping;
        use crate::sys::socket::*;
        use std::io::{IoSlice, IoSliceMut};

        let sock_addr = Ipv4Address::from_str("127.0.0.1:6791").unwrap();

        let ssock = socket(
            AddressFamily::INET,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        )?;

        let rsock = socket(
            AddressFamily::INET,
            SockType::Datagram,
            SockFlag::SOCK_NONBLOCK,
            None,
        )?;

        crate::sys::socket::bind(rsock.as_raw_fd(), sock_addr)?;

        setsockopt(&rsock, Timestamping, &TimestampingFlag::all())?;

        let sbuf = (0..400).map(|i| i as u8).collect::<Vec<_>>();

        let mut recv_buf = vec![0; 1024];

        let mut recv_iovs = Vec::new();
        let mut pkt_iovs = Vec::new();

        for (ix, chunk) in recv_buf.chunks_mut(256).enumerate() {
            pkt_iovs.push(IoSliceMut::new(chunk));
            if ix % 2 == 1 {
                recv_iovs.push(pkt_iovs);
                pkt_iovs = Vec::new();
            }
        }
        drop(pkt_iovs);

        let flags = MsgFlags::empty();
        let iov1 = [IoSlice::new(&sbuf)];

        sendmsg(ssock.as_raw_fd(), sock_addr, &iov1, Default::default(), flags).unwrap();

        let mut headers = super::RecvMmsgHeaders::with_capacity(recv_iovs.len());

        let mut cmsgs = Vec::with_capacity(recv_iovs.len());

        for _ in 0..recv_iovs.len() {
            cmsgs.push(cmsg_vec_internal![ScmTimestampsns]);
        }

        let t = sys::time::TimeSpec::from_duration(std::time::Duration::from_secs(10));

        let items = recv_iovs.iter_mut().zip(cmsgs.iter_mut().map(CmsgBuf::handle));

        let _ = super::recvmmsg(rsock.as_raw_fd(), &mut headers, items, flags, Some(t))?;

        for (i, rmsg) in headers.iter().enumerate() {
            #[cfg(not(any(qemu, target_arch = "aarch64")))]
            let mut saw_time = false;

            for cmsg in cmsgs[i].iter() {
                if let ControlMessageOwned::ScmTimestampsns(timestamps) = cmsg {
                    let ts = timestamps.system;

                    let sys_time =
                        crate::time::clock_gettime(crate::time::ClockId::CLOCK_REALTIME)?;
                    let diff = if ts > sys_time {
                        ts - sys_time
                    } else {
                        sys_time - ts
                    };
                    assert!(std::time::Duration::from(diff).as_secs() < 60);
                    #[cfg(not(any(qemu, target_arch = "aarch64")))]
                    {
                        saw_time = true;
                    }
                }
            }

            #[cfg(not(any(qemu, target_arch = "aarch64")))]
            assert!(saw_time);

            assert_eq!(rmsg.bytes(), 400);
        }

        Ok(())
    }
}
}

/// Create an endpoint for communication
///
/// The `protocol` specifies a particular protocol to be used with the
/// socket.  Normally only a single protocol exists to support a
/// particular socket type within a given protocol family, in which case
/// protocol can be specified as `None`.  However, it is possible that many
/// protocols may exist, in which case a particular protocol must be
/// specified in this manner.
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/socket.html)
pub fn socket<T: Into<Option<SockProtocol>>>(
    domain: AddressFamily,
    ty: SockType,
    flags: SockFlag,
    protocol: T,
) -> Result<OwnedFd> {
    let protocol = match protocol.into() {
        None => 0,
        Some(p) => p as c_int,
    };

    // SockFlags are usually embedded into `ty`, but we don't do that in `nix` because it's a
    // little easier to understand by separating it out. So we have to merge these bitfields
    // here.
    let mut ty = ty as c_int;
    ty |= flags.bits();

    let res = unsafe { libc::socket(domain.family(), ty, protocol) };

    match res {
        -1 => Err(Errno::last()),
        fd => {
            // Safe because libc::socket returned success
            unsafe { Ok(OwnedFd::from_raw_fd(fd)) }
        }
    }
}

/// Create a pair of connected sockets
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/socketpair.html)
pub fn socketpair<T: Into<Option<SockProtocol>>>(
    domain: AddressFamily,
    ty: SockType,
    protocol: T,
    flags: SockFlag,
) -> Result<(OwnedFd, OwnedFd)> {
    let protocol = match protocol.into() {
        None => 0,
        Some(p) => p as c_int,
    };

    // SockFlags are usually embedded into `ty`, but we don't do that in `nix` because it's a
    // little easier to understand by separating it out. So we have to merge these bitfields
    // here.
    let mut ty = ty as c_int;
    ty |= flags.bits();

    let mut fds = [-1, -1];

    let res = unsafe {
        libc::socketpair(domain.family(), ty, protocol, fds.as_mut_ptr())
    };
    Errno::result(res)?;

    // Safe because socketpair returned success.
    unsafe { Ok((OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1]))) }
}

/// Listen for connections on a socket
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/listen.html)
pub fn listen<F: AsFd>(sock: &F, backlog: usize) -> Result<()> {
    let fd = sock.as_fd().as_raw_fd();
    let res = unsafe { libc::listen(fd, backlog as c_int) };

    Errno::result(res).map(drop)
}

/// Bind a name to a socket
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/bind.html)
pub fn bind<A>(fd: RawFd, addr: A) -> Result<()>
where
    A: AsRef<Addr>,
{
    let res = unsafe {
        libc::bind(fd, addr.as_ref().as_ptr().cast(), addr.as_ref().len() as _)
    };

    Errno::result(res).map(drop)
}

/// Accept a connection on a socket
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/accept.html)
pub fn accept(sockfd: RawFd) -> Result<RawFd> {
    let res = unsafe { libc::accept(sockfd, ptr::null_mut(), ptr::null_mut()) };

    Errno::result(res)
}

/// Accept a connection on a socket
///
/// [Further reading](https://man7.org/linux/man-pages/man2/accept.2.html)
#[cfg(any(
    all(
        target_os = "android",
        any(
            target_arch = "aarch64",
            target_arch = "x86",
            target_arch = "x86_64"
        )
    ),
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub fn accept4(sockfd: RawFd, flags: SockFlag) -> Result<RawFd> {
    let res = unsafe {
        libc::accept4(sockfd, ptr::null_mut(), ptr::null_mut(), flags.bits())
    };

    Errno::result(res)
}

/// Initiate a connection on a socket
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/connect.html)
pub fn connect<A>(fd: RawFd, addr: A) -> Result<()>
where
    A: AsRef<Addr>,
{
    let res = unsafe {
        libc::connect(
            fd,
            addr.as_ref().as_ptr().cast(),
            addr.as_ref().len() as _,
        )
    };

    Errno::result(res).map(drop)
}

/// Receive data from a connection-oriented socket. Returns the number of
/// bytes read
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/recv.html)
pub fn recv(sockfd: RawFd, buf: &mut [u8], flags: MsgFlags) -> Result<usize> {
    unsafe {
        let ret = libc::recv(
            sockfd,
            buf.as_mut_ptr().cast(),
            buf.len() as size_t,
            flags.bits(),
        );

        Errno::result(ret).map(|r| r as usize)
    }
}

/// Receive data from a connectionless or connection-oriented socket. Returns
/// the number of bytes read and, for connectionless sockets,  the socket
/// address of the sender.
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/recvfrom.html)
pub fn recvfrom(sockfd: RawFd, buf: &mut [u8]) -> Result<(usize, Address)> {
    unsafe {
        let mut addr = Address::default();
        let mut len = mem::size_of::<libc::sockaddr_storage>() as socklen_t;

        let ret = Errno::result(libc::recvfrom(
            sockfd,
            buf.as_mut_ptr().cast(),
            buf.len() as size_t,
            0,
            addr.as_mut_ptr().cast(),
            &mut len as *mut socklen_t,
        ))? as usize;

        cfg_if! {
            if #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "redox",
            ))] {
                addr.len = len as _;
            }
        }

        Ok((ret, addr))
    }
}

/// Send a message to a socket
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html)
pub fn sendto<A>(
    fd: RawFd,
    buf: &[u8],
    addr: A,
    flags: MsgFlags,
) -> Result<usize>
where
    A: AsRef<Addr>,
{
    let ret = unsafe {
        libc::sendto(
            fd,
            buf.as_ptr().cast(),
            buf.len() as size_t,
            flags.bits(),
            addr.as_ref().as_ptr().cast(),
            addr.as_ref().len() as _,
        )
    };

    Errno::result(ret).map(|r| r as usize)
}

/// Send data to a connection-oriented socket. Returns the number of bytes read
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/send.html)
pub fn send(fd: RawFd, buf: &[u8], flags: MsgFlags) -> Result<usize> {
    let ret = unsafe {
        libc::send(fd, buf.as_ptr().cast(), buf.len() as size_t, flags.bits())
    };

    Errno::result(ret).map(|r| r as usize)
}

/*
 *
 * ===== Socket Options =====
 *
 */

/// Represents a socket option that can be retrieved.
pub trait GetSockOpt: Copy {
    type Val;

    /// Look up the value of this socket option on the given socket.
    fn get<F: AsFd>(&self, fd: &F) -> Result<Self::Val>;
}

/// Represents a socket option that can be set.
pub trait SetSockOpt: Clone {
    type Val;

    /// Set the value of this socket option on the given socket.
    fn set<F: AsFd>(&self, fd: &F, val: &Self::Val) -> Result<()>;
}

/// Get the current value for the requested socket option
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html)
pub fn getsockopt<F: AsFd, O: GetSockOpt>(fd: &F, opt: O) -> Result<O::Val> {
    opt.get(fd)
}

/// Sets the value for the requested socket option
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html)
///
/// # Examples
///
/// ```
/// use nix::sys::socket::setsockopt;
/// use nix::sys::socket::sockopt::KeepAlive;
/// use std::net::TcpListener;
///
/// let listener = TcpListener::bind("0.0.0.0:0").unwrap();
/// let fd = listener;
/// let res = setsockopt(&fd, KeepAlive, &true);
/// assert!(res.is_ok());
/// ```
pub fn setsockopt<F: AsFd, O: SetSockOpt>(
    fd: &F,
    opt: O,
    val: &O::Val,
) -> Result<()> {
    opt.set(fd, val)
}

/// Get the address of the peer connected to the socket `fd`.
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/getpeername.html)
pub fn getpeername(fd: RawFd) -> Result<Address> {
    unsafe {
        let mut addr = Address::default();
        let mut len = mem::size_of::<libc::sockaddr_storage>() as _;

        let ret = libc::getpeername(fd, addr.as_mut_ptr().cast(), &mut len);

        Errno::result(ret)?;

        cfg_if! {
            if #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "redox",
            ))] {
                addr.len = len as _;
            }
        }

        Ok(addr)
    }
}

/// Get the current address to which the socket `fd` is bound.
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockname.html)
pub fn getsockname(fd: RawFd) -> Result<Address> {
    unsafe {
        let mut addr = Address::default();

        let mut len = mem::size_of::<libc::sockaddr_storage>() as _;

        let ret = libc::getsockname(fd, addr.as_mut_ptr().cast(), &mut len);

        Errno::result(ret)?;

        cfg_if! {
            if #[cfg(any(
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "redox",
            ))] {
                addr.len = len as _;
            }
        }

        Ok(addr)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Shutdown {
    /// Further receptions will be disallowed.
    Read,
    /// Further  transmissions will be disallowed.
    Write,
    /// Further receptions and transmissions will be disallowed.
    Both,
}

/// Shut down part of a full-duplex connection.
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/shutdown.html)
pub fn shutdown(df: RawFd, how: Shutdown) -> Result<()> {
    unsafe {
        use libc::shutdown;

        let how = match how {
            Shutdown::Read => libc::SHUT_RD,
            Shutdown::Write => libc::SHUT_WR,
            Shutdown::Both => libc::SHUT_RDWR,
        };

        Errno::result(shutdown(df, how)).map(drop)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_os = "redox"))]
    #[test]
    fn can_use_cmsg_space() {
        let _ = cmsg_space!(ScmTimestamp);
    }

    #[cfg(not(any(
        target_os = "redox",
        target_os = "linux",
        target_os = "android"
    )))]
    #[test]
    fn can_open_routing_socket() {
        let _ = super::socket(
            super::AddressFamily::ROUTE,
            super::SockType::Raw,
            super::SockFlag::empty(),
            None,
        )
        .expect("Failed to open routing socket");
    }
}
