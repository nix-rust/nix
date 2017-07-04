pub use self::os::*;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod os {
    use libc::{self, c_int, uint8_t};

    pub const AF_UNIX: c_int  = libc::AF_UNIX;
    pub const AF_LOCAL: c_int = libc::AF_LOCAL;
    pub const AF_INET: c_int  = libc::AF_INET;
    pub const AF_INET6: c_int = libc::AF_INET6;
    pub const AF_NETLINK: c_int = libc::AF_NETLINK;
    pub const AF_PACKET: c_int = libc::AF_PACKET;

    pub const SOCK_STREAM: c_int = libc::SOCK_STREAM;
    pub const SOCK_DGRAM: c_int = libc::SOCK_DGRAM;
    pub const SOCK_SEQPACKET: c_int = libc::SOCK_SEQPACKET;
    pub const SOCK_RAW: c_int = libc::SOCK_RAW;
    pub const SOCK_RDM: c_int = 4;

    pub const SOL_IP: c_int = libc::SOL_IP;
    pub const SOL_SOCKET: c_int = libc::SOL_SOCKET;
    pub const SOL_TCP: c_int = libc::SOL_TCP;
    pub const SOL_UDP: c_int = 17;
    pub const SOL_IPV6: c_int = libc::SOL_IPV6;
    pub const SOL_NETLINK: c_int = libc::SOL_NETLINK;
    pub const IPPROTO_IP: c_int = libc::IPPROTO_IP;
    pub const IPPROTO_IPV6: c_int = libc::IPPROTO_IPV6;
    pub const IPPROTO_TCP: c_int = libc::IPPROTO_TCP;
    pub const IPPROTO_UDP: c_int = SOL_UDP;

    pub const SO_ACCEPTCONN: c_int = libc::SO_ACCEPTCONN;
    pub const SO_BINDTODEVICE: c_int = libc::SO_BINDTODEVICE;
    pub const SO_BROADCAST: c_int = libc::SO_BROADCAST;
    pub const SO_BSDCOMPAT: c_int = libc::SO_BSDCOMPAT;
    pub const SO_DEBUG: c_int = libc::SO_DEBUG;
    pub const SO_DOMAIN: c_int = libc::SO_DOMAIN;
    pub const SO_ERROR: c_int = libc::SO_ERROR;
    pub const SO_DONTROUTE: c_int = libc::SO_DONTROUTE;
    pub const SO_KEEPALIVE: c_int = libc::SO_KEEPALIVE;
    pub const SO_LINGER: c_int = libc::SO_LINGER;
    pub const SO_MARK: c_int = libc::SO_MARK;
    pub const SO_OOBINLINE: c_int = libc::SO_OOBINLINE;
    pub const SO_PASSCRED: c_int = libc::SO_PASSCRED;
    pub const SO_PEEK_OFF: c_int = libc::SO_PEEK_OFF;
    pub const SO_PEERCRED: c_int = libc::SO_PEERCRED;
    pub const SO_PRIORITY: c_int = libc::SO_PRIORITY;
    pub const SO_PROTOCOL: c_int = libc::SO_PROTOCOL;
    pub const SO_RCVBUF: c_int = libc::SO_RCVBUF;
    pub const SO_RCVBUFFORCE: c_int = 33;
    pub const SO_RCVLOWAT: c_int = libc::SO_RCVLOWAT;
    pub const SO_SNDLOWAT: c_int = libc::SO_SNDLOWAT;
    pub const SO_RCVTIMEO: c_int = libc::SO_RCVTIMEO;
    pub const SO_SNDTIMEO: c_int = libc::SO_SNDTIMEO;
    pub const SO_REUSEADDR: c_int = libc::SO_REUSEADDR;
    pub const SO_REUSEPORT: c_int = libc::SO_REUSEPORT;
    pub const SO_RXQ_OVFL: c_int = libc::SO_RXQ_OVFL;
    pub const SO_SNDBUF: c_int = libc::SO_SNDBUF;
    pub const SO_SNDBUFFORCE: c_int = libc::SO_SNDBUFFORCE;
    pub const SO_TIMESTAMP: c_int = libc::SO_TIMESTAMP;
    pub const SO_TYPE: c_int = libc::SO_TYPE;
    pub const SO_BUSY_POLL: c_int = libc::SO_BUSY_POLL;
    #[cfg(target_os = "linux")]
    pub const SO_ORIGINAL_DST: c_int = 80;

    // Socket options for TCP sockets
    pub const TCP_NODELAY: c_int = libc::TCP_NODELAY;
    pub const TCP_MAXSEG: c_int = libc::TCP_MAXSEG;
    pub const TCP_CORK: c_int = libc::TCP_CORK;
    pub const TCP_KEEPIDLE: c_int = libc::TCP_KEEPIDLE;

    // Socket options for the IP layer of the socket
    pub const IP_MULTICAST_IF: c_int = 32;

    pub type IpMulticastTtl = uint8_t;

    pub const IP_MULTICAST_TTL: c_int = libc::IP_MULTICAST_TTL;
    pub const IP_MULTICAST_LOOP: c_int = libc::IP_MULTICAST_LOOP;
    pub const IP_ADD_MEMBERSHIP: c_int = libc::IP_ADD_MEMBERSHIP;
    pub const IP_DROP_MEMBERSHIP: c_int = libc::IP_DROP_MEMBERSHIP;

    pub const IPV6_ADD_MEMBERSHIP: c_int = libc::IPV6_ADD_MEMBERSHIP;
    pub const IPV6_DROP_MEMBERSHIP: c_int = libc::IPV6_DROP_MEMBERSHIP;

    pub type InAddrT = u32;

    // Declarations of special addresses
    pub const INADDR_ANY: InAddrT = 0;
    pub const INADDR_NONE: InAddrT = 0xffffffff;
    pub const INADDR_BROADCAST: InAddrT = 0xffffffff;

    // Flags for send/recv and their relatives
    libc_bitflags!{
        pub flags MsgFlags: libc::c_int {
            MSG_OOB,
            MSG_PEEK,
            MSG_CTRUNC,
            MSG_TRUNC,
            MSG_DONTWAIT,
            MSG_EOR,
            MSG_ERRQUEUE,
            MSG_CMSG_CLOEXEC,
        }
    }

    // shutdown flags
    pub const SHUT_RD: c_int = libc::SHUT_RD;
    pub const SHUT_WR: c_int = libc::SHUT_WR;
    pub const SHUT_RDWR: c_int = libc::SHUT_RDWR;

    // Ancillary message types
    pub const SCM_RIGHTS: c_int = libc::SCM_RIGHTS;
}

