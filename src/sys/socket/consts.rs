pub use self::os::*;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod os {
    use libc::{self, c_int, uint8_t};

    pub const AF_UNIX: c_int  = 1;
    pub const AF_LOCAL: c_int = AF_UNIX;
    pub const AF_INET: c_int  = 2;
    pub const AF_INET6: c_int = 10;
    pub const AF_NETLINK: c_int = 16;
    pub const AF_PACKET: c_int = 17;

    pub const SOCK_STREAM: c_int = 1;
    pub const SOCK_DGRAM: c_int = 2;
    pub const SOCK_SEQPACKET: c_int = 5;
    pub const SOCK_RAW: c_int = 3;
    pub const SOCK_RDM: c_int = 4;

    pub const SOL_IP: c_int     = 0;
    pub const SOL_SOCKET: c_int = 1;
    pub const SOL_TCP: c_int    = 6;
    pub const SOL_UDP: c_int    = 17;
    pub const SOL_IPV6: c_int   = 41;
    pub const SOL_NETLINK: c_int = 270;
    pub const IPPROTO_IP: c_int = SOL_IP;
    pub const IPPROTO_IPV6: c_int = SOL_IPV6;
    pub const IPPROTO_TCP: c_int = SOL_TCP;
    pub const IPPROTO_UDP: c_int = SOL_UDP;

    pub const SO_ACCEPTCONN: c_int = 30;
    pub const SO_BINDTODEVICE: c_int = 25;
    pub const SO_BROADCAST: c_int = 6;
    pub const SO_BSDCOMPAT: c_int = 14;
    pub const SO_DEBUG: c_int = 1;
    pub const SO_DOMAIN: c_int = 39;
    pub const SO_ERROR: c_int = 4;
    pub const SO_DONTROUTE: c_int = 5;
    pub const SO_KEEPALIVE: c_int = 9;
    pub const SO_LINGER: c_int = 13;
    pub const SO_MARK: c_int = 36;
    pub const SO_OOBINLINE: c_int = 10;
    pub const SO_PASSCRED: c_int = 16;
    pub const SO_PEEK_OFF: c_int = 42;
    pub const SO_PEERCRED: c_int = 17;
    pub const SO_PRIORITY: c_int = 12;
    pub const SO_PROTOCOL: c_int = 38;
    pub const SO_RCVBUF: c_int = 8;
    pub const SO_RCVBUFFORCE: c_int = 33;
    pub const SO_RCVLOWAT: c_int = 18;
    pub const SO_SNDLOWAT: c_int = 19;
    pub const SO_RCVTIMEO: c_int = 20;
    pub const SO_SNDTIMEO: c_int = 21;
    pub const SO_REUSEADDR: c_int = 2;
    pub const SO_REUSEPORT: c_int = 15;
    pub const SO_RXQ_OVFL: c_int = 40;
    pub const SO_SNDBUF: c_int = 7;
    pub const SO_SNDBUFFORCE: c_int = 32;
    pub const SO_TIMESTAMP: c_int = 29;
    pub const SO_TYPE: c_int = 3;
    pub const SO_BUSY_POLL: c_int = 46;

    // Socket options for TCP sockets
    pub const TCP_NODELAY: c_int = 1;
    pub const TCP_MAXSEG: c_int = 2;
    pub const TCP_CORK: c_int = 3;
    pub const TCP_KEEPIDLE: c_int = libc::TCP_KEEPIDLE;

    // Socket options for the IP layer of the socket
    pub const IP_MULTICAST_IF: c_int = 32;

    pub type IpMulticastTtl = uint8_t;

    pub const IP_MULTICAST_TTL: c_int = 33;
    pub const IP_MULTICAST_LOOP: c_int = 34;
    pub const IP_ADD_MEMBERSHIP: c_int = 35;
    pub const IP_DROP_MEMBERSHIP: c_int = 36;

    pub const IPV6_ADD_MEMBERSHIP: c_int = libc::IPV6_ADD_MEMBERSHIP;
    pub const IPV6_DROP_MEMBERSHIP: c_int = libc::IPV6_DROP_MEMBERSHIP;

    pub type InAddrT = u32;

    // Declarations of special addresses
    pub const INADDR_ANY: InAddrT = 0;
    pub const INADDR_NONE: InAddrT = 0xffffffff;
    pub const INADDR_BROADCAST: InAddrT = 0xffffffff;

    // Flags for send/recv and their relatives
    bitflags!{
        flags MsgFlags : libc::c_int {
            const MSG_OOB      = 0x0001,
            const MSG_PEEK     = 0x0002,
            const MSG_CTRUNC   = 0x0008,
            const MSG_TRUNC    = 0x0020,
            const MSG_DONTWAIT = 0x0040,
            const MSG_EOR      = 0x0080,
            const MSG_ERRQUEUE = 0x2000,
        }
    }

    // shutdown flags
    pub const SHUT_RD: c_int   = 0;
    pub const SHUT_WR: c_int   = 1;
    pub const SHUT_RDWR: c_int = 2;

    // Ancillary message types
    pub const SCM_RIGHTS: c_int = 1;
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

    pub const AF_UNIX: c_int  = 1;
    pub const AF_LOCAL: c_int = AF_UNIX;
    pub const AF_INET: c_int  = 2;
    #[cfg(target_os = "netbsd")]
    pub const AF_INET6: c_int = 24;
    #[cfg(target_os = "openbsd")]
    pub const AF_INET6: c_int = 26;
    #[cfg(target_os = "freebsd")]
    pub const AF_INET6: c_int = 28;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub const AF_INET6: c_int = 30;

    pub const SOCK_STREAM: c_int = 1;
    pub const SOCK_DGRAM: c_int = 2;
    pub const SOCK_SEQPACKET: c_int = 5;
    pub const SOCK_RAW: c_int = 3;
    pub const SOCK_RDM: c_int = 4;

    pub const SOL_SOCKET: c_int = 0xffff;
    pub const IPPROTO_IP: c_int = 0;
    pub const IPPROTO_IPV6: c_int = 41;
    pub const IPPROTO_TCP: c_int = 6;
    pub const IPPROTO_UDP: c_int = 17;

    pub const SO_ACCEPTCONN: c_int          = 0x0002;
    pub const SO_BROADCAST: c_int           = 0x0020;
    pub const SO_DEBUG: c_int               = 0x0001;
    #[cfg(not(target_os = "netbsd"))]
    pub const SO_DONTTRUNC: c_int           = 0x2000;
    #[cfg(target_os = "netbsd")]
    pub const SO_USELOOPBACK: c_int         = 0x0040;
    pub const SO_ERROR: c_int               = 0x1007;
    pub const SO_DONTROUTE: c_int           = 0x0010;
    pub const SO_KEEPALIVE: c_int           = 0x0008;
    pub const SO_LABEL: c_int               = 0x1010;
    pub const SO_LINGER: c_int              = 0x0080;
    pub const SO_NREAD: c_int               = 0x1020;
    pub const SO_NKE: c_int                 = 0x1021;
    pub const SO_NOSIGPIPE: c_int           = 0x1022;
    pub const SO_NOADDRERR: c_int           = 0x1023;
    pub const SO_NOTIFYCONFLICT: c_int      = 0x1026;
    pub const SO_NP_EXTENSIONS: c_int       = 0x1083;
    pub const SO_NWRITE: c_int              = 0x1024;
    pub const SO_OOBINLINE: c_int           = 0x0100;
    pub const SO_PEERLABEL: c_int           = 0x1011;
    pub const SO_RCVBUF: c_int              = 0x1002;
    pub const SO_RCVLOWAT: c_int            = 0x1004;
    pub const SO_SNDLOWAT: c_int            = 0x1003;
    pub const SO_RCVTIMEO: c_int            = 0x1006;
    pub const SO_SNDTIMEO: c_int            = 0x1005;
    pub const SO_RANDOMPORT: c_int          = 0x1082;
    pub const SO_RESTRICTIONS: c_int        = 0x1081;
    pub const SO_RESTRICT_DENYIN: c_int     = 0x00000001;
    pub const SO_RESTRICT_DENYOUT: c_int    = 0x00000002;
    pub const SO_REUSEADDR: c_int           = 0x0004;
    pub const SO_REUSEPORT: c_int           = 0x0200;
    pub const SO_REUSESHAREUID: c_int       = 0x1025;
    pub const SO_SNDBUF: c_int              = 0x1001;
    #[cfg(not(target_os = "netbsd"))]
    pub const SO_TIMESTAMP: c_int           = 0x0400;
    #[cfg(not(target_os = "netbsd"))]
    pub const SO_TIMESTAMP_MONOTONIC: c_int = 0x0800;
    #[cfg(target_os = "netbsd")]
    pub const SO_TIMESTAMP: c_int           = 0x2000;
    pub const SO_TYPE: c_int                = 0x1008;
    #[cfg(not(target_os = "netbsd"))]
    pub const SO_WANTMORE: c_int            = 0x4000;
    pub const SO_WANTOOBFLAG: c_int         = 0x8000;
    #[allow(overflowing_literals)]
    pub const SO_RESTRICT_DENYSET: c_int    = 0x80000000;

    // Socket options for TCP sockets
    pub const TCP_NODELAY: c_int = 1;
    pub const TCP_MAXSEG: c_int = 2;
    #[cfg(any(target_os = "macos",
              target_os = "ios"))]
    pub const TCP_KEEPALIVE: c_int = libc::TCP_KEEPALIVE;
    #[cfg(target_os = "freebsd")]
    pub const TCP_KEEPIDLE: c_int = libc::TCP_KEEPIDLE;
    #[cfg(target_os = "netbsd")]
    pub const TCP_KEEPIDLE: c_int = 3;

    // Socket options for the IP layer of the socket
    pub const IP_MULTICAST_IF: c_int = 9;

    pub type IpMulticastTtl = uint8_t;

    pub const IP_MULTICAST_TTL: c_int = 10;
    pub const IP_MULTICAST_LOOP: c_int = 11;
    pub const IP_ADD_MEMBERSHIP: c_int = 12;
    pub const IP_DROP_MEMBERSHIP: c_int = 13;

    pub const IPV6_JOIN_GROUP: c_int = libc::IPV6_JOIN_GROUP;
    pub const IPV6_LEAVE_GROUP: c_int = libc::IPV6_LEAVE_GROUP;

    pub type InAddrT = u32;

    // Declarations of special addresses
    pub const INADDR_ANY: InAddrT = 0;
    pub const INADDR_NONE: InAddrT = 0xffffffff;
    pub const INADDR_BROADCAST: InAddrT = 0xffffffff;

    // Flags for send/recv and their relatives
    bitflags!{
        flags MsgFlags : libc::c_int {
            const MSG_OOB      = 0x01,
            const MSG_PEEK     = 0x02,
            const MSG_EOR      = 0x08,
            const MSG_TRUNC    = 0x10,
            const MSG_CTRUNC   = 0x20,
            const MSG_DONTWAIT = 0x80,
        }
    }

    // shutdown flags
    pub const SHUT_RD: c_int   = 0;
    pub const SHUT_WR: c_int   = 1;
    pub const SHUT_RDWR: c_int = 2;

    // Ancillary message types
    pub const SCM_RIGHTS: c_int = 1;
}

