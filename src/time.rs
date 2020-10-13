use bitflags::bitflags;
use crate::sys::time::{TimeSpec, TimeValLike};
#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "linux",
    target_os = "android",
    target_os = "emscripten",
))]
use crate::{unistd::Pid, Error};
use crate::{Errno, Result};
use libc::{self, clockid_t};
use std::mem::MaybeUninit;

/// Clock identifier
///
/// Newtype pattern around `clockid_t` (which is just alias). It pervents bugs caused by
/// accidentally passing wrong value.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ClockId(clockid_t);

impl ClockId {
    /// Creates `ClockId` from raw `clockid_t`
    pub fn from_raw(clk_id: clockid_t) -> Self {
        ClockId(clk_id)
    }

    /// Returns `ClockId` of a `pid` CPU-time clock
    #[cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "linux",
        target_os = "android",
        target_os = "emscripten",
    ))]
    pub fn pid_cpu_clock_id(pid: Pid) -> Result<Self> {
        clock_getcpuclockid(pid)
    }

    /// Returns resolution of the clock id
    pub fn res(self) -> Result<TimeSpec> {
        clock_getres(self)
    }

    /// Returns the current time on the clock id
    pub fn now(self) -> Result<TimeSpec> {
        clock_gettime(self)
    }

    /// Sets time to `timespec` on the clock id
    #[cfg(not(any(
        target_os = "macos",
        target_os = "ios",
        all(
            not(any(target_env = "uclibc", target_env = "newlibc")),
            any(target_os = "redox", target_os = "hermit",),
        ),
    )))]
    pub fn set_time(self, timespec: TimeSpec) -> Result<()> {
        clock_settime(self, timespec)
    }

    /// Gets the raw `clockid_t` wrapped by `self`
    pub fn as_raw(self) -> clockid_t {
        self.0
    }

    #[cfg(any(
        target_os = "fuchsia",
        all(
            not(any(target_env = "uclibc", target_env = "newlib")),
            any(target_os = "linux", target_os = "android", target_os = "emscripten"),
        )
    ))]
    pub const CLOCK_BOOTTIME: ClockId = ClockId(libc::CLOCK_BOOTTIME);
    #[cfg(any(
        target_os = "fuchsia",
        all(
            not(any(target_env = "uclibc", target_env = "newlib")),
            any(target_os = "linux", target_os = "android", target_os = "emscripten")
        )
    ))]
    pub const CLOCK_BOOTTIME_ALARM: ClockId = ClockId(libc::CLOCK_BOOTTIME_ALARM);
    pub const CLOCK_MONOTONIC: ClockId = ClockId(libc::CLOCK_MONOTONIC);
    #[cfg(any(
        target_os = "fuchsia",
        all(
            not(any(target_env = "uclibc", target_env = "newlib")),
            any(target_os = "linux", target_os = "android", target_os = "emscripten")
        )
    ))]
    pub const CLOCK_MONOTONIC_COARSE: ClockId = ClockId(libc::CLOCK_MONOTONIC_COARSE);
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    pub const CLOCK_MONOTONIC_FAST: ClockId = ClockId(libc::CLOCK_MONOTONIC_FAST);
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    pub const CLOCK_MONOTONIC_PRECISE: ClockId = ClockId(libc::CLOCK_MONOTONIC_PRECISE);
    #[cfg(any(
        target_os = "fuchsia",
        all(
            not(any(target_env = "uclibc", target_env = "newlib")),
            any(target_os = "linux", target_os = "android", target_os = "emscripten")
        )
    ))]
    pub const CLOCK_MONOTONIC_RAW: ClockId = ClockId(libc::CLOCK_MONOTONIC_RAW);
    #[cfg(any(
        target_os = "fuchsia",
        target_env = "uclibc",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "dragonfly",
        all(
            not(target_env = "newlib"),
            any(target_os = "linux", target_os = "android", target_os = "emscripten")
        )
    ))]
    pub const CLOCK_PROCESS_CPUTIME_ID: ClockId = ClockId(libc::CLOCK_PROCESS_CPUTIME_ID);
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    pub const CLOCK_PROF: ClockId = ClockId(libc::CLOCK_PROF);
    pub const CLOCK_REALTIME: ClockId = ClockId(libc::CLOCK_REALTIME);
    #[cfg(any(
        target_os = "fuchsia",
        all(
            not(any(target_env = "uclibc", target_env = "newlib")),
            any(target_os = "linux", target_os = "android", target_os = "emscripten")
        )
    ))]
    pub const CLOCK_REALTIME_ALARM: ClockId = ClockId(libc::CLOCK_REALTIME_ALARM);
    #[cfg(any(
        target_os = "fuchsia",
        all(
            not(any(target_env = "uclibc", target_env = "newlib")),
            any(target_os = "linux", target_os = "android", target_os = "emscripten")
        )
    ))]
    pub const CLOCK_REALTIME_COARSE: ClockId = ClockId(libc::CLOCK_REALTIME_COARSE);
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    pub const CLOCK_REALTIME_FAST: ClockId = ClockId(libc::CLOCK_REALTIME_FAST);
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    pub const CLOCK_REALTIME_PRECISE: ClockId = ClockId(libc::CLOCK_REALTIME_PRECISE);
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    pub const CLOCK_SECOND: ClockId = ClockId(libc::CLOCK_SECOND);
    #[cfg(any(
        target_os = "fuchsia",
        all(
            not(any(target_env = "uclibc", target_env = "newlib")),
            any(
                target_os = "emscripten",
                all(target_os = "linux", target_env = "musl")
            )
        )
    ))]
    pub const CLOCK_SGI_CYCLE: ClockId = ClockId(libc::CLOCK_SGI_CYCLE);
    #[cfg(any(
        target_os = "fuchsia",
        all(
            not(any(target_env = "uclibc", target_env = "newlib")),
            any(
                target_os = "emscripten",
                all(target_os = "linux", target_env = "musl")
            )
        )
    ))]
    pub const CLOCK_TAI: ClockId = ClockId(libc::CLOCK_TAI);
    #[cfg(any(
        target_env = "uclibc",
        target_os = "fuchsia",
        target_os = "ios",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "dragonfly",
        all(
            not(target_env = "newlib"),
            any(target_os = "linux", target_os = "android", target_os = "emscripten",),
        ),
    ))]
    pub const CLOCK_THREAD_CPUTIME_ID: ClockId = ClockId(libc::CLOCK_THREAD_CPUTIME_ID);
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    pub const CLOCK_UPTIME: ClockId = ClockId(libc::CLOCK_UPTIME);
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    pub const CLOCK_UPTIME_FAST: ClockId = ClockId(libc::CLOCK_UPTIME_FAST);
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    pub const CLOCK_UPTIME_PRECISE: ClockId = ClockId(libc::CLOCK_UPTIME_PRECISE);
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    pub const CLOCK_VIRTUAL: ClockId = ClockId(libc::CLOCK_VIRTUAL);
}

