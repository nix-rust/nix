//! Posix Message Queue functions
//!
//! [Further reading and details on the C API](http://man7.org/linux/man-pages/man7/mq_overview.7.html)

use {Errno, Result};

use libc::{self, c_char, c_long, mode_t, mqd_t, size_t};
use std::ffi::CString;
use sys::stat::Mode;
use std::mem;

libc_bitflags!{
    flags MQ_OFlag: libc::c_int {
        O_RDONLY,
        O_WRONLY,
        O_RDWR,
        O_CREAT,
        O_EXCL,
        O_NONBLOCK,
        O_CLOEXEC,
    }
}

libc_bitflags!{
    flags FdFlag: libc::c_int {
        FD_CLOEXEC,
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MqAttr {
    mq_attr: libc::mq_attr,
}

impl PartialEq<MqAttr> for MqAttr {
    fn eq(&self, other: &MqAttr) -> bool {
        let self_attr = self.mq_attr;
        let other_attr = other.mq_attr;
        self_attr.mq_flags == other_attr.mq_flags && self_attr.mq_maxmsg == other_attr.mq_maxmsg &&
        self_attr.mq_msgsize == other_attr.mq_msgsize &&
        self_attr.mq_curmsgs == other_attr.mq_curmsgs
    }
}

impl MqAttr {
    pub fn new(mq_flags: c_long,
               mq_maxmsg: c_long,
               mq_msgsize: c_long,
               mq_curmsgs: c_long)
               -> MqAttr {
        let mut attr = unsafe { mem::uninitialized::<libc::mq_attr>() };
        attr.mq_flags = mq_flags;
        attr.mq_maxmsg = mq_maxmsg;
        attr.mq_msgsize = mq_msgsize;
        attr.mq_curmsgs = mq_curmsgs;
        MqAttr { mq_attr: attr }
    }

    pub fn flags(&self) -> c_long {
        self.mq_attr.mq_flags
    }
}


pub fn mq_open(name: &CString,
               oflag: MQ_OFlag,
               mode: Mode,
               attr: Option<&MqAttr>)
               -> Result<mqd_t> {
    let res = match attr {
        Some(mq_attr) => unsafe {
            libc::mq_open(name.as_ptr(),
                          oflag.bits(),
                          mode.bits() as mode_t,
                          &mq_attr.mq_attr as *const libc::mq_attr)
        },
        None => unsafe { libc::mq_open(name.as_ptr(), oflag.bits()) },
    };
    Errno::result(res)
}

pub fn mq_unlink(name: &CString) -> Result<()> {
    let res = unsafe { libc::mq_unlink(name.as_ptr()) };
    Errno::result(res).map(drop)
}

pub fn mq_close(mqdes: mqd_t) -> Result<()> {
    let res = unsafe { libc::mq_close(mqdes) };
    Errno::result(res).map(drop)
}

pub fn mq_receive(mqdes: mqd_t, message: &mut [u8], msg_prio: &mut u32) -> Result<usize> {
    let len = message.len() as size_t;
    let res = unsafe {
        libc::mq_receive(mqdes,
                         message.as_mut_ptr() as *mut c_char,
                         len,
                         msg_prio as *mut u32)
    };
    Errno::result(res).map(|r| r as usize)
}

pub fn mq_send(mqdes: mqd_t, message: &[u8], msq_prio: u32) -> Result<()> {
    let res = unsafe {
        libc::mq_send(mqdes,
                      message.as_ptr() as *const c_char,
                      message.len(),
                      msq_prio)
    };
    Errno::result(res).map(drop)
}

pub fn mq_getattr(mqd: mqd_t) -> Result<MqAttr> {
    let mut attr = unsafe { mem::uninitialized::<libc::mq_attr>() };
    let res = unsafe { libc::mq_getattr(mqd, &mut attr) };
    Errno::result(res).map(|_| MqAttr { mq_attr: attr })
}

/// Set the attributes of the message queue. Only `O_NONBLOCK` can be set, everything else will be ignored
/// Returns the old attributes
/// It is recommend to use the `mq_set_nonblock()` and `mq_remove_nonblock()` convenience functions as they are easier to use
///
/// [Further reading](http://man7.org/linux/man-pages/man3/mq_setattr.3.html)
pub fn mq_setattr(mqd: mqd_t, newattr: &MqAttr) -> Result<MqAttr> {
    let mut attr = unsafe { mem::uninitialized::<libc::mq_attr>() };
    let res = unsafe { libc::mq_setattr(mqd, &newattr.mq_attr as *const libc::mq_attr, &mut attr) };
    Errno::result(res).map(|_| MqAttr { mq_attr: attr })
}

/// Convenience function.
/// Sets the `O_NONBLOCK` attribute for a given message queue descriptor
/// Returns the old attributes
pub fn mq_set_nonblock(mqd: mqd_t) -> Result<(MqAttr)> {
    let oldattr = try!(mq_getattr(mqd));
    let newattr = MqAttr::new(O_NONBLOCK.bits() as c_long,
                              oldattr.mq_attr.mq_maxmsg,
                              oldattr.mq_attr.mq_msgsize,
                              oldattr.mq_attr.mq_curmsgs);
    mq_setattr(mqd, &newattr)
}

/// Convenience function.
/// Removes `O_NONBLOCK` attribute for a given message queue descriptor
/// Returns the old attributes
pub fn mq_remove_nonblock(mqd: mqd_t) -> Result<(MqAttr)> {
    let oldattr = try!(mq_getattr(mqd));
    let newattr = MqAttr::new(0,
                              oldattr.mq_attr.mq_maxmsg,
                              oldattr.mq_attr.mq_msgsize,
                              oldattr.mq_attr.mq_curmsgs);
    mq_setattr(mqd, &newattr)
}
