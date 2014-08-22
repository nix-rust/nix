use libc::{timespec, time_t, c_int, c_long, c_void};
use errno::{SysResult, SysError};
use fcntl::Fd;

pub use self::ffi::kevent as KEvent;

mod ffi {
    pub use libc::{c_int, c_void, uintptr_t, intptr_t, timespec};
    use super::{EventFilter, EventFlag, FilterFlag};

    // Packed to 32 bytes
    pub struct kevent {
        pub ident: uintptr_t,       // 8
        pub filter: EventFilter,    // 2
        pub flags: EventFlag,       // 2
        pub fflags: FilterFlag,     // 4
        pub data: intptr_t,         // 8
        pub udata: *mut c_void      // 8
    }

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
#[deriving(Show, PartialEq)]
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
        static EV_ADD       = 0x0001,
        static EV_DELETE    = 0x0002,
        static EV_ENABLE    = 0x0004,
        static EV_DISABLE   = 0x0008,
        static EV_RECEIPT   = 0x0040,
        static EV_ONESHOT   = 0x0010,
        static EV_CLEAR     = 0x0020,
        static EV_DISPATCH  = 0x0080,
        static EV_SYSFLAGS  = 0xF000,
        static EV_FLAG0     = 0x1000,
        static EV_FLAG1     = 0x2000,
        static EV_EOF       = 0x8000,
        static EV_ERROR     = 0x4000
    }
)

bitflags!(
    flags FilterFlag: u32 {
        static NOTE_TRIGGER                         = 0x01000000,
        static NOTE_FFNOP                           = 0x00000000,
        static NOTE_FFAND                           = 0x40000000,
        static NOTE_FFOR                            = 0x80000000,
        static NOTE_FFCOPY                          = 0xc0000000,
        static NOTE_FFCTRLMASK                      = 0xc0000000,
        static NOTE_FFLAGSMASK                      = 0x00ffffff,
        static NOTE_LOWAT                           = 0x00000001,
        static NOTE_DELETE                          = 0x00000001,
        static NOTE_WRITE                           = 0x00000002,
        static NOTE_EXTEND                          = 0x00000004,
        static NOTE_ATTRIB                          = 0x00000008,
        static NOTE_LINK                            = 0x00000010,
        static NOTE_RENAME                          = 0x00000020,
        static NOTE_REVOKE                          = 0x00000040,
        static NOTE_NONE                            = 0x00000080,
        static NOTE_EXIT                            = 0x80000000,
        static NOTE_FORK                            = 0x40000000,
        static NOTE_EXEC                            = 0x20000000,
        static NOTE_REAP                            = 0x10000000,
        static NOTE_SIGNAL                          = 0x08000000,
        static NOTE_EXITSTATUS                      = 0x04000000,
        static NOTE_RESOURCEEND                     = 0x02000000,
        static NOTE_APPACTIVE                       = 0x00800000,
        static NOTE_APPBACKGROUND                   = 0x00400000,
        static NOTE_APPNONUI                        = 0x00200000,
        static NOTE_APPINACTIVE                     = 0x00100000,
        static NOTE_APPALLSTATES                    = 0x00f00000,
        static NOTE_PDATAMASK                       = 0x000fffff,
        static NOTE_PCTRLMASK                       = 0xfff00000,
        static NOTE_EXIT_REPARENTED                 = 0x00080000,
        static NOTE_VM_PRESSURE                     = 0x80000000,
        static NOTE_VM_PRESSURE_TERMINATE           = 0x40000000,
        static NOTE_VM_PRESSURE_SUDDEN_TERMINATE    = 0x20000000,
        static NOTE_VM_ERROR                        = 0x10000000,
        static NOTE_SECONDS                         = 0x00000001,
        static NOTE_USECONDS                        = 0x00000002,
        static NOTE_NSECONDS                        = 0x00000004,
        static NOTE_ABSOLUTE                        = 0x00000008,
        static NOTE_TRACK                           = 0x00000001,
        static NOTE_TRACKERR                        = 0x00000002,
        static NOTE_CHILD                           = 0x00000004
    }
)

pub static EV_POLL: EventFlag = EV_FLAG0;
pub static EV_OOBAND: EventFlag = EV_FLAG1;

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
              timeout_ms: uint) -> SysResult<uint> {

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

    return Ok(res as uint)
}

/*
    // Packed to 32 bytes
    pub struct kevent {
        pub ident: uintptr_t,   // 8
        pub filter: i16,        // 2
        pub flags: u16,         // 2
        pub fflags: u32,        // 4
        pub data: intptr_t,     // 8
        pub udata: *mut c_void  // 8
    }
 */

#[inline]
pub fn ev_set(ev: &mut KEvent,
              ident: uint,
              filter: EventFilter,
              flags: EventFlag,
              fflags: FilterFlag,
              udata: *mut c_void) {

    ev.ident  = ident;
    ev.filter = filter;
    ev.flags  = flags;
    ev.fflags = fflags;
    ev.data   = 0;
    ev.udata  = udata;
}