impl Into<clockid_t> for ClockId {
    fn into(self) -> clockid_t {
        self.as_raw()
    }
}

impl From<clockid_t> for ClockId {
    fn from(clk_id: clockid_t) -> Self {
        ClockId::from_raw(clk_id)
    }
}

impl std::fmt::Display for ClockId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

/// Get the resolution of the specified clock, (see
/// [clock_getres(2)](https://pubs.opengroup.org/onlinepubs/7908799/xsh/clock_getres.html)).
pub fn clock_getres(clock_id: ClockId) -> Result<TimeSpec> {
    let mut c_time: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
    let ret = unsafe { libc::clock_getres(clock_id.as_raw(), c_time.as_mut_ptr()) };
    Errno::result(ret)?;
    let res = unsafe { c_time.assume_init() };
    Ok(TimeSpec::from(res))
}

/// Get the time of the specified clock, (see
/// [clock_gettime(2)](https://pubs.opengroup.org/onlinepubs/7908799/xsh/clock_gettime.html)).
pub fn clock_gettime(clock_id: ClockId) -> Result<TimeSpec> {
    let mut c_time: MaybeUninit<libc::timespec> = MaybeUninit::uninit();
    let ret = unsafe { libc::clock_gettime(clock_id.as_raw(), c_time.as_mut_ptr()) };
    Errno::result(ret)?;
    let res = unsafe { c_time.assume_init() };
    Ok(TimeSpec::from(res))
}

/// Set the time of the specified clock, (see
/// [clock_settime(2)](https://pubs.opengroup.org/onlinepubs/7908799/xsh/clock_settime.html)).
#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    all(
        not(any(target_env = "uclibc", target_env = "newlibc")),
        any(target_os = "redox", target_os = "hermit",),
    ),
)))]
pub fn clock_settime(clock_id: ClockId, timespec: TimeSpec) -> Result<()> {
    let ret = unsafe { libc::clock_settime(clock_id.as_raw(), timespec.as_ref()) };
    Errno::result(ret).map(drop)
}

/// Get the clock id of the specified process id, (see
/// [clock_getcpuclockid(3)](https://pubs.opengroup.org/onlinepubs/009695399/functions/clock_getcpuclockid.html)).
#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "linux",
    target_os = "android",
    target_os = "emscripten",
))]
pub fn clock_getcpuclockid(pid: Pid) -> Result<ClockId> {
    let mut clk_id: MaybeUninit<libc::clockid_t> = MaybeUninit::uninit();
    let ret = unsafe { libc::clock_getcpuclockid(pid.into(), clk_id.as_mut_ptr()) };
    if ret == 0 {
        let res = unsafe { clk_id.assume_init() };
        Ok(ClockId::from(res))
    } else {
        Err(Error::Sys(Errno::from_i32(ret)))
    }
}

bitflags! {
    /// Flags that are used for arming the timer.
    pub struct ClockNanosleepFlags: libc::c_int {
        const TIMER_ABSTIME = libc::TIMER_ABSTIME;
    }
}

/// Suspend execution of this thread for the amount of time specified by rqtp
/// and measured against the clock speficied by ClockId. If flags is
/// TIMER_ABSTIME, this function will suspend execution until the time value of
/// clock_id reaches the absolute time specified by rqtp. If a signal is caught
/// by a signal-catching function, or a signal causes the process to terminate,
/// this sleep is interrrupted.
/// see also [man 3 clock_nanosleep](https://pubs.opengroup.org/onlinepubs/009695399/functions/clock_nanosleep.html)
#[cfg(any(
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "linux",
    target_os = "android",
    target_os = "solaris",
    target_os = "illumos",
))]
pub fn clock_nanosleep(
    clock_id: ClockId,
    flags: ClockNanosleepFlags,
    rqtp: &TimeSpec,
) -> Result<TimeSpec> {
    let mut rmtp: TimeSpec = TimeSpec::nanoseconds(0);
    let ret = unsafe {
        libc::clock_nanosleep(
            clock_id.as_raw(),
            flags.bits(),
            rqtp.as_ref() as *const _,
            rmtp.as_mut() as *mut _,
        )
    };
    if ret == 0 {
        Ok(rmtp)
    } else {
        Err(Error::Sys(Errno::from_i32(ret)))
    }
}