#[cfg(target_os = "dragonfly")]
mod os {
    use libc::{c_int, uint8_t};

    pub const AF_UNIX: c_int  = 1;
    pub const AF_LOCAL: c_int = AF_UNIX;
    pub const AF_INET: c_int  = 2;
    pub const AF_INET6: c_int = 28;

    pub const SOCK_STREAM: c_int = 1;
    pub const SOCK_DGRAM: c_int = 2;
    pub const SOCK_SEQPACKET: c_int = 5;
    pub const SOCK_RAW: c_int = 3;
    pub const SOCK_RDM: c_int = 4;

    pub const SOL_SOCKET: c_int = 0xffff;
    pub const IPPROTO_IP: c_int = 0;
    pub const IPPROTO_IPV6: c_int = 41;
    pub const IPPROTO_TCP: c_int = 6;
    pub const IPPROTO_UDP: c_int = 17;

    pub const SO_ACCEPTCONN: c_int          = 0x0002;
    pub const SO_BROADCAST: c_int           = 0x0020;
    pub const SO_DEBUG: c_int               = 0x0001;
    pub const SO_ERROR: c_int               = 0x1007;
    pub const SO_DONTROUTE: c_int           = 0x0010;
    pub const SO_KEEPALIVE: c_int           = 0x0008;
    pub const SO_LINGER: c_int              = 0x0080;
    pub const SO_NOSIGPIPE: c_int           = 0x0800; // different value!
    pub const SO_OOBINLINE: c_int           = 0x0100;
    pub const SO_RCVBUF: c_int              = 0x1002;
    pub const SO_RCVLOWAT: c_int            = 0x1004;
    pub const SO_SNDLOWAT: c_int            = 0x1003;
    pub const SO_RCVTIMEO: c_int            = 0x1006;
    pub const SO_SNDTIMEO: c_int            = 0x1005;
    pub const SO_REUSEADDR: c_int           = 0x0004;
    pub const SO_REUSEPORT: c_int           = 0x0200;
    pub const SO_SNDBUF: c_int              = 0x1001;
    pub const SO_TIMESTAMP: c_int           = 0x0400;
    pub const SO_TYPE: c_int                = 0x1008;

