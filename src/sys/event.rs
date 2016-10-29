/* TOOD: Implement for other kqueue based systems
 */

use {Errno, Result};
#[cfg(not(target_os = "netbsd"))]
use libc::{timespec, time_t, c_int, c_long, intptr_t, uintptr_t};
#[cfg(target_os = "netbsd")]
use libc::{timespec, time_t, c_long, intptr_t, uintptr_t, size_t};
use libc;
use std::os::unix::io::RawFd;
use std::ptr;
use std::mem;

// Redefine kevent in terms of programmer-friendly enums and bitfields.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct KEvent {
    kevent: libc::kevent,
}

#[cfg(any(target_os = "openbsd", target_os = "freebsd",
          target_os = "dragonfly", target_os = "macos"))]
type type_of_udata = *mut ::c_void;
#[cfg(any(target_os = "netbsd"))]
type type_of_udata = intptr_t;

#[cfg(not(target_os = "netbsd"))]
type type_of_event_filter = i16;
#[cfg(not(target_os = "netbsd"))]
#[repr(i16)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventFilter {
    EVFILT_AIO = libc::EVFILT_AIO,
    #[cfg(target_os = "dragonfly")]
    EVFILT_EXCEPT = libc::EVFILT_EXCEPT,
    #[cfg(any(target_os = "macos",
              target_os = "dragonfly",
              target_os = "freebsd"))]
    EVFILT_FS = libc::EVFILT_FS,
    #[cfg(target_os = "freebsd")]
    EVFILT_LIO = libc::EVFILT_LIO,
    #[cfg(target_os = "macos")]
    EVFILT_MACHPORT = libc::EVFILT_MACHPORT,
    EVFILT_PROC = libc::EVFILT_PROC,
    EVFILT_READ = libc::EVFILT_READ,
    EVFILT_SIGNAL = libc::EVFILT_SIGNAL,
    EVFILT_TIMER = libc::EVFILT_TIMER,
    #[cfg(any(target_os = "macos",
              target_os = "dragonfly",
              target_os = "freebsd"))]
    EVFILT_USER = libc::EVFILT_USER,
    #[cfg(target_os = "macos")]
    EVFILT_VM = libc::EVFILT_VM,
    EVFILT_VNODE = libc::EVFILT_VNODE,
    EVFILT_WRITE = libc::EVFILT_WRITE,
}

#[cfg(target_os = "netbsd")]
type type_of_event_filter = i32;
#[cfg(target_os = "netbsd")]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventFilter {
    EVFILT_READ = libc::EVFILT_READ,
    EVFILT_WRITE = libc::EVFILT_WRITE,
    EVFILT_AIO = libc::EVFILT_AIO,
    EVFILT_VNODE = libc::EVFILT_VNODE,
    EVFILT_PROC = libc::EVFILT_PROC,
    EVFILT_SIGNAL = libc::EVFILT_SIGNAL,
    EVFILT_TIMER = libc::EVFILT_TIMER,
}

#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly"))]
pub type type_of_event_flag = u16;
#[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
pub type type_of_event_flag = u32;
libc_bitflags!{
    flags EventFlag: type_of_event_flag {
        EV_ADD,
        EV_CLEAR,
        EV_DELETE,
        EV_DISABLE,
        EV_DISPATCH,
        #[cfg(target_os = "freebsd")]
        EV_DROP,
        EV_ENABLE,
        EV_EOF,
        EV_ERROR,
        #[cfg(target_os = "macos")]
        EV_FLAG0,
        EV_FLAG1,
        #[cfg(target_os = "dragonfly")]
        EV_NODATA,
        EV_ONESHOT,
        #[cfg(target_os = "macos")]
        EV_OOBAND,
        #[cfg(target_os = "macos")]
        EV_POLL,
        #[cfg(not(target_os = "openbsd"))]
        EV_RECEIPT,
        EV_SYSFLAGS,
    }
}

