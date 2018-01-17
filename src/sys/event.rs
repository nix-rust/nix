/* TOOD: Implement for other kqueue based systems
 */

use std::mem;
use std::os::unix::io::RawFd;
use std::ptr;

#[cfg(not(target_os = "netbsd"))]
use libc::{timespec, c_int, intptr_t, uintptr_t};
#[cfg(target_os = "netbsd")]
use libc::{timespec, time_t, c_long, intptr_t, uintptr_t, size_t};
use libc;

use {Errno, Result};
use sys::time::TimeSpec;

// Redefine kevent in terms of programmer-friendly enums and bitfields.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct KEvent {
    kevent: libc::kevent,
}

#[cfg(any(target_os = "dragonfly", target_os = "freebsd",
          target_os = "ios", target_os = "macos",
          target_os = "openbsd"))]
type type_of_udata = *mut libc::c_void;
#[cfg(any(target_os = "dragonfly", target_os = "freebsd",
          target_os = "ios", target_os = "macos"))]
type type_of_data = libc::intptr_t;
#[cfg(any(target_os = "netbsd"))]
type type_of_udata = intptr_t;
#[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
type type_of_data = libc::int64_t;

#[cfg(target_os = "netbsd")]
type type_of_event_filter = u32;
#[cfg(not(target_os = "netbsd"))]
type type_of_event_filter = i16;
libc_enum! {
    #[cfg_attr(target_os = "netbsd", repr(u32))]
    #[cfg_attr(not(target_os = "netbsd"), repr(i16))]
    pub enum EventFilter {
        EVFILT_AIO,
        #[cfg(target_os = "dragonfly")]
        EVFILT_EXCEPT,
        #[cfg(any(target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "ios",
                  target_os = "macos"))]
        EVFILT_FS,
        #[cfg(target_os = "freebsd")]
        EVFILT_LIO,
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        EVFILT_MACHPORT,
        EVFILT_PROC,
        EVFILT_READ,
        EVFILT_SIGNAL,
        EVFILT_TIMER,
        #[cfg(any(target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "ios",
                  target_os = "macos"))]
        EVFILT_USER,
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        EVFILT_VM,
        EVFILT_VNODE,
        EVFILT_WRITE,
    }
}

#[cfg(any(target_os = "dragonfly", target_os = "freebsd",
          target_os = "ios", target_os = "macos",
          target_os = "openbsd"))]
pub type type_of_event_flag = u16;
#[cfg(any(target_os = "netbsd"))]
pub type type_of_event_flag = u32;
libc_bitflags!{
    pub struct EventFlag: type_of_event_flag {
        EV_ADD;
        EV_CLEAR;
        EV_DELETE;
        EV_DISABLE;
        // No released version of OpenBSD supports EV_DISPATCH or EV_RECEIPT.
        // These have been commited to the -current branch though and are
        // expected to be part of the OpenBSD 6.2 release in Nov 2017.
        // See: https://marc.info/?l=openbsd-tech&m=149621427511219&w=2
        // https://github.com/rust-lang/libc/pull/613
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd",
                  target_os = "ios", target_os = "macos",
                  target_os = "netbsd"))]
        EV_DISPATCH;
        #[cfg(target_os = "freebsd")]
        EV_DROP;
        EV_ENABLE;
        EV_EOF;
        EV_ERROR;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EV_FLAG0;
        EV_FLAG1;
        #[cfg(target_os = "dragonfly")]
        EV_NODATA;
        EV_ONESHOT;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EV_OOBAND;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EV_POLL;
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd",
                  target_os = "ios", target_os = "macos",
                  target_os = "netbsd"))]
        EV_RECEIPT;
        EV_SYSFLAGS;
    }
}

libc_bitflags!(
    pub struct FilterFlag: u32 {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_ABSOLUTE;
        NOTE_ATTRIB;
        NOTE_CHILD;
        NOTE_DELETE;
        #[cfg(target_os = "openbsd")]
        NOTE_EOF;
        NOTE_EXEC;
        NOTE_EXIT;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_EXIT_REPARENTED;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_EXITSTATUS;
        NOTE_EXTEND;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFAND;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFCOPY;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFCTRLMASK;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFLAGSMASK;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFNOP;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_FFOR;
        NOTE_FORK;
        NOTE_LINK;
        NOTE_LOWAT;
        #[cfg(target_os = "freebsd")]
        NOTE_MSECONDS;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_NONE;
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
        NOTE_NSECONDS;
        #[cfg(target_os = "dragonfly")]
        NOTE_OOB;
        NOTE_PCTRLMASK;
        NOTE_PDATAMASK;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_REAP;
        NOTE_RENAME;
        NOTE_REVOKE;
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
        NOTE_SECONDS;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_SIGNAL;
        NOTE_TRACK;
        NOTE_TRACKERR;
        #[cfg(any(target_os = "macos",
                  target_os = "ios",
                  target_os = "freebsd",
                  target_os = "dragonfly"))]
        NOTE_TRIGGER;
        #[cfg(target_os = "openbsd")]
        NOTE_TRUNCATE;
        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
        NOTE_USECONDS;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_VM_ERROR;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_VM_PRESSURE;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_VM_PRESSURE_SUDDEN_TERMINATE;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        NOTE_VM_PRESSURE_TERMINATE;
        NOTE_WRITE;
    }
);