    // Socket options for TCP sockets
    pub const TCP_NODELAY: c_int = 1;
    pub const TCP_MAXSEG: c_int = 2;
    pub const TCP_KEEPIDLE: c_int = 0x100;

    // Socket options for the IP layer of the socket
    pub const IP_MULTICAST_IF: c_int = 9;

    pub type IpMulticastTtl = uint8_t;

    pub const IP_MULTICAST_TTL: c_int = 10;
    pub const IP_MULTICAST_LOOP: c_int = 11;
    pub const IP_ADD_MEMBERSHIP: c_int = 12;
    pub const IP_DROP_MEMBERSHIP: c_int = 13;
    pub const IPV6_JOIN_GROUP: c_int = libc::IPV6_JOIN_GROUP;
    pub const IPV6_LEAVE_GROUP: c_int = libc::IPV6_LEAVE_GROUP;

    pub type InAddrT = u32;

    // Declarations of special addresses
    pub const INADDR_ANY: InAddrT = 0;
    pub const INADDR_NONE: InAddrT = 0xffffffff;
    pub const INADDR_BROADCAST: InAddrT = 0xffffffff;

    // Flags for send/recv and their relatives
    bitflags!{
        flags MsgFlags : libc::c_int {
            const MSG_OOB      = 0x01,
            const MSG_PEEK     = 0x02,
            const MSG_DONTWAIT = 0x80,
        }
    }

    // shutdown flags
    pub const SHUT_RD: c_int   = 0;
    pub const SHUT_WR: c_int   = 1;
    pub const SHUT_RDWR: c_int = 2;
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
    pub fn test_linux_consts() {
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
            SO_PASSCRED,
            SO_PRIORITY,
            // SO_PROTOCOL,
            SO_RCVBUFFORCE,
            // SO_PEEK_OFF,
            SO_PEERCRED,
            SO_SNDBUFFORCE,
            MSG_ERRQUEUE);
    }
}