// Not all of these constants exist on freebsd
#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
mod os {
    #[cfg(any(target_os = "macos",
              target_os = "ios",
              target_os = "freebsd"))]
    use libc::{self, c_int, uint8_t};
    #[cfg(any(target_os = "openbsd", target_os = "netbsd"))]
    use libc::{self, c_int, uint8_t};

    pub const AF_UNIX: c_int  = libc::AF_UNIX;
    pub const AF_LOCAL: c_int = libc::AF_LOCAL;
    pub const AF_INET: c_int  = libc::AF_INET;
    pub const AF_INET6: c_int = libc::AF_INET6;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub const AF_SYSTEM: c_int = libc::AF_SYSTEM;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub const AF_SYS_CONTROL: c_int = 2;

    pub const SOCK_STREAM: c_int = libc::SOCK_STREAM;
    pub const SOCK_DGRAM: c_int = libc::SOCK_DGRAM;
    pub const SOCK_SEQPACKET: c_int = libc::SOCK_SEQPACKET;
    pub const SOCK_RAW: c_int = libc::SOCK_RAW;
    pub const SOCK_RDM: c_int = libc::SOCK_RDM;

    pub const SOL_SOCKET: c_int = libc::SOL_SOCKET;
    pub const IPPROTO_IP: c_int = libc::IPPROTO_IP;
    pub const IPPROTO_IPV6: c_int = libc::IPPROTO_IPV6;
    pub const IPPROTO_TCP: c_int = libc::IPPROTO_TCP;
    pub const IPPROTO_UDP: c_int = 17;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub const SYSPROTO_CONTROL: c_int = 2;

    pub const SO_ACCEPTCONN: c_int          = libc::SO_ACCEPTCONN;
    pub const SO_BROADCAST: c_int           = libc::SO_BROADCAST;
    pub const SO_DEBUG: c_int               = libc::SO_DEBUG;
    #[cfg(not(target_os = "netbsd"))]
    pub const SO_DONTTRUNC: c_int           = 0x2000;
    pub const SO_USELOOPBACK: c_int         = libc::SO_USELOOPBACK;
    pub const SO_ERROR: c_int               = libc::SO_ERROR;
    pub const SO_DONTROUTE: c_int           = libc::SO_DONTROUTE;
    pub const SO_KEEPALIVE: c_int           = libc::SO_KEEPALIVE;
    pub const SO_LABEL: c_int               = 0x1010;
    pub const SO_LINGER: c_int              = libc::SO_LINGER;
    pub const SO_NREAD: c_int               = 0x1020;
    pub const SO_NKE: c_int                 = 0x1021;
    pub const SO_NOSIGPIPE: c_int           = 0x1022;
    pub const SO_NOADDRERR: c_int           = 0x1023;
    pub const SO_NOTIFYCONFLICT: c_int      = 0x1026;
    pub const SO_NP_EXTENSIONS: c_int       = 0x1083;
    pub const SO_NWRITE: c_int              = 0x1024;
    pub const SO_OOBINLINE: c_int           = libc::SO_OOBINLINE;
    pub const SO_PEERLABEL: c_int           = 0x1011;
    pub const SO_RCVBUF: c_int              = libc::SO_RCVBUF;
    pub const SO_RCVLOWAT: c_int            = libc::SO_RCVLOWAT;
    pub const SO_SNDLOWAT: c_int            = libc::SO_SNDLOWAT;
    pub const SO_RCVTIMEO: c_int            = libc::SO_RCVTIMEO;
    pub const SO_SNDTIMEO: c_int            = libc::SO_SNDTIMEO;
    pub const SO_RANDOMPORT: c_int          = 0x1082;
    pub const SO_RESTRICTIONS: c_int        = 0x1081;
    pub const SO_RESTRICT_DENYIN: c_int     = 0x00000001;
    pub const SO_RESTRICT_DENYOUT: c_int    = 0x00000002;
    pub const SO_REUSEADDR: c_int           = libc::SO_REUSEADDR;
    pub const SO_REUSEPORT: c_int           = libc::SO_REUSEPORT;
    pub const SO_REUSESHAREUID: c_int       = 0x1025;
    pub const SO_SNDBUF: c_int              = libc::SO_SNDBUF;
    pub const SO_TIMESTAMP: c_int           = libc::SO_TIMESTAMP;
    #[cfg(not(target_os = "netbsd"))]
    pub const SO_TIMESTAMP_MONOTONIC: c_int = 0x0800;
    pub const SO_TYPE: c_int                = libc::SO_TYPE;
    #[cfg(not(target_os = "netbsd"))]
    pub const SO_WANTMORE: c_int            = 0x4000;
    pub const SO_WANTOOBFLAG: c_int         = 0x8000;
    #[allow(overflowing_literals)]
    pub const SO_RESTRICT_DENYSET: c_int    = 0x80000000;

    // Socket options for TCP sockets
    pub const TCP_NODELAY: c_int = libc::TCP_NODELAY;
    pub const TCP_MAXSEG: c_int = 2;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub const TCP_KEEPALIVE: c_int = libc::TCP_KEEPALIVE;
    #[cfg(target_os = "freebsd")]
    pub const TCP_KEEPIDLE: c_int = libc::TCP_KEEPIDLE;

    // Socket options for the IP layer of the socket
    pub const IP_MULTICAST_IF: c_int = 9;

    pub type IpMulticastTtl = uint8_t;

    pub const IP_MULTICAST_TTL: c_int = libc::IP_MULTICAST_TTL;
    pub const IP_MULTICAST_LOOP: c_int = libc::IP_MULTICAST_LOOP;
    pub const IP_ADD_MEMBERSHIP: c_int = libc::IP_ADD_MEMBERSHIP;
    pub const IP_DROP_MEMBERSHIP: c_int = libc::IP_DROP_MEMBERSHIP;

    pub const IPV6_JOIN_GROUP: c_int = libc::IPV6_JOIN_GROUP;
    pub const IPV6_LEAVE_GROUP: c_int = libc::IPV6_LEAVE_GROUP;

    pub type InAddrT = u32;

    // Declarations of special addresses
    pub const INADDR_ANY: InAddrT = 0;
    pub const INADDR_NONE: InAddrT = 0xffffffff;
    pub const INADDR_BROADCAST: InAddrT = 0xffffffff;

    // Flags for send/recv and their relatives
    libc_bitflags!{
        pub flags MsgFlags: libc::c_int {
            MSG_OOB,
            MSG_PEEK,
            MSG_EOR,
            MSG_TRUNC,
            MSG_CTRUNC,
            MSG_DONTWAIT,
        }
    }

    // shutdown flags
    pub const SHUT_RD: c_int = libc::SHUT_RD;
    pub const SHUT_WR: c_int = libc::SHUT_WR;
    pub const SHUT_RDWR: c_int = libc::SHUT_RDWR;

    // Ancillary message types
    pub const SCM_RIGHTS: c_int = 1;
}

