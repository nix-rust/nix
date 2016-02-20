//! Posix Message Queue functions
//!
//! [Further reading and details on the C API](http://man7.org/linux/man-pages/man7/mq_overview.7.html)

use {Errno, Result};

use libc::{c_int, c_long, c_char, size_t, mode_t};
use std::ffi::CString;
use sys::stat::Mode;
use std::ptr;

pub use self::consts::*;

pub type MQd = c_int;

#[cfg(target_os = "linux")]
mod consts {
    use libc::c_int;

    bitflags!(
        flags MQ_OFlag: c_int {
            const O_RDONLY    = 0o00000000,
            const O_WRONLY    = 0o00000001,
            const O_RDWR      = 0o00000002,
            const O_CREAT     = 0o00000100,
            const O_EXCL      = 0o00000200,
            const O_NONBLOCK  = 0o00004000,
            const O_CLOEXEC   = 0o02000000,
        }
    );

    bitflags!(
        flags FdFlag: c_int {
            const FD_CLOEXEC = 1
        }
    );
}

mod ffi {
    use libc::{c_char, size_t, ssize_t, c_uint, c_int};
    use super::MQd;
    use super::MqAttr;

    #[allow(improper_ctypes)]
    extern "C" {
        pub fn mq_open(name: *const c_char, oflag: c_int, ...) -> MQd;

        pub fn mq_close (mqd: MQd) -> c_int;

        pub fn mq_unlink(name: *const c_char) -> c_int;

        pub fn mq_receive (mqd: MQd, msg_ptr: *const c_char, msg_len: size_t, msq_prio: *const c_uint) -> ssize_t;

        pub fn mq_send (mqd: MQd, msg_ptr: *const c_char, msg_len: size_t, msq_prio: c_uint) -> c_int;

        pub fn mq_getattr(mqd: MQd, attr: *mut MqAttr) -> c_int;

        pub fn mq_setattr(mqd: MQd, newattr: *const MqAttr, oldattr: *mut MqAttr) -> c_int;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MqAttr {
    pub mq_flags: c_long,
    pub mq_maxmsg: c_long,
    pub mq_msgsize: c_long,
    pub mq_curmsgs: c_long,
    pad: [c_long; 4]
}

impl MqAttr {
   pub fn new(mq_flags: c_long, mq_maxmsg: c_long, mq_msgsize: c_long, mq_curmsgs: c_long) -> MqAttr {
       MqAttr { mq_flags: mq_flags, mq_maxmsg: mq_maxmsg, mq_msgsize: mq_msgsize, mq_curmsgs: mq_curmsgs, pad: [0; 4] }
   }
}


pub fn mq_open(name: &CString, oflag: MQ_OFlag, mode: Mode, attr: Option<&MqAttr>) -> Result<MQd> {
    let attr_p = attr.map(|attr| attr as *const MqAttr).unwrap_or(ptr::null());
    let res = unsafe { ffi::mq_open(name.as_ptr(), oflag.bits(), mode.bits() as mode_t, attr_p) };

    Errno::result(res)
}

pub fn mq_unlink(name: &CString) -> Result<()> {
    let res = unsafe { ffi::mq_unlink(name.as_ptr()) };
    Errno::result(res).map(drop)
}

pub fn mq_close(mqdes: MQd) -> Result<()>  {
    let res = unsafe { ffi::mq_close(mqdes) };
    Errno::result(res).map(drop)
}


pub fn mq_receive(mqdes: MQd, message: &mut [u8], msq_prio: u32) -> Result<usize> {
    let len = message.len() as size_t;
    let res = unsafe { ffi::mq_receive(mqdes, message.as_mut_ptr() as *mut c_char, len, &msq_prio) };

    Errno::result(res).map(|r| r as usize)
}

pub fn mq_send(mqdes: MQd, message: &[u8], msq_prio: u32) -> Result<()> {
    let res = unsafe { ffi::mq_send(mqdes, message.as_ptr() as *const c_char, message.len(), msq_prio) };

    Errno::result(res).map(drop)
}

pub fn mq_getattr(mqd: MQd) -> Result<MqAttr> {
    let mut attr = MqAttr::new(0, 0, 0, 0);
    let res = unsafe { ffi::mq_getattr(mqd, &mut attr) };
    try!(Errno::result(res));
    Ok(attr)
}

/// Set the attributes of the message queue. Only O_NONBLOCK can be set, everything else will be ignored
/// Returns the old attributes
/// It is recommend to use the mq_set_nonblock() and mq_remove_nonblock() convenience functions as they are easier to use
///
/// [Further reading](http://man7.org/linux/man-pages/man3/mq_setattr.3.html)
pub fn mq_setattr(mqd: MQd, newattr: &MqAttr) -> Result<MqAttr> {
    let mut attr = MqAttr::new(0, 0, 0, 0);
    let res = unsafe { ffi::mq_setattr(mqd, newattr as *const MqAttr, &mut attr) };
    try!(Errno::result(res));
    Ok(attr)
}

/// Convenience function.
/// Sets the O_NONBLOCK attribute for a given message queue descriptor
/// Returns the old attributes
pub fn mq_set_nonblock(mqd: MQd) -> Result<(MqAttr)> {
    let oldattr = try!(mq_getattr(mqd));
    let newattr = MqAttr::new(O_NONBLOCK.bits() as c_long, oldattr.mq_maxmsg, oldattr.mq_msgsize, oldattr.mq_curmsgs);
    mq_setattr(mqd, &newattr)
}

/// Convenience function.
/// Removes O_NONBLOCK attribute for a given message queue descriptor
/// Returns the old attributes
pub fn mq_remove_nonblock(mqd: MQd) -> Result<(MqAttr)> {
    let oldattr = try!(mq_getattr(mqd));
    let newattr = MqAttr::new(0, oldattr.mq_maxmsg, oldattr.mq_msgsize, oldattr.mq_curmsgs);
    mq_setattr(mqd, &newattr)
}
