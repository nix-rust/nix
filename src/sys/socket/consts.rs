pub use self::os::*;

#[cfg(target_os = "linux")]
mod os {
    use libc::{c_int, uint8_t};

    pub type AddressFamily = c_int;

    pub const AF_UNIX: AddressFamily  = 1;
    pub const AF_LOCAL: AddressFamily = AF_UNIX;
    pub const AF_INET: AddressFamily  = 2;
    pub const AF_INET6: AddressFamily = 10;

    pub type SockType = c_int;

    pub const SOCK_STREAM: SockType = 1;
    pub const SOCK_DGRAM: SockType = 2;
    pub const SOCK_SEQPACKET: SockType = 5;
    pub const SOCK_RAW: SockType = 3;
    pub const SOCK_RDM: SockType = 4;

    pub type SockLevel = c_int;

    pub const SOL_IP: SockLevel     = 0;
    pub const IPPROTO_IP: SockLevel = SOL_IP;
    pub const SOL_SOCKET: SockLevel = 1;
    pub const SOL_TCP: SockLevel    = 6;
    pub const IPPROTO_TCP: SockLevel = SOL_TCP;
    pub const SOL_UDP: SockLevel    = 17;
    pub const SOL_IPV6: SockLevel   = 41;

    pub type SockOpt = c_int;

    pub const SO_ACCEPTCONN: SockOpt = 30;
    pub const SO_BINDTODEVICE: SockOpt = 25;
    pub const SO_BROADCAST: SockOpt = 6;
    pub const SO_BSDCOMPAT: SockOpt = 14;
    pub const SO_DEBUG: SockOpt = 1;
    pub const SO_DOMAIN: SockOpt = 39;
    pub const SO_ERROR: SockOpt = 4;
    pub const SO_DONTROUTE: SockOpt = 5;
    pub const SO_KEEPALIVE: SockOpt = 9;
    pub const SO_LINGER: SockOpt = 13;
    pub const SO_MARK: SockOpt = 36;
    pub const SO_OOBINLINE: SockOpt = 10;
    pub const SO_PASSCRED: SockOpt = 16;
    pub const SO_PEEK_OFF: SockOpt = 42;
    pub const SO_PEERCRED: SockOpt = 17;
    pub const SO_PRIORITY: SockOpt = 12;
    pub const SO_PROTOCOL: SockOpt = 38;
    pub const SO_RCVBUF: SockOpt = 8;
    pub const SO_RCVBUFFORCE: SockOpt = 33;
    pub const SO_RCVLOWAT: SockOpt = 18;
    pub const SO_SNDLOWAT: SockOpt = 19;
    pub const SO_RCVTIMEO: SockOpt = 20;
    pub const SO_SNDTIMEO: SockOpt = 21;
    pub const SO_REUSEADDR: SockOpt = 2;
    pub const SO_REUSEPORT: SockOpt = 15;
    pub const SO_RXQ_OVFL: SockOpt = 40;
    pub const SO_SNDBUF: SockOpt = 7;
    pub const SO_SNDBUFFORCE: SockOpt = 32;
    pub const SO_TIMESTAMP: SockOpt = 29;
    pub const SO_TYPE: SockOpt = 3;
    pub const SO_BUSY_POLL: SockOpt = 46;

    // Socket options for TCP sockets
    pub const TCP_NODELAY: SockOpt = 1;
    pub const TCP_MAXSEG: SockOpt = 2;
    pub const TCP_CORK: SockOpt = 3;

    // Socket options for the IP layer of the socket
    pub const IP_MULTICAST_IF: SockOpt = 32;

    pub type IpMulticastTtl = uint8_t;

    pub const IP_MULTICAST_TTL: SockOpt = 33;
    pub const IP_MULTICAST_LOOP: SockOpt = 34;
    pub const IP_ADD_MEMBERSHIP: SockOpt = 35;
    pub const IP_DROP_MEMBERSHIP: SockOpt = 36;

    pub type InAddrT = u32;

    // Declarations of special addresses
    pub const INADDR_ANY: InAddrT = 0;
    pub const INADDR_NONE: InAddrT = 0xffffffff;
    pub const INADDR_BROADCAST: InAddrT = 0xffffffff;