#[cfg(target_os = "dragonfly")]
mod os {
    use libc::{c_int, uint8_t};

    pub const AF_UNIX: c_int  = libc::AF_UNIX;
    pub const AF_LOCAL: c_int = libc::AF_LOCAL;
    pub const AF_INET: c_int  = libc::AF_INET;
    pub const AF_INET6: c_int = libc::AF_INET6;

    pub const SOCK_STREAM: c_int = libc::SOCK_STREAM;
    pub const SOCK_DGRAM: c_int = libc::SOCK_DGRAM;
    pub const SOCK_SEQPACKET: c_int = libc::SOCK_SEQPACKET;
    pub const SOCK_RAW: c_int = libc::SOCK_RAW;
    pub const SOCK_RDM: c_int = libc::SOCK_RDM;

    pub const SOL_SOCKET: c_int = libc::SOL_SOCKET;
    pub const IPPROTO_IP: c_int = libc::IPPROTO_IP;
    pub const IPPROTO_IPV6: c_int = libc::IPPROTO_IPV6;
    pub const IPPROTO_TCP: c_int = libc::IPPROTO_TCP;
    pub const IPPROTO_UDP: c_int = libc::IPPROTO_UDP;

    pub const SO_ACCEPTCONN: c_int = libc::SO_ACCEPTCONN;
    pub const SO_BROADCAST: c_int = libc::SO_BROADCAST;
    pub const SO_DEBUG: c_int = libc::SO_DEBUG;
    pub const SO_ERROR: c_int = libc::SO_ERROR;
    pub const SO_DONTROUTE: c_int = libc::SO_DONTROUTE;
    pub const SO_KEEPALIVE: c_int = libc::SO_KEEPALIVE;
    pub const SO_LINGER: c_int = libc::SO_LINGER;
    pub const SO_NOSIGPIPE: c_int = libc::SO_NOSIGPIPE;
    pub const SO_OOBINLINE: c_int = libc::SO_OOBINLINE;
    pub const SO_RCVBUF: c_int = libc::SO_RCVBUF;
    pub const SO_RCVLOWAT: c_int = libc::RCVLOWAT;
    pub const SO_SNDLOWAT: c_int = libc::SO_SNDLOWAT;
    pub const SO_RCVTIMEO: c_int = libc::SO_RCVTIMEO;
    pub const SO_SNDTIMEO: c_int = libc::SO_SNDTIMEO;
    pub const SO_REUSEADDR: c_int = libc::SO_REUSEADDR;
    pub const SO_REUSEPORT: c_int = libc::SO_REUSEPORT;
    pub const SO_SNDBUF: c_int = libc::SO_SNDBUF;
    pub const SO_TIMESTAMP: c_int = libc::SO_TIMESTAMP;
    pub const SO_TYPE: c_int = libc::SO_TYPE;

