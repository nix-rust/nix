/* TOOD: Implement for other kqueue based systems
 */

use {Errno, Result};
#[cfg(not(target_os = "netbsd"))]
use libc::{timespec, time_t, c_int, c_long, uintptr_t};
#[cfg(target_os = "netbsd")]
use libc::{timespec, time_t, c_long, uintptr_t, size_t};
use std::os::unix::io::RawFd;
use std::ptr;

pub use self::ffi::kevent as KEvent;

mod ffi {
    pub use libc::{c_int, c_void, uintptr_t, intptr_t, timespec, size_t, int64_t};
    use super::{EventFilter, EventFlag, FilterFlag};

    #[cfg(not(target_os = "netbsd"))]
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct kevent {
        pub ident: uintptr_t,       // 8
        pub filter: EventFilter,    // 2
        pub flags: EventFlag,       // 2
        pub fflags: FilterFlag,     // 4
        pub data: intptr_t,         // 8
        pub udata: usize             // 8
    }

    #[cfg(target_os = "netbsd")]
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct kevent {
        pub ident: uintptr_t,
        pub filter: EventFilter,
        pub flags: EventFlag,
        pub fflags: FilterFlag,
        pub data: int64_t,
        pub udata: intptr_t
    }

    // Bug in rustc, cannot determine that kevent is #[repr(C)]
    #[allow(improper_ctypes)]
    extern {
        pub fn kqueue() -> c_int;

        #[cfg(not(target_os = "netbsd"))]
        pub fn kevent(
            kq: c_int,
            changelist: *const kevent,
            nchanges: c_int,
            eventlist: *mut kevent,
            nevents: c_int,
            timeout: *const timespec) -> c_int;

        #[cfg(target_os = "netbsd")]
        pub fn kevent(
            kq: c_int,
            changelist: *const kevent,
            nchanges: size_t,
            eventlist: *mut kevent,
            nevents: size_t,
            timeout: *const timespec) -> c_int;
    }
}

#[cfg(not(any(target_os = "dragonfly", target_os = "netbsd")))]
#[repr(i16)]
#[derive(Clone, Copy, Debug, PartialEq)]
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

#[cfg(target_os = "dragonfly")]
#[repr(i16)] // u_short
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventFilter {
    EVFILT_READ = -1,
    EVFILT_WRITE = -2,
    EVFILT_AIO = -3,
    EVFILT_VNODE = -4,
    EVFILT_PROC = -5,
    EVFILT_SIGNAL = -6,
    EVFILT_TIMER = -7,
    EVFILT_EXCEPT = -8,
    EVFILT_USER = -9,
}

#[cfg(target_os = "netbsd")]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventFilter {
    EVFILT_READ = 0,
    EVFILT_WRITE = 1,
    EVFILT_AIO = 2,
    EVFILT_VNODE = 3,
    EVFILT_PROC = 4,
    EVFILT_SIGNAL = 5,
    EVFILT_TIMER = 6,
    EVFILT_SYSCOUNT = 7
}

#[cfg(not(any(target_os = "dragonfly", target_os = "netbsd")))]
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

#[cfg(target_os = "dragonfly")]
bitflags!(
    flags EventFlag: u16 {
        const EV_ADD       = 0x0001,
        const EV_DELETE    = 0x0002,
        const EV_ENABLE    = 0x0004,
        const EV_DISABLE   = 0x0008,
        const EV_RECEIPT   = 0x0040,
        const EV_ONESHOT   = 0x0010,
        const EV_CLEAR     = 0x0020,
        const EV_SYSFLAGS  = 0xF000,
        const EV_NODATA    = 0x1000,
        const EV_FLAG1     = 0x2000,
        const EV_EOF       = 0x8000,
        const EV_ERROR     = 0x4000
    }
);

#[cfg(target_os = "netbsd")]
bitflags!(
    flags EventFlag: u32 {
        const EV_ADD       = 0x0001,
        const EV_DELETE    = 0x0002,
        const EV_ENABLE    = 0x0004,
        const EV_DISABLE   = 0x0008,
        const EV_ONESHOT   = 0x0010,
        const EV_CLEAR     = 0x0020,
        const EV_SYSFLAGS  = 0xF000,
        const EV_NODATA    = 0x1000,
        const EV_FLAG1     = 0x2000,
        const EV_EOF       = 0x8000,
        const EV_ERROR     = 0x4000
    }
);

#[cfg(not(any(target_os = "dragonfly", target_os="netbsd")))]
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

