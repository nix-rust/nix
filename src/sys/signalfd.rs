use libc;
use fcntl::{Fd, SfdFlag, F_SETFD, F_SETFL, FD_CLOEXEC, O_NONBLOCK, SFD_CLOEXEC, SFD_NONBLOCK, fcntl};
use errno::{SysResult, SysError};
use sys::signal;
use sys::signal::SigSet;
use features;

mod ffi {
    extern {
        #[linkage = "extern_weak"]
        pub static signalfd: *const ();
    }
}

#[allow(dead_code)]
#[repr(C)]
pub struct SigInfo {
    signo:      u32,
    errno:      i32,    // unused
    code:       i32,
    pid:        u32,
    uid:        u32,
    fd:         i32,    // SIGIO
    tid:        u32,    // timer ID (POSIX timers)
    band:       u32,    // SIGIO
    overrun:    u32,    // overrun count (POSIX timers)
    trapno:     u32,    // trap number that caused signal
    status:     i32,    // exit status or signal (SIGCHLD)
    integer:    i32,    // integer sent by sigqueue
    ptr:        u64,    // pointer sent by sigqueue
    utime:      u64,    // user CPU time consumed (SIGCHLD)
    stime:      u64,    // system CPU time consumed (SIGCHLD)
    addr:       u64,    // address that generated hardward-generated signals
    _pad:       [u8, ..6]
}

type SignalfdFunc = unsafe extern fn(libc::c_int, *const signal::sigset_t, libc::c_int) -> libc::c_int;

#[inline]
fn sys_signalfd() -> SignalfdFunc {
    use std::mem;

    if ffi::signalfd.is_null() {
        unimplemented!()
    }

    unsafe { mem::transmute::<*const (), SignalfdFunc>(ffi::signalfd) }
}

pub fn signalfd(mask: SigSet, flags: SfdFlag) -> SysResult<Fd> {
    let res = if features::socket_atomic_cloexec() {
        unsafe { sys_signalfd()(-1, mask.inner() as *const signal::sigset_t, flags.bits()) }
    } else {
        unsafe { sys_signalfd()(-1, mask.inner() as *const signal::sigset_t, 0i32) }
    };

    if res < 0 {
        return Err(SysError::last());
    }

    if !features::socket_atomic_cloexec() {
        if flags.bits() & SFD_CLOEXEC.bits() != 0 {
            try!(fcntl(res, F_SETFD(FD_CLOEXEC)));
        }

        if flags.bits() & SFD_NONBLOCK.bits() != 0 {
            try!(fcntl(res, F_SETFL(O_NONBLOCK)));
        }
    }

    Ok(res)
}

pub fn update_signalfd(fd: Fd, mask: SigSet) -> SysResult<()> {
    let res = unsafe { sys_signalfd()(fd, mask.inner() as *const signal::sigset_t, 0) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(())
}