    // Socket options for TCP sockets
    pub const TCP_NODELAY: c_int = libc::TCP_NODELAY;
    pub const TCP_MAXSEG: c_int = libc::TCP_MAXSEG;
    pub const TCP_KEEPIDLE: c_int = libc::TCP_KEEPIDLE;

    // Socket options for the IP layer of the socket
    pub const IP_MULTICAST_IF: c_int = 9;

    pub type IpMulticastTtl = uint8_t;

    pub const IP_MULTICAST_TTL: c_int = libc::IP_MULTICAST_TTL;
    pub const IP_MULTICAST_LOOP: c_int = libc::IP_MULTICAST_LOOP;
    pub const IP_ADD_MEMBERSHIP: c_int = libc::IP_ADD_MEMBERSHIP;
    pub const IP_DROP_MEMBERSHIP: c_int = libc::IP_DROP_MEMBERSHIP;
    pub const IPV6_JOIN_GROUP: c_int = libc::IPV6_JOIN_GROUP;
    pub const IPV6_LEAVE_GROUP: c_int = libc::IPV6_LEAVE_GROUP;

    pub type InAddrT = u32;

    // Declarations of special addresses
    pub const INADDR_ANY: InAddrT = 0;
    pub const INADDR_NONE: InAddrT = 0xffffffff;
    pub const INADDR_BROADCAST: InAddrT = 0xffffffff;

