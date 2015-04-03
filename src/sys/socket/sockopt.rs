use {Result, Error, from_ffi};
use super::{ffi, consts, SockOpt};
use errno::Errno;
use sys::time::TimeVal;
use libc::{c_int, uint8_t, c_void, socklen_t};
use std::mem;
use std::os::unix::io::RawFd;

// Helper to generate the sockopt accessors
// TODO: Figure out how to ommit gets when not supported by opt
macro_rules! sockopt_impl {
    ($name:ident, $flag:path, bool) => {
        sockopt_impl!($name, $flag, bool, GetBool, bool, SetBool);
    };

    ($name:ident, $flag:path, u8) => {
        sockopt_impl!($name, $flag, u8, GetU8, u8, SetU8);
    };

    ($name:ident, $flag:path, $ty:ty) => {
        sockopt_impl!($name, $flag, $ty, GetStruct<$ty>, &'a $ty, SetStruct<$ty>);
    };

    ($name:ident, $flag:path, $get_ty:ty, $getter:ty, $set_ty:ty, $setter:ty) => {
        #[derive(Copy, Debug)]
        pub struct $name;

        impl<'a> SockOpt for $name {
            type Get = $get_ty;
            type Set = $set_ty;

            fn get(&self, fd: RawFd, level: c_int) -> Result<$get_ty> {
                unsafe {
                    let mut getter: $getter = Get::blank();

                    let res = ffi::getsockopt(
                        fd, level, $flag,
                        getter.ffi_ptr(),
                        getter.ffi_len());

                    if res < 0 {
                        return Err(Error::Sys(Errno::last()));
                    }

                    Ok(getter.unwrap())
                }
            }

            fn set(&self, fd: RawFd, level: c_int, val: $set_ty) -> Result<()> {
                unsafe {
                    let setter: $setter = Set::new(val);

                    let res = ffi::setsockopt(
                        fd, level, $flag,
                        setter.ffi_ptr(),
                        setter.ffi_len());

                    from_ffi(res)
                }
            }
        }
    };
}

/*
 *
 * ===== Define sockopts =====
 *
 */

sockopt_impl!(ReuseAddr, consts::SO_REUSEADDR, bool);
sockopt_impl!(ReusePort, consts::SO_REUSEPORT, bool);
sockopt_impl!(TcpNoDelay, consts::TCP_NODELAY, bool);
sockopt_impl!(Linger, consts::SO_LINGER, super::linger);
sockopt_impl!(IpAddMembership, consts::IP_ADD_MEMBERSHIP, super::ip_mreq);
sockopt_impl!(IpDropMembership, consts::IP_DROP_MEMBERSHIP, super::ip_mreq);
sockopt_impl!(IpMulticastTtl, consts::IP_MULTICAST_TTL, u8);
sockopt_impl!(ReceiveTimeout, consts::SO_RCVTIMEO, TimeVal);
sockopt_impl!(SendTimeout, consts::SO_SNDTIMEO, TimeVal);
sockopt_impl!(Broadcast, consts::SO_BROADCAST, bool);

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

trait Set<T> {
    fn new(val: T) -> Self;
    unsafe fn ffi_ptr(&self) -> *const c_void;
    unsafe fn ffi_len(&self) -> socklen_t;
}

struct GetStruct<T> {
    len: socklen_t,
    val: T,
}

impl<T> Get<T> for GetStruct<T> {
    unsafe fn blank() -> Self {
        mem::zeroed()
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

impl<'a, T> Set<&'a T> for SetStruct<'a, T> {
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
        mem::zeroed()
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

impl Set<bool> for SetBool {
    fn new(val: bool) -> SetBool {
        SetBool { val: if val { 1 } else { 0 } }
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
        mem::zeroed()
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

impl Set<u8> for SetU8 {
    fn new(val: u8) -> SetU8 {
        SetU8 { val: val as uint8_t }
    }

    unsafe fn ffi_ptr(&self) -> *const c_void {
        mem::transmute(&self.val)
    }

    unsafe fn ffi_len(&self) -> socklen_t {
        mem::size_of::<c_int>() as socklen_t
    }
}