#[cfg(target_os = "dragonfly")]
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
        const NOTE_EXIT                            = 0x80000000,
        const NOTE_FORK                            = 0x40000000,
        const NOTE_EXEC                            = 0x20000000,
        const NOTE_SIGNAL                          = 0x08000000,
        const NOTE_PDATAMASK                       = 0x000fffff,
        const NOTE_PCTRLMASK                       = 0xf0000000, // NOTE: FreeBSD uses 0xfff00000,
        const NOTE_TRACK                           = 0x00000001,
        const NOTE_TRACKERR                        = 0x00000002,
        const NOTE_CHILD                           = 0x00000004
    }
);

#[cfg(target_os = "netbsd")]
bitflags!(
    flags FilterFlag: u32 {
        const NOTE_LOWAT                           = 0x00000001,
        const NOTE_DELETE                          = 0x00000001,
        const NOTE_WRITE                           = 0x00000002,
        const NOTE_EXTEND                          = 0x00000004,
        const NOTE_ATTRIB                          = 0x00000008,
        const NOTE_LINK                            = 0x00000010,
        const NOTE_RENAME                          = 0x00000020,
        const NOTE_REVOKE                          = 0x00000040,
        const NOTE_EXIT                            = 0x80000000,
        const NOTE_FORK                            = 0x40000000,
        const NOTE_EXEC                            = 0x20000000,
        const NOTE_SIGNAL                          = 0x08000000,
        const NOTE_PDATAMASK                       = 0x000fffff,
        const NOTE_PCTRLMASK                       = 0xf0000000, // NOTE: FreeBSD uses 0xfff00000,
        const NOTE_TRACK                           = 0x00000001,
        const NOTE_TRACKERR                        = 0x00000002,
        const NOTE_CHILD                           = 0x00000004
    }
);

#[cfg(not(any(target_os = "dragonfly", target_os = "netbsd")))]
pub const EV_POLL: EventFlag = EV_FLAG0;

#[cfg(not(any(target_os = "dragonfly", target_os = "netbsd")))]
pub const EV_OOBAND: EventFlag = EV_FLAG1;

pub fn kqueue() -> Result<RawFd> {
    let res = unsafe { ffi::kqueue() };

    Errno::result(res)
}

pub fn kevent(kq: RawFd,
              changelist: &[KEvent],
              eventlist: &mut [KEvent],
              timeout_ms: usize) -> Result<usize> {

    // Convert ms to timespec
    let timeout = timespec {
        tv_sec: (timeout_ms / 1000) as time_t,
        tv_nsec: ((timeout_ms % 1000) * 1_000_000) as c_long
    };

    kevent_ts(kq, changelist, eventlist, Some(timeout))
}

#[cfg(not(target_os = "netbsd"))]
pub fn kevent_ts(kq: RawFd,
              changelist: &[KEvent],
              eventlist: &mut [KEvent],
              timeout_opt: Option<timespec>) -> Result<usize> {

    let res = unsafe {
        ffi::kevent(
            kq,
            changelist.as_ptr(),
            changelist.len() as c_int,
            eventlist.as_mut_ptr(),
            eventlist.len() as c_int,
            if let Some(ref timeout) = timeout_opt {timeout as *const timespec} else {ptr::null()})
    };

    Errno::result(res).map(|r| r as usize)
}

#[cfg(target_os = "netbsd")]
pub fn kevent_ts(kq: RawFd,
              changelist: &[KEvent],
              eventlist: &mut [KEvent],
              timeout_opt: Option<timespec>) -> Result<usize> {

    let res = unsafe {
        ffi::kevent(
            kq,
            changelist.as_ptr(),
            changelist.len() as size_t,
            eventlist.as_mut_ptr(),
            eventlist.len() as size_t,
            if let Some(ref timeout) = timeout_opt {timeout as *const timespec} else {ptr::null()})
    };

    Errno::result(res).map(|r| r as usize)
}

#[cfg(not(target_os = "netbsd"))]
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

#[cfg(target_os = "netbsd")]
#[inline]
pub fn ev_set(ev: &mut KEvent,
              ident: usize,
              filter: EventFilter,
              flags: EventFlag,
              fflags: FilterFlag,
              udata: i64) {

    ev.ident  = ident as uintptr_t;
    ev.filter = filter;
    ev.flags  = flags;
    ev.fflags = fflags;
    ev.data   = 0;
    ev.udata  = udata;
}
