use super::{ffi, consts, GetSockOpt, SetSockOpt};
use {Errno, Result};
use sys::time::TimeVal;
use libc::{c_int, uint8_t, c_void, socklen_t};
use std::mem;
use std::os::unix::io::RawFd;

macro_rules! setsockopt_impl {
    ($name:ident, $level:path, $flag:path, $ty:ty, $setter:ty) => {
        impl SetSockOpt for $name {
            type Val = $ty;

            fn set(&self, fd: RawFd, val: &$ty) -> Result<()> {
                unsafe {
                    let setter: $setter = Set::new(val);

                    let res = ffi::setsockopt(fd, $level, $flag,
                                              setter.ffi_ptr(),
                                              setter.ffi_len());
                    Errno::result(res).map(drop)
                }
            }
        }
    }
}

macro_rules! getsockopt_impl {
    ($name:ident, $level:path, $flag:path, $ty:ty, $getter:ty) => {
        impl GetSockOpt for $name {
            type Val = $ty;

            fn get(&self, fd: RawFd) -> Result<$ty> {
                unsafe {
                    let mut getter: $getter = Get::blank();

                    let res = ffi::getsockopt(fd, $level, $flag,
                                              getter.ffi_ptr(),
                                              getter.ffi_len());
                    try!(Errno::result(res));

                    Ok(getter.unwrap())
                }
            }
        }
    }
}

// Helper to generate the sockopt accessors
macro_rules! sockopt_impl {
    (GetOnly, $name:ident, $level:path, $flag:path, $ty:ty) => {
        sockopt_impl!(GetOnly, $name, $level, $flag, $ty, GetStruct<$ty>);
    };

    (GetOnly, $name:ident, $level:path, $flag:path, bool) => {
        sockopt_impl!(GetOnly, $name, $level, $flag, bool, GetBool);
    };

    (GetOnly, $name:ident, $level:path, $flag:path, u8) => {
        sockopt_impl!(GetOnly, $name, $level, $flag, u8, GetU8);
    };

    (GetOnly, $name:ident, $level:path, $flag:path, usize) => {
        sockopt_impl!(GetOnly, $name, $level, $flag, usize, GetUsize);
    };

    (GetOnly, $name:ident, $level:path, $flag:path, $ty:ty, $getter:ty) => {
        #[derive(Copy, Clone, Debug)]
        pub struct $name;

        getsockopt_impl!($name, $level, $flag, $ty, $getter);
    };

    (SetOnly, $name:ident, $level:path, $flag:path, $ty:ty) => {
        sockopt_impl!(SetOnly, $name, $level, $flag, $ty, SetStruct<$ty>);
    };

    (SetOnly, $name:ident, $level:path, $flag:path, bool) => {
        sockopt_impl!(SetOnly, $name, $level, $flag, bool, SetBool);
    };

    (SetOnly, $name:ident, $level:path, $flag:path, u8) => {
        sockopt_impl!(SetOnly, $name, $level, $flag, u8, SetU8);
    };

    (SetOnly, $name:ident, $level:path, $flag:path, usize) => {
        sockopt_impl!(SetOnly, $name, $level, $flag, usize, SetUsize);
    };

    (SetOnly, $name:ident, $level:path, $flag:path, $ty:ty, $setter:ty) => {
        #[derive(Copy, Clone, Debug)]
        pub struct $name;

        setsockopt_impl!($name, $level, $flag, $ty, $setter);
    };

    (Both, $name:ident, $level:path, $flag:path, $ty:ty, $getter:ty, $setter:ty) => {
        #[derive(Copy, Clone, Debug)]
        pub struct $name;

        setsockopt_impl!($name, $level, $flag, $ty, $setter);
        getsockopt_impl!($name, $level, $flag, $ty, $getter);
    };

    (Both, $name:ident, $level:path, $flag:path, bool) => {
        sockopt_impl!(Both, $name, $level, $flag, bool, GetBool, SetBool);
    };

    (Both, $name:ident, $level:path, $flag:path, u8) => {
        sockopt_impl!(Both, $name, $level, $flag, u8, GetU8, SetU8);
    };

    (Both, $name:ident, $level:path, $flag:path, usize) => {
        sockopt_impl!(Both, $name, $level, $flag, usize, GetUsize, SetUsize);
    };

    (Both, $name:ident, $level:path, $flag:path, $ty:ty) => {
        sockopt_impl!(Both, $name, $level, $flag, $ty, GetStruct<$ty>, SetStruct<$ty>);
    };
}