bitflags!(
    flags FilterFlag: u32 {
        #[cfg(target_os = "macos")]
        const NOTE_ABSOLUTE                        = libc::NOTE_ABSOLUTE,
        const NOTE_ATTRIB                          = libc::NOTE_ATTRIB,
        const NOTE_CHILD                           = libc::NOTE_CHILD,
        const NOTE_DELETE                          = libc::NOTE_DELETE,
        #[cfg(target_os = "openbsd")]
        const NOTE_EOF                             = libc::NOTE_EOF,
        const NOTE_EXEC                            = libc::NOTE_EXEC,
        const NOTE_EXIT                            = libc::NOTE_EXIT,
        #[cfg(target_os = "macos")]
        const NOTE_EXIT_REPARENTED                 = libc::NOTE_EXIT_REPARENTED,
        #[cfg(target_os = "macos")]
        const NOTE_EXITSTATUS                      = libc::NOTE_EXITSTATUS,
        const NOTE_EXTEND                          = libc::NOTE_EXTEND,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFAND                           = libc::NOTE_FFAND,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFCOPY                          = libc::NOTE_FFCOPY,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFCTRLMASK                      = libc::NOTE_FFCTRLMASK,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFLAGSMASK                      = libc::NOTE_FFLAGSMASK,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFNOP                           = libc::NOTE_FFNOP,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_FFOR                            = libc::NOTE_FFOR,
        const NOTE_FORK                            = libc::NOTE_FORK,
        const NOTE_LINK                            = libc::NOTE_LINK,
        const NOTE_LOWAT                           = libc::NOTE_LOWAT,
        #[cfg(target_os = "freebsd")]
        const NOTE_MSECONDS                        = libc::NOTE_MSECONDS,
        #[cfg(target_os = "macos")]
        const NOTE_NONE                            = libc::NOTE_NONE,
        #[cfg(any(target_os = "macos", target_os = "freebsd"))]
        const NOTE_NSECONDS                        = libc::NOTE_NSECONDS,
        #[cfg(target_os = "dragonfly")]
        const NOTE_OOB                             = libc::NOTE_OOB,
        const NOTE_PCTRLMASK                       = libc::NOTE_PCTRLMASK,
        const NOTE_PDATAMASK                       = libc::NOTE_PDATAMASK,
        #[cfg(target_os = "macos")]
        const NOTE_REAP                            = libc::NOTE_REAP,
        const NOTE_RENAME                          = libc::NOTE_RENAME,
        const NOTE_REVOKE                          = libc::NOTE_REVOKE,
        #[cfg(any(target_os = "macos", target_os = "freebsd"))]
        const NOTE_SECONDS                         = libc::NOTE_SECONDS,
        #[cfg(target_os = "macos")]
        const NOTE_SIGNAL                          = libc::NOTE_SIGNAL,
        const NOTE_TRACK                           = libc::NOTE_TRACK,
        const NOTE_TRACKERR                        = libc::NOTE_TRACKERR,
        #[cfg(any(target_os = "macos",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        const NOTE_TRIGGER                         = libc::NOTE_TRIGGER,
        #[cfg(target_os = "openbsd")]
        const NOTE_TRUNCATE                        = libc::NOTE_TRUNCATE,
        #[cfg(any(target_os = "macos", target_os = "freebsd"))]
        const NOTE_USECONDS                        = libc::NOTE_USECONDS,
        #[cfg(target_os = "macos")]
        const NOTE_VM_ERROR                        = libc::NOTE_VM_ERROR,
        #[cfg(target_os = "macos")]
        const NOTE_VM_PRESSURE                     = libc::NOTE_VM_PRESSURE,
        #[cfg(target_os = "macos")]
        const NOTE_VM_PRESSURE_SUDDEN_TERMINATE    = libc::NOTE_VM_PRESSURE_SUDDEN_TERMINATE,
        #[cfg(target_os = "macos")]
        const NOTE_VM_PRESSURE_TERMINATE           = libc::NOTE_VM_PRESSURE_TERMINATE,
        const NOTE_WRITE                           = libc::NOTE_WRITE,
    }
);

