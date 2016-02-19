use Result;
use errno::Errno;
use libc::{CLOCK_REALTIME, CLOCK_MONOTONIC, c_int};
use std::os::unix::io::RawFd;
use std::ptr::null_mut;
use std::mem::uninitialized;

pub use libc::timespec;

pub enum ClockId {
    Realtime = CLOCK_REALTIME as isize,
    Monotonic = CLOCK_MONOTONIC as isize,
}

bitflags! {
    flags TfFlag: c_int {
        const TFD_NONBLOCK = 0o00004000,
        const TFD_CLOEXEC = 0o02000000,
    }
}

bitflags! {
    flags TfTimerFlag: c_int {
        const TFD_TIMER_ABSTIME = 1,
    }
}

mod ffi {
    use libc::{c_int, timespec};

    #[repr(C)]
    #[derive(Clone)]
    pub struct ITimerSpec {
        pub it_interval: timespec,
        pub it_value: timespec,
    }

    extern {
        
        pub fn timerfd_create(clockid: c_int, flags: c_int) -> c_int;

        pub fn timerfd_settime(fd: c_int, flags: c_int, new_value: *const ITimerSpec, old_value: *mut ITimerSpec) -> c_int;

        pub fn timerfd_gettime(fd: c_int, curr_value: *mut ITimerSpec) -> c_int;
    }
}

pub use self::ffi::ITimerSpec;

pub fn timerfd_create(clockid: ClockId, flags: TfFlag) -> Result<RawFd> {
    let fd = unsafe { ffi::timerfd_create(clockid as c_int, flags.bits() as c_int) };

    Errno::result(fd)
}

pub fn timerfd_settime(fd: RawFd, flags: TfTimerFlag, new_value: &ITimerSpec, old_value: Option<&mut ITimerSpec>) -> Result<()> {
    let res = unsafe { ffi::timerfd_settime(fd as c_int,
                                            flags.bits() as c_int,
                                            new_value as *const ITimerSpec,
                                            old_value.map(|x| x as *mut ITimerSpec).unwrap_or(null_mut())
                                            ) };

    Errno::result(res).map(drop)
}

pub fn timerfd_gettime(fd: RawFd) -> Result<ITimerSpec> {
    let mut spec = unsafe { uninitialized() };
    let res = unsafe { ffi::timerfd_gettime(fd as c_int, &mut spec as *mut ITimerSpec) };

    Errno::result(res).map(|_| spec)
}