/*
 *
 * ===== Define sockopts =====
 *
 */

sockopt_impl!(Both, ReuseAddr, consts::SOL_SOCKET, consts::SO_REUSEADDR, bool);
sockopt_impl!(Both, ReusePort, consts::SOL_SOCKET, consts::SO_REUSEPORT, bool);
sockopt_impl!(Both, TcpNoDelay, consts::IPPROTO_TCP, consts::TCP_NODELAY, bool);
sockopt_impl!(Both, Linger, consts::SOL_SOCKET, consts::SO_LINGER, super::linger);
sockopt_impl!(SetOnly, IpAddMembership, consts::IPPROTO_IP, consts::IP_ADD_MEMBERSHIP, super::ip_mreq);
sockopt_impl!(SetOnly, IpDropMembership, consts::IPPROTO_IP, consts::IP_DROP_MEMBERSHIP, super::ip_mreq);
#[cfg(not(any(target_os = "dragonfly", target_os = "freebsd", target_os = "ios", target_os = "macos", target_os = "netbsd", target_os = "openbsd")))]
sockopt_impl!(SetOnly, Ipv6AddMembership, consts::IPPROTO_IPV6, consts::IPV6_ADD_MEMBERSHIP, super::ipv6_mreq);
#[cfg(not(any(target_os = "dragonfly", target_os = "freebsd", target_os = "ios", target_os = "macos", target_os = "netbsd", target_os = "openbsd")))]
sockopt_impl!(SetOnly, Ipv6DropMembership, consts::IPPROTO_IPV6, consts::IPV6_DROP_MEMBERSHIP, super::ipv6_mreq);
#[cfg(any(target_os = "dragonfly", target_os = "freebsd", target_os = "ios", target_os = "macos", target_os = "netbsd", target_os = "openbsd"))]
sockopt_impl!(SetOnly, Ipv6AddMembership, consts::IPPROTO_IPV6, consts::IPV6_JOIN_GROUP, super::ipv6_mreq);
#[cfg(any(target_os = "dragonfly", target_os = "freebsd", target_os = "ios", target_os = "macos", target_os = "netbsd", target_os = "openbsd"))]
sockopt_impl!(SetOnly, Ipv6DropMembership, consts::IPPROTO_IPV6, consts::IPV6_LEAVE_GROUP, super::ipv6_mreq);
sockopt_impl!(Both, IpMulticastTtl, consts::IPPROTO_IP, consts::IP_MULTICAST_TTL, u8);
sockopt_impl!(Both, IpMulticastLoop, consts::IPPROTO_IP, consts::IP_MULTICAST_LOOP, bool);
sockopt_impl!(Both, ReceiveTimeout, consts::SOL_SOCKET, consts::SO_RCVTIMEO, TimeVal);
sockopt_impl!(Both, SendTimeout, consts::SOL_SOCKET, consts::SO_SNDTIMEO, TimeVal);
sockopt_impl!(Both, Broadcast, consts::SOL_SOCKET, consts::SO_BROADCAST, bool);
sockopt_impl!(Both, OobInline, consts::SOL_SOCKET, consts::SO_OOBINLINE, bool);
sockopt_impl!(GetOnly, SocketError, consts::SOL_SOCKET, consts::SO_ERROR, i32);
sockopt_impl!(Both, KeepAlive, consts::SOL_SOCKET, consts::SO_KEEPALIVE, bool);
#[cfg(target_os = "linux")]
sockopt_impl!(GetOnly, PeerCredentials, consts::SOL_SOCKET, consts::SO_PEERCRED, super::ucred);
#[cfg(any(target_os = "macos",
          target_os = "ios"))]
sockopt_impl!(Both, TcpKeepAlive, consts::IPPROTO_TCP, consts::TCP_KEEPALIVE, u32);
#[cfg(any(target_os = "freebsd",
          target_os = "dragonfly",
          target_os = "linux",
          target_os = "android",
          target_os = "nacl"))]