pub fn kqueue() -> Result<RawFd> {
    let res = unsafe { libc::kqueue() };

    Errno::result(res)
}


/*
 * KEvent can't derive Send because on some operating systems, udata is defined
 * as a void*.  However, KEvent's public API always treats udata as a uintptr_t,
 * which is safe to Send.
 */
unsafe impl Send for KEvent {
}

impl KEvent {
    pub fn new(ident: uintptr_t, filter: EventFilter, flags: EventFlag,
               fflags:FilterFlag, data: intptr_t, udata: uintptr_t) -> KEvent {
        KEvent { kevent: libc::kevent {
            ident: ident,
            filter: filter as type_of_event_filter,
            flags: flags.bits(),
            fflags: fflags.bits(),
            data: data,
            udata: udata as type_of_udata
        } }
    }

    pub fn ident(&self) -> uintptr_t {
        self.kevent.ident
    }

    pub fn filter(&self) -> EventFilter {
        unsafe { mem::transmute(self.kevent.filter as type_of_event_filter) }
    }

    pub fn flags(&self) -> EventFlag {
        EventFlag::from_bits(self.kevent.flags).unwrap()
    }

    pub fn fflags(&self) -> FilterFlag {
        FilterFlag::from_bits(self.kevent.fflags).unwrap()
    }

    pub fn data(&self) -> intptr_t {
        self.kevent.data
    }

    pub fn udata(&self) -> uintptr_t {
        self.kevent.udata as uintptr_t
    }
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

#[cfg(any(target_os = "macos",
          target_os = "freebsd",
          target_os = "dragonfly",
          target_os = "openbsd"))]
type type_of_nchanges = c_int;
#[cfg(target_os = "netbsd")]
type type_of_nchanges = size_t;

pub fn kevent_ts(kq: RawFd,
              changelist: &[KEvent],
              eventlist: &mut [KEvent],
              timeout_opt: Option<timespec>) -> Result<usize> {

    let res = unsafe {
        libc::kevent(
            kq,
            changelist.as_ptr() as *const libc::kevent,
            changelist.len() as type_of_nchanges,
            eventlist.as_mut_ptr() as *mut libc::kevent,
            eventlist.len() as type_of_nchanges,
            if let Some(ref timeout) = timeout_opt {timeout as *const timespec} else {ptr::null()})
    };

    Errno::result(res).map(|r| r as usize)
}

#[inline]
pub fn ev_set(ev: &mut KEvent,
              ident: usize,
              filter: EventFilter,
              flags: EventFlag,
              fflags: FilterFlag,
              udata: uintptr_t) {

    ev.kevent.ident  = ident as uintptr_t;
    ev.kevent.filter = filter as type_of_event_filter;
    ev.kevent.flags  = flags.bits();
    ev.kevent.fflags = fflags.bits();
    ev.kevent.data   = 0;
    ev.kevent.udata  = udata as type_of_udata;
}

#[test]
fn test_struct_kevent() {
    let udata : uintptr_t = 12345;

    let expected = libc::kevent{ident: 0xdeadbeef,
                                filter: libc::EVFILT_READ,
                                flags: libc::EV_DISPATCH | libc::EV_ADD,
                                fflags: libc::NOTE_CHILD | libc::NOTE_EXIT,
                                data: 0x1337,
                                udata: udata as type_of_udata};
    let actual = KEvent::new(0xdeadbeef,
                             EventFilter::EVFILT_READ,
                             EV_DISPATCH | EV_ADD,
                             NOTE_CHILD | NOTE_EXIT,
                             0x1337,
                             udata);
    assert!(expected.ident == actual.ident());
    assert!(expected.filter == actual.filter() as type_of_event_filter);
    assert!(expected.flags == actual.flags().bits());
    assert!(expected.fflags == actual.fflags().bits());
    assert!(expected.data == actual.data());
    assert!(expected.udata == actual.udata() as type_of_udata);
    assert!(mem::size_of::<libc::kevent>() == mem::size_of::<KEvent>());
}
