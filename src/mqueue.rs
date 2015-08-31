use {Error, Result, from_ffi};
use errno::Errno;

use libc::{c_int, c_long, c_char, size_t, mode_t, strlen};
use std::ffi::CString;
use sys::stat::Mode;

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


#[inline]
pub fn mq_open(name: &CString, oflag: MQ_OFlag, mode: Mode, attr: &MqAttr) -> Result<MQd> {
    let res = unsafe { ffi::mq_open(name.as_ptr(), oflag.bits(), mode.bits() as mode_t, attr as *const MqAttr) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(res)
}

pub fn mq_unlink(name: &CString) -> Result<()> {
    let res = unsafe { ffi::mq_unlink(name.as_ptr()) };
    from_ffi(res)
}

pub fn mq_close(mqdes: MQd) -> Result<()>  {
    let res = unsafe { ffi::mq_close(mqdes) };
    from_ffi(res)
}


pub fn mq_receive(mqdes: MQd, message: &mut [u8], msq_prio: u32) -> Result<usize> {
    let len = message.len() as size_t;
    let res = unsafe { ffi::mq_receive(mqdes, message.as_mut_ptr() as *mut c_char, len, &msq_prio) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(res as usize)
}

pub fn mq_send(mqdes: MQd, message: &CString, msq_prio: u32) -> Result<usize> {
    let len = unsafe { strlen(message.as_ptr()) as size_t };
    let res = unsafe { ffi::mq_send(mqdes, message.as_ptr(), len, msq_prio) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(res as usize)
}

pub fn mq_getattr(mqd: MQd) -> Result<MqAttr> {
    let mut attr = MqAttr::new(0, 0, 0, 0);
    let res = unsafe { ffi::mq_getattr(mqd, &mut attr) };
    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }
    Ok(attr)
}


pub fn mq_setattr(mqd: MQd, newattr: &MqAttr) -> Result<MqAttr> {
    let mut attr = MqAttr::new(0, 0, 0, 0);
    let res = unsafe { ffi::mq_setattr(mqd, newattr as *const MqAttr, &mut attr) };
    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }
    Ok(attr)
}