sockopt_impl!(Both, TcpKeepIdle, consts::IPPROTO_TCP, consts::TCP_KEEPIDLE, u32);
sockopt_impl!(Both, RcvBuf, consts::SOL_SOCKET, consts::SO_RCVBUF, usize);
sockopt_impl!(Both, SndBuf, consts::SOL_SOCKET, consts::SO_SNDBUF, usize);
#[cfg(target_os = "linux")]
sockopt_impl!(SetOnly, RcvBufForce, consts::SOL_SOCKET, consts::SO_RCVBUFFORCE, usize);
#[cfg(target_os = "linux")]
sockopt_impl!(SetOnly, SndBufForce, consts::SOL_SOCKET, consts::SO_SNDBUFFORCE, usize);
sockopt_impl!(GetOnly, SockType, consts::SOL_SOCKET, consts::SO_TYPE, super::SockType);
#[cfg(any(target_os = "freebsd",
          target_os = "linux",
          target_os = "nacl"))]
sockopt_impl!(GetOnly, AcceptConn, consts::SOL_SOCKET, consts::SO_ACCEPTCONN, bool);

/*
 *
 * ===== Accessor helpers =====
 *
 */

trait Get<T> {
    unsafe fn blank() -> Self;
    unsafe fn ffi_ptr(&mut self) -> *mut c_void;
    unsafe fn ffi_len(&mut self) -> *mut socklen_t;
    unsafe fn unwrap(self) -> T;
}

trait Set<'a, T> {
    fn new(val: &'a T) -> Self;
    unsafe fn ffi_ptr(&self) -> *const c_void;
    unsafe fn ffi_len(&self) -> socklen_t;
}

struct GetStruct<T> {
    len: socklen_t,
    val: T,
}

impl<T> Get<T> for GetStruct<T> {
    unsafe fn blank() -> Self {
        GetStruct {
            len: mem::size_of::<T>() as socklen_t,
            val: mem::zeroed(),
        }
    }

    unsafe fn ffi_ptr(&mut self) -> *mut c_void {
        mem::transmute(&mut self.val)
    }

    unsafe fn ffi_len(&mut self) -> *mut socklen_t {
        mem::transmute(&mut self.len)
    }

    unsafe fn unwrap(self) -> T {
        assert!(self.len as usize == mem::size_of::<T>(), "invalid getsockopt implementation");
        self.val
    }
}

struct SetStruct<'a, T: 'static> {
    ptr: &'a T,
}

impl<'a, T> Set<'a, T> for SetStruct<'a, T> {
    fn new(ptr: &'a T) -> SetStruct<'a, T> {
        SetStruct { ptr: ptr }
    }

    unsafe fn ffi_ptr(&self) -> *const c_void {
        mem::transmute(self.ptr)
    }

    unsafe fn ffi_len(&self) -> socklen_t {
        mem::size_of::<T>() as socklen_t
    }
}

struct GetBool {
    len: socklen_t,
    val: c_int,
}

impl Get<bool> for GetBool {
    unsafe fn blank() -> Self {
        GetBool {
            len: mem::size_of::<c_int>() as socklen_t,
            val: mem::zeroed(),
        }
    }

    unsafe fn ffi_ptr(&mut self) -> *mut c_void {
        mem::transmute(&mut self.val)
    }

    unsafe fn ffi_len(&mut self) -> *mut socklen_t {
        mem::transmute(&mut self.len)
    }

    unsafe fn unwrap(self) -> bool {
        assert!(self.len as usize == mem::size_of::<c_int>(), "invalid getsockopt implementation");
        self.val != 0
    }
}

struct SetBool {
    val: c_int,
}

impl<'a> Set<'a, bool> for SetBool {
    fn new(val: &'a bool) -> SetBool {
        SetBool { val: if *val { 1 } else { 0 } }
    }

    unsafe fn ffi_ptr(&self) -> *const c_void {
        mem::transmute(&self.val)
    }

    unsafe fn ffi_len(&self) -> socklen_t {
        mem::size_of::<c_int>() as socklen_t
    }
}

