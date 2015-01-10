/* TOOD: Implement for other kqueue based systems
 */

use libc::{timespec, time_t, c_int, c_long, uintptr_t};
use errno::{SysResult, SysError};
use fcntl::Fd;
use std::fmt;

pub use self::ffi::kevent as KEvent;

mod ffi {
    pub use libc::{c_int, c_void, uintptr_t, intptr_t, timespec};
    use super::{EventFilter, EventFlag, FilterFlag};

    #[derive(Copy)]
    #[repr(C)]
    pub struct kevent {
        pub ident: uintptr_t,       // 8
        pub filter: EventFilter,    // 2
        pub flags: EventFlag,       // 2
        pub fflags: FilterFlag,     // 4
        pub data: intptr_t,         // 8
        pub udata: usize             // 8
    }

    // Bug in rustc, cannot determine that kevent is #[repr(C)]
    #[allow(improper_ctypes)]
    extern {
        pub fn kqueue() -> c_int;

        pub fn kevent(
            kq: c_int,
            changelist: *const kevent,
            nchanges: c_int,
            eventlist: *mut kevent,
            nevents: c_int,
            timeout: *const timespec) -> c_int;
    }
}

#[repr(i16)]
#[derive(Copy, Show, PartialEq)]
pub enum EventFilter {
    EVFILT_READ = -1,
    EVFILT_WRITE = -2,
    EVFILT_AIO = -3,
    EVFILT_VNODE = -4,
    EVFILT_PROC = -5,
    EVFILT_SIGNAL = -6,
    EVFILT_TIMER = -7,
    EVFILT_MACHPORT = -8,
    EVFILT_FS = -9,
    EVFILT_USER = -10,
    // -11: unused
    EVFILT_VM = -12,
    EVFILT_SYSCOUNT = 13
}

bitflags!(
    flags EventFlag: u16 {
        const EV_ADD       = 0x0001,
        const EV_DELETE    = 0x0002,
        const EV_ENABLE    = 0x0004,
        const EV_DISABLE   = 0x0008,
        const EV_RECEIPT   = 0x0040,
        const EV_ONESHOT   = 0x0010,
        const EV_CLEAR     = 0x0020,
        const EV_DISPATCH  = 0x0080,
        const EV_SYSFLAGS  = 0xF000,
        const EV_FLAG0     = 0x1000,
        const EV_FLAG1     = 0x2000,
        const EV_EOF       = 0x8000,
        const EV_ERROR     = 0x4000
    }
);

impl fmt::Show for EventFlag {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut one = false;
        let flags = [
            (EV_ADD, "EV_ADD"),
            (EV_DELETE, "EV_DELETE"),
            (EV_ENABLE, "EV_ENABLE"),
            (EV_DISABLE, "EV_DISABLE"),
            (EV_RECEIPT, "EV_RECEIPT"),
            (EV_ONESHOT, "EV_ONESHOT"),
            (EV_CLEAR, "EV_CLEAR"),
            (EV_DISPATCH, "EV_DISPATCH"),
            (EV_SYSFLAGS, "EV_SYSFLAGS"),
            (EV_FLAG0, "EV_FLAG0"),
            (EV_FLAG1, "EV_FLAG1"),
            (EV_EOF, "EV_EOF")];

        for &(flag, msg) in flags.iter() {
            if self.contains(flag) {
                if one { try!(write!(fmt, " | ")) }
                try!(write!(fmt, "{}", msg));

                one = true
            }
        }

        if !one {
            try!(write!(fmt, "<None>"));
        }

        Ok(())
    }
}

bitflags!(
    flags FilterFlag: u32 {
        const NOTE_TRIGGER                         = 0x01000000,
        const NOTE_FFNOP                           = 0x00000000,
        const NOTE_FFAND                           = 0x40000000,
        const NOTE_FFOR                            = 0x80000000,
        const NOTE_FFCOPY                          = 0xc0000000,
        const NOTE_FFCTRLMASK                      = 0xc0000000,
        const NOTE_FFLAGSMASK                      = 0x00ffffff,
        const NOTE_LOWAT                           = 0x00000001,
        const NOTE_DELETE                          = 0x00000001,
        const NOTE_WRITE                           = 0x00000002,
        const NOTE_EXTEND                          = 0x00000004,
        const NOTE_ATTRIB                          = 0x00000008,
        const NOTE_LINK                            = 0x00000010,
        const NOTE_RENAME                          = 0x00000020,
        const NOTE_REVOKE                          = 0x00000040,
        const NOTE_NONE                            = 0x00000080,
        const NOTE_EXIT                            = 0x80000000,
        const NOTE_FORK                            = 0x40000000,
        const NOTE_EXEC                            = 0x20000000,
        const NOTE_REAP                            = 0x10000000,
        const NOTE_SIGNAL                          = 0x08000000,
        const NOTE_EXITSTATUS                      = 0x04000000,
        const NOTE_RESOURCEEND                     = 0x02000000,
        const NOTE_APPACTIVE                       = 0x00800000,
        const NOTE_APPBACKGROUND                   = 0x00400000,
        const NOTE_APPNONUI                        = 0x00200000,
        const NOTE_APPINACTIVE                     = 0x00100000,
        const NOTE_APPALLSTATES                    = 0x00f00000,
        const NOTE_PDATAMASK                       = 0x000fffff,
        const NOTE_PCTRLMASK                       = 0xfff00000,
        const NOTE_EXIT_REPARENTED                 = 0x00080000,
        const NOTE_VM_PRESSURE                     = 0x80000000,
        const NOTE_VM_PRESSURE_TERMINATE           = 0x40000000,
        const NOTE_VM_PRESSURE_SUDDEN_TERMINATE    = 0x20000000,
        const NOTE_VM_ERROR                        = 0x10000000,
        const NOTE_SECONDS                         = 0x00000001,
        const NOTE_USECONDS                        = 0x00000002,
        const NOTE_NSECONDS                        = 0x00000004,
        const NOTE_ABSOLUTE                        = 0x00000008,
        const NOTE_TRACK                           = 0x00000001,
        const NOTE_TRACKERR                        = 0x00000002,
        const NOTE_CHILD                           = 0x00000004
    }
);

pub const EV_POLL: EventFlag = EV_FLAG0;
pub const EV_OOBAND: EventFlag = EV_FLAG1;

pub fn kqueue() -> SysResult<Fd> {
    let res = unsafe { ffi::kqueue() };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
}

pub fn kevent(kq: Fd,
              changelist: &[KEvent],
              eventlist: &mut [KEvent],
              timeout_ms: usize) -> SysResult<usize> {

    // Convert ms to timespec
    let timeout = timespec {
        tv_sec: (timeout_ms / 1000) as time_t,
        tv_nsec: ((timeout_ms % 1000) * 1_000_000) as c_long
    };

    let res = unsafe {
        ffi::kevent(
            kq,
            changelist.as_ptr(),
            changelist.len() as c_int,
            eventlist.as_mut_ptr(),
            eventlist.len() as c_int,
            &timeout as *const timespec)
    };

    if res < 0 {
        return Err(SysError::last());
    }

    return Ok(res as usize)
}

#[inline]
pub fn ev_set(ev: &mut KEvent,
              ident: usize,
              filter: EventFilter,
              flags: EventFlag,
              fflags: FilterFlag,
              udata: usize) {

    ev.ident  = ident as uintptr_t;
    ev.filter = filter;
    ev.flags  = flags;
    ev.fflags = fflags;
    ev.data   = 0;
    ev.udata  = udata;
}