/// The `kqueue` system call creates a new kernel event queue and returns a
/// descriptor.
///
/// For more information see [kqueue(2)].
///
/// [kqueue(2)]: https://www.freebsd.org/cgi/man.cgi?query=kqueue
pub fn kqueue() -> Result<RawFd> {
    let res = unsafe { libc::kqueue() };
    Errno::result(res)
}

// KEvent can't derive Send because on some operating systems, udata is defined
// as a void*.  However, KEvent's public API always treats udata as an intptr_t,
// which is safe to Send.
unsafe impl Send for KEvent {
}

impl KEvent {
    pub fn new(ident: uintptr_t, filter: EventFilter, flags: EventFlag,
               fflags:FilterFlag, data: intptr_t, udata: intptr_t) -> KEvent {
        KEvent { kevent: libc::kevent {
            ident: ident,
            filter: filter as type_of_event_filter,
            flags: flags.bits(),
            fflags: fflags.bits(),
            data: data as type_of_data,
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
        self.kevent.data as intptr_t
    }

    pub fn udata(&self) -> intptr_t {
        self.kevent.udata as intptr_t
    }
}

#[cfg(any(target_os = "macos",
          target_os = "ios",
          target_os = "freebsd",
          target_os = "dragonfly",
          target_os = "openbsd"))]
type type_of_nchanges = c_int;
#[cfg(target_os = "netbsd")]
type type_of_nchanges = size_t;

/// The `kevent` system call is used to register events with the queue, and
/// return any pending events to the user.
///
/// For more information see [kqueue(2)].
///
/// [kqueue(2)]: https://www.freebsd.org/cgi/man.cgi?query=kqueue
///
/// # Examples
///
/// Using `std::time::Duration`.
///
/// ```
/// use std::time::Duration;
/// use nix::sys::event::{kqueue, kevent};
///
/// let kq = kqueue().unwrap();
/// let mut events = Vec::new();
///
/// // With a timeout.
/// let timeout = Duration::from_millis(100);
/// kevent(kq, &[], &mut events, Some(timeout)).unwrap();
///
/// // Without a timeout.
/// kevent::<Duration>(kq, &[], &mut events, None).unwrap();
/// ```
///
/// Using `libc::timespec` directly.
///
/// ```
/// use nix::libc::timespec;
/// use nix::sys::event::{kqueue, kevent};
///
/// let kq = kqueue().unwrap();
/// let mut events = Vec::new();
///
/// let timeout = timespec { tv_sec: 0, tv_nsec: 1000};
/// assert_eq!(kevent(kq, &[], &mut events, Some(timeout)).unwrap(), 0);
/// ```
pub fn kevent<T: Into<TimeSpec>>(kq: RawFd, changelist: &[KEvent], eventlist: &mut [KEvent], timeout: Option<T>) -> Result<usize> {
    let timeout = timeout.map(|t| t.into());
    let timeout_ptr = if let Some(ref timeout) = timeout {
        timeout.as_ref() as *const timespec
    } else {
        ptr::null()
    };

    let res = unsafe {
        libc::kevent(
            kq,
            changelist.as_ptr() as *const libc::kevent,
            changelist.len() as type_of_nchanges,
            eventlist.as_mut_ptr() as *mut libc::kevent,
            eventlist.len() as type_of_nchanges,
            timeout_ptr,
        )
    };

    Errno::result(res).map(|r| r as usize)
}

/// Initialize a kevent structure.
///
/// This API matches the `EV_SET` macro, a better way to create a `KEvent` is to
/// use [`KEvent.new`].
///
/// For more information see [kqueue(2)].
///
/// [`KEvent.new`]: struct.KEvent.html
/// [kqueue(2)]: https://www.freebsd.org/cgi/man.cgi?query=kqueue
#[inline]
pub fn ev_set(ev: &mut KEvent, ident: uintptr_t, filter: EventFilter, flags: EventFlag, fflags: FilterFlag, udata: intptr_t) {
    ev.kevent.ident  = ident as uintptr_t;
    ev.kevent.filter = filter as type_of_event_filter;
    ev.kevent.flags  = flags.bits();
    ev.kevent.fflags = fflags.bits();
    ev.kevent.data   = 0;
    ev.kevent.udata  = udata as type_of_udata;
}

#[test]
fn test_struct_kevent() {
    let udata : intptr_t = 12345;

    let expected = libc::kevent{ident: 0xdead_beef,
                                filter: libc::EVFILT_READ,
                                flags: libc::EV_ONESHOT | libc::EV_ADD,
                                fflags: libc::NOTE_CHILD | libc::NOTE_EXIT,
                                data: 0x1337,
                                udata: udata as type_of_udata};
    let actual = KEvent::new(0xdead_beef,
                             EventFilter::EVFILT_READ,
                             EventFlag::EV_ONESHOT | EventFlag::EV_ADD,
                             FilterFlag::NOTE_CHILD | FilterFlag::NOTE_EXIT,
                             0x1337,
                             udata);
    assert!(expected.ident == actual.ident());
    assert!(expected.filter == actual.filter() as type_of_event_filter);
    assert!(expected.flags == actual.flags().bits());
    assert!(expected.fflags == actual.fflags().bits());
    assert!(expected.data == actual.data() as type_of_data);
    assert!(expected.udata == actual.udata() as type_of_udata);
    assert!(mem::size_of::<libc::kevent>() == mem::size_of::<KEvent>());
}
