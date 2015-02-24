use libc::{
    c_int,
    uint32_t
};
use std::os::unix::Fd;

use errno::Errno;
use fcntl::{O_CLOEXEC, O_NONBLOCK};
use nix::{NixError, NixResult, AsCString};

mod ffi {
    use libc::{
        c_char,
        c_int,
        uint32_t
    };

    extern {
        pub fn inotify_init() -> c_int;
        pub fn inotify_init1(flags: c_int) -> c_int;
        pub fn inotify_add_watch(fd: c_int, path: *const c_char, mask: uint32_t) -> c_int;
        pub fn inotify_rm_watch(fd: c_int, wd: uint32_t) -> c_int;
    }
}

/* the following are legal, implemented events that user-space can watch for */
pub type EventFlags = uint32_t;

pub const IN_ACCESS:        EventFlags = 0x00000001;
pub const IN_MODIFY:        EventFlags = 0x00000002;
pub const IN_ATTRIB:        EventFlags = 0x00000004;
pub const IN_CLOSE_WRITE:   EventFlags = 0x00000008;
pub const IN_CLOSE_NOWRITE: EventFlags = 0x00000010;
pub const IN_OPEN:          EventFlags = 0x00000020;
pub const IN_MOVED_FROM:    EventFlags = 0x00000040;
pub const IN_MOVED_TO:      EventFlags = 0x00000080;
pub const IN_CREATE:        EventFlags = 0x00000100;
pub const IN_DELETE:        EventFlags = 0x00000200;
pub const IN_DELETE_SELF:   EventFlags = 0x00000400;
pub const IN_MOVE_SELF:     EventFlags = 0x00000800;

/* the following are legal events. they are sent as needed to any watch */
pub const IN_UNMOUNT:       EventFlags = 0x00002000;
pub const IN_Q_OVERFLOW:    EventFlags = 0x00004000;
pub const IN_IGNORED:       EventFlags = 0x00008000;

/* special flags */
pub const IN_ONLYDIR:       EventFlags = 0x01000000;
pub const IN_DONT_FOLLOW:   EventFlags = 0x02000000;
pub const IN_EXCL_UNLINK:   EventFlags = 0x04000000;
pub const IN_MASK_ADD:      EventFlags = 0x20000000;
pub const IN_ISDIR:         EventFlags = 0x40000000;
pub const IN_ONESHOT:       EventFlags = 0x80000000;

/* helper events */
pub const IN_CLOSE:         EventFlags = IN_CLOSE_WRITE | IN_CLOSE_NOWRITE;
pub const IN_MOVE:          EventFlags = IN_MOVED_FROM | IN_MOVED_TO;
pub const IN_ALL_EVENTS:    EventFlags =
    IN_ACCESS | IN_MODIFY | IN_ATTRIB | IN_CLOSE_WRITE |
    IN_CLOSE_NOWRITE | IN_OPEN | IN_MOVED_FROM | IN_MOVED_TO |
    IN_DELETE | IN_CREATE | IN_DELETE_SELF | IN_MOVE_SELF;


/* Flags for inotify_init1 */
pub type InotifyInitFlags = c_int;

pub const IN_CLOEXEC: InotifyInitFlags = 0o02000000;  // O_CLOEXEC
pub const IN_NONBLOCK: InotifyInitFlags = 0o00004000; // O_NONBLOCK

/*
#[repr(C)]
pub struct inotify_event {
    pub wd: c_int,
    pub mask: uint32_t,
    pub cookie: uint32_t,
    pub len: uint32_t,
    pub name: [u8] // ? char name[0]
}
*/

#[inline]
pub fn inotify_init1(flags: InotifyInitFlags) -> NixResult<Fd> {
    let res = unsafe { ffi::inotify_init1(flags) };

    if res < 0 {
        return Err(NixError::Sys(Errno::last()));
    }

    Ok(res)
}

#[inline]
pub fn inotify_add_watch<T: AsCString>(fd: Fd, path: T, mask: EventFlags) -> NixResult<Fd> {
    let res = unsafe { ffi::inotify_add_watch(fd, path.as_c_char(), mask) };

    if res < 0 {
        return Err(NixError::Sys(Errno::last()));
    }

    Ok(res)
}

#[inline]
pub fn inotify_rm_watch(fd: Fd, wd: uint32_t) -> NixResult<()> {
    let res = unsafe { ffi::inotify_rm_watch(fd, wd) };

    if res < 0 {
        return Err(NixError::Sys(Errno::last()));
    }

    Ok(())
}
