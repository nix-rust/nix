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
            const MSG_OOB              = 0x00001,
            const MSG_PEEK             = 0x00002,
            const MSG_DONTROUTE        = 0x00004,
            const MSG_TRYHARD          = 0x00004, /* Synonym for MSG_DONTROUTE for DECnet */
            const MSG_CTRUNC           = 0x00008,
            const MSG_PROBE            = 0x00010, /* Do not send. Only probe path f.e. for MTU */
            const MSG_TRUNC            = 0x00020,
            const MSG_DONTWAIT         = 0x00040, /* Nonblocking io */
            const MSG_EOR              = 0x00080, /* End of record */
            const MSG_WAITALL          = 0x00100, /* Wait for a full request */
            const MSG_FIN              = 0x00200,
            const MSG_SYN              = 0x00400,
            const MSG_CONFIRM          = 0x00800, /* Confirm path validity */
            const MSG_RST              = 0x01000,
            const MSG_ERRQUEUE         = 0x02000, /* Fetch message from error queue */
            const MSG_NOSIGNAL         = 0x04000, /* Do not generate SIGPIPE */
            const MSG_MORE             = 0x08000, /* Sender will send more */
            const MSG_WAITFORONE       = 0x10000, /* recvmmsg(): block until 1+ packets avail */
            const MSG_SENDPAGE_NOTLAST = 0x20000, /* sendpage() internal : not the last page */
            const MSG_BATCH            = 0x40000, /* sendmmsg(): more messages coming */
            const MSG_EOF              = MSG_FIN,
            const MSG_FASTOPEN         = 0x20000000, /* Send data in TCP SYN */
            const MSG_CMSG_CLOEXEC     = 0x40000000, /* Set close_on_exec for file descriptor received through SCM_RIGHTS */
            const MSG_CMSG_COMPAT    = 0x80000000, /* This message needs 32 bit fixups */
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
            const MSG_OOB        = 0x00001, /* process out-of-band data */
            const MSG_PEEK       = 0x00002, /* peek at incoming message */
            const MSG_DONTROUTE  = 0x00004, /* send without using routing tables */
            const MSG_EOR        = 0x00008, /* data completes record */
            const MSG_TRUNC      = 0x00010, /* data discarded before delivery */
            const MSG_CTRUNC     = 0x00020, /* control data lost before delivery */
            const MSG_WAITALL    = 0x00040, /* wait for full request or error */
            const MSG_DONTWAIT   = 0x00080, /* this message should be nonblocking */
            const MSG_EOF        = 0x00100, /* data completes connection */
            const MSG_WAITSTREAM = 0x00200, /* wait up to full request.. may return partial */
            const MSG_FLUSH      = 0x00400, /* Start of 'hold' seq; dump so_temp */
            const MSG_HOLD       = 0x00800, /* Hold frag in so_temp */
            const MSG_SEND       = 0x01000, /* Send the packet in so_temp */
            const MSG_HAVEMORE   = 0x02000, /* Data ready to be read */
            const MSG_RCVMORE    = 0x04000, /* Data remains in current pkt */
            const MSG_NEEDSA     = 0x10000, /* Fail receive if socket address cannot be allocated */
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
        const MSG_OOB          = 0x00000001, /* process out-of-band data */
        const MSG_PEEK         = 0x00000002, /* peek at incoming message */
        const MSG_DONTROUTE    = 0x00000004, /* send without using routing tables */
        const MSG_EOR          = 0x00000008, /* data completes record */
        const MSG_TRUNC        = 0x00000010, /* data discarded before delivery */
        const MSG_CTRUNC       = 0x00000020, /* control data lost before delivery */
        const MSG_WAITALL      = 0x00000040, /* wait for full request or error */
        const MSG_DONTWAIT     = 0x00000080, /* this message should be nonblocking */
        const MSG_EOF          = 0x00000100, /* data completes connection */
        const MSG_UNUSED09     = 0x00000200, /* was: notification message (SCTP) */
        const MSG_NOSIGNAL     = 0x00000400, /* No SIGPIPE to unconnected socket stream */
        const MSG_SYNC         = 0x00000800, /* No asynchronized pru_send */
        const MSG_CMSG_CLOEXEC = 0x00001000, /* make received fds close-on-exec */
        const MSG_FBLOCKING    = 0x00010000, /* force blocking operation */
        const MSG_FNONBLOCKING = 0x00020000, /* force non-blocking operation */
        const MSG_FMASK        = 0xFFFF0000, /* force mask */
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