struct GetU8 {
    len: socklen_t,
    val: uint8_t,
}

impl Get<u8> for GetU8 {
    unsafe fn blank() -> Self {
        GetU8 {
            len: mem::size_of::<uint8_t>() as socklen_t,
            val: mem::zeroed(),
        }
    }

    unsafe fn ffi_ptr(&mut self) -> *mut c_void {
        mem::transmute(&mut self.val)
    }

    unsafe fn ffi_len(&mut self) -> *mut socklen_t {
        mem::transmute(&mut self.len)
    }

    unsafe fn unwrap(self) -> u8 {
        assert!(self.len as usize == mem::size_of::<uint8_t>(), "invalid getsockopt implementation");
        self.val as u8
    }
}

struct SetU8 {
    val: uint8_t,
}

impl<'a> Set<'a, u8> for SetU8 {
    fn new(val: &'a u8) -> SetU8 {
        SetU8 { val: *val as uint8_t }
    }

    unsafe fn ffi_ptr(&self) -> *const c_void {
        mem::transmute(&self.val)
    }

    unsafe fn ffi_len(&self) -> socklen_t {
        mem::size_of::<c_int>() as socklen_t
    }
}

struct GetUsize {
    len: socklen_t,
    val: c_int,
}

impl Get<usize> for GetUsize {
    unsafe fn blank() -> Self {
        GetUsize {
            len: mem::size_of::<c_int>() as socklen_t,
            val: mem::zeroed(),
        }
    }

    unsafe fn ffi_ptr(&mut self) -> *mut c_void {
        mem::transmute(&mut self.val)
    }

    unsafe fn ffi_len(&mut self) -> *mut socklen_t {
        mem::transmute(&mut self.len)
    }

    unsafe fn unwrap(self) -> usize {
        assert!(self.len as usize == mem::size_of::<c_int>(), "invalid getsockopt implementation");
        self.val as usize
    }
}

struct SetUsize {
    val: c_int,
}

impl<'a> Set<'a, usize> for SetUsize {
    fn new(val: &'a usize) -> SetUsize {
        SetUsize { val: *val as c_int }
    }

    unsafe fn ffi_ptr(&self) -> *const c_void {
        mem::transmute(&self.val)
    }

    unsafe fn ffi_len(&self) -> socklen_t {
        mem::size_of::<c_int>() as socklen_t
    }
}

#[cfg(test)]
mod test {
    #[cfg(target_os = "linux")]
    #[test]
    fn can_get_peercred_on_unix_socket() {
        use super::super::*;

        let (a, b) = socketpair(AddressFamily::Unix, SockType::Stream, 0, SockFlag::empty()).unwrap();
        let a_cred = getsockopt(a, super::PeerCredentials).unwrap();
        let b_cred = getsockopt(b, super::PeerCredentials).unwrap();
        assert_eq!(a_cred, b_cred);
        assert!(a_cred.pid != 0);
    }

    #[test]
    fn is_socket_type_unix() {
        use super::super::*;
        use ::unistd::close;

        let (a, b) = socketpair(AddressFamily::Unix, SockType::Stream, 0, SockFlag::empty()).unwrap();
        let a_type = getsockopt(a, super::SockType).unwrap();
        assert!(a_type == SockType::Stream);
        close(a).unwrap();
        close(b).unwrap();
    }

    #[test]
    fn is_socket_type_dgram() {
        use super::super::*;
        use ::unistd::close;

        let s = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), 0).unwrap();
        let s_type = getsockopt(s, super::SockType).unwrap();
        assert!(s_type == SockType::Datagram);
        close(s).unwrap();
    }

    #[cfg(any(target_os = "freebsd",
              target_os = "linux",
              target_os = "nacl"))]
    #[test]
    fn can_get_listen_on_tcp_socket() {
        use super::super::*;
        use ::unistd::close;

        let s = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), 0).unwrap();
        let s_listening = getsockopt(s, super::AcceptConn).unwrap();
        assert!(!s_listening);
        listen(s, 10).unwrap();
        let s_listening2 = getsockopt(s, super::AcceptConn).unwrap();
        assert!(s_listening2);
        close(s).unwrap();
    }
}