    pub type SockMessageFlags = i32;
    // Flags for send/recv and their relatives
    pub const MSG_OOB: SockMessageFlags = 0x1;
    pub const MSG_PEEK: SockMessageFlags = 0x2;
    pub const MSG_DONTWAIT: SockMessageFlags = 0x40;
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod os {
    use libc::{c_int, uint8_t};

    pub type AddressFamily = c_int;

    pub const AF_UNIX: AddressFamily  = 1;
    pub const AF_LOCAL: AddressFamily = AF_UNIX;
    pub const AF_INET: AddressFamily  = 2;
    pub const AF_INET6: AddressFamily = 30;

    pub type SockType = c_int;

    pub const SOCK_STREAM: SockType = 1;
    pub const SOCK_DGRAM: SockType = 2;
    pub const SOCK_SEQPACKET: SockType = 5;
    pub const SOCK_RAW: SockType = 3;
    pub const SOCK_RDM: SockType = 4;

    pub type SockLevel = c_int;

    pub const SOL_SOCKET: SockLevel = 0xffff;
    pub const IPPROTO_IP: SockLevel = 0;
    pub const IPPROTO_TCP: SockLevel = 6;
    pub const IPPROTO_UDP: SockLevel = 17;

    pub type SockOpt = c_int;

    pub const SO_ACCEPTCONN: SockOpt          = 0x0002;
    pub const SO_BROADCAST: SockOpt           = 0x0020;
    pub const SO_DEBUG: SockOpt               = 0x0001;
    pub const SO_DONTTRUNC: SockOpt           = 0x2000;
    pub const SO_ERROR: SockOpt               = 0x1007;
    pub const SO_DONTROUTE: SockOpt           = 0x0010;
    pub const SO_KEEPALIVE: SockOpt           = 0x0008;
    pub const SO_LABEL: SockOpt               = 0x1010;
    pub const SO_LINGER: SockOpt              = 0x0080;
    pub const SO_NREAD: SockOpt               = 0x1020;
    pub const SO_NKE: SockOpt                 = 0x1021;
    pub const SO_NOSIGPIPE: SockOpt           = 0x1022;
    pub const SO_NOADDRERR: SockOpt           = 0x1023;
    pub const SO_NOTIFYCONFLICT: SockOpt      = 0x1026;
    pub const SO_NP_EXTENSIONS: SockOpt       = 0x1083;
    pub const SO_NWRITE: SockOpt              = 0x1024;
    pub const SO_OOBINLINE: SockOpt           = 0x0100;
    pub const SO_PEERLABEL: SockOpt           = 0x1011;
    pub const SO_RCVBUF: SockOpt              = 0x1002;
    pub const SO_RCVLOWAT: SockOpt            = 0x1004;
    pub const SO_SNDLOWAT: SockOpt            = 0x1003;
    pub const SO_RCVTIMEO: SockOpt            = 0x1006;
    pub const SO_SNDTIMEO: SockOpt            = 0x1005;
    pub const SO_RANDOMPORT: SockOpt          = 0x1082;
    pub const SO_RESTRICTIONS: SockOpt        = 0x1081;
    pub const SO_RESTRICT_DENYIN: SockOpt     = 0x00000001;
    pub const SO_RESTRICT_DENYOUT: SockOpt    = 0x00000002;
    pub const SO_REUSEADDR: SockOpt           = 0x0004;
    pub const SO_REUSEPORT: SockOpt           = 0x0200;
    pub const SO_REUSESHAREUID: SockOpt       = 0x1025;
    pub const SO_SNDBUF: SockOpt              = 0x1001;
    pub const SO_TIMESTAMP: SockOpt           = 0x0400;
    pub const SO_TIMESTAMP_MONOTONIC: SockOpt = 0x0800;
    pub const SO_TYPE: SockOpt                = 0x1008;
    pub const SO_WANTMORE: SockOpt            = 0x4000;
    pub const SO_WANTOOBFLAG: SockOpt         = 0x8000;
    #[allow(overflowing_literals)]
    pub const SO_RESTRICT_DENYSET: SockOpt    = 0x80000000;

    // Socket options for TCP sockets
    pub const TCP_NODELAY: SockOpt = 1;
    pub const TCP_MAXSEG: SockOpt = 2;

    // Socket options for the IP layer of the socket
    pub const IP_MULTICAST_IF: SockOpt = 9;

    pub type IpMulticastTtl = uint8_t;

    pub const IP_MULTICAST_TTL: SockOpt = 10;
    pub const IP_MULTICAST_LOOP: SockOpt = 11;
    pub const IP_ADD_MEMBERSHIP: SockOpt = 12;
    pub const IP_DROP_MEMBERSHIP: SockOpt = 13;

    pub type InAddrT = u32;

    // Declarations of special addresses
    pub const INADDR_ANY: InAddrT = 0;
    pub const INADDR_NONE: InAddrT = 0xffffffff;
    pub const INADDR_BROADCAST: InAddrT = 0xffffffff;

    pub type SockMessageFlags = i32;
    // Flags for send/recv and their relatives
    pub const MSG_OOB: SockMessageFlags = 0x1;
    pub const MSG_PEEK: SockMessageFlags = 0x2;
    pub const MSG_DONTWAIT: SockMessageFlags = 0x80;
}