    // Flags for send/recv and their relatives
    libc_bitflags!{
        pub flags MsgFlags: libc::c_int {
            MSG_OOB,
            MSG_PEEK,
            MSG_DONTWAIT,
        }
    }

    // shutdown flags
    pub const SHUT_RD: c_int = libc::SHUT_RD;
    pub const SHUT_WR: c_int = libc::SHUT_WR;
    pub const SHUT_RDWR: c_int = libc::SHUT_RDWR;
}

#[cfg(test)]
mod test {
    use super::*;
    use nixtest::{assert_const_eq,get_int_const,GetConst};
    use libc::{c_char};
    use std::fmt;

    impl fmt::Display for MsgFlags {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.bits())
        }
    }

    impl GetConst for MsgFlags {
        unsafe fn get_const(name: *const c_char) -> MsgFlags {
            MsgFlags::from_bits_truncate(get_int_const(name))
        }
    }

    macro_rules! check_const {
        ($($konst:ident),+) => {{
            $(assert_const_eq(stringify!($konst), $konst);)+
        }};
    }

    #[test]
    pub fn test_const_values() {
        check_const!(
            AF_UNIX,
            AF_LOCAL,
            AF_INET,
            AF_INET6,
            SOCK_STREAM,
            SOCK_DGRAM,
            SOCK_SEQPACKET,
            SOCK_RAW,
            SOCK_RDM,
            SOL_SOCKET,
            IPPROTO_IP,
            IPPROTO_IPV6,
            IPPROTO_TCP,
            IPPROTO_UDP,
            SO_ACCEPTCONN,
            SO_BROADCAST,
            SO_DEBUG,
            SO_ERROR,
            SO_DONTROUTE,
            SO_KEEPALIVE,
            SO_LINGER,
            SO_OOBINLINE,
            SO_RCVBUF,
            SO_RCVLOWAT,
            SO_SNDLOWAT,
            SO_RCVTIMEO,
            SO_SNDTIMEO,
            SO_REUSEADDR,
            // SO_REUSEPORT,
            SO_SNDBUF,
            SO_TIMESTAMP,
            SO_TYPE,
            TCP_NODELAY,
            TCP_MAXSEG,
            IP_MULTICAST_IF,
            IP_MULTICAST_TTL,
            IP_MULTICAST_LOOP,
            IP_ADD_MEMBERSHIP,
            IP_DROP_MEMBERSHIP,
            INADDR_ANY,
            INADDR_NONE,
            INADDR_BROADCAST,
            MSG_OOB,
            MSG_PEEK,
            MSG_DONTWAIT,
            MSG_EOR,
            MSG_TRUNC,
            MSG_CTRUNC,
            SHUT_RD,
            SHUT_WR,
            SHUT_RDWR
            );


    }

    #[cfg(target_os = "linux")]
    #[test]
    pub fn test_general_linux_consts() {
        // TODO Figure out how to test new constants
        check_const!(
            SOL_IP,
            SOL_TCP,
            SOL_UDP,
            SOL_IPV6,
            SO_BINDTODEVICE,
            SO_BSDCOMPAT,
            // SO_DOMAIN,
            // SO_MARK,
            TCP_CORK,
            // SO_BUSY_POLL,
            // SO_RXQ_OVFL,
            SO_PRIORITY,
            // SO_PROTOCOL,
            SO_RCVBUFFORCE,
            // SO_PEEK_OFF,
            MSG_ERRQUEUE);
    }

    #[cfg(all(target_os = "linux", not(target_arch="arm")))]
    #[test]
    pub fn test_linux_not_arm_consts() {
        // TODO Figure out how to test new constants
        check_const!(
            SO_PASSCRED,
            SO_PEERCRED,
            SO_SNDBUFFORCE);
    }

}
