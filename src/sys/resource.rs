use std::mem;

use libc::{self, c_int};
pub use libc::{rlimit, rlim_t, RLIM_INFINITY};

#[cfg(any(target_os = "linux",
          target_os = "openbsd",
          target_os = "netbsd",
          target_os = "bitrig"))]
pub use libc::{RLIM_SAVED_CUR, RLIM_SAVED_MAX};

use {Errno, Result};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum Resource {
    // POSIX
    RLIMIT_CORE = libc::RLIMIT_CORE,
    RLIMIT_CPU = libc::RLIMIT_CPU,
    RLIMIT_DATA = libc::RLIMIT_DATA,
    RLIMIT_FSIZE = libc::RLIMIT_FSIZE,
    RLIMIT_NOFILE = libc::RLIMIT_NOFILE,
    RLIMIT_STACK = libc::RLIMIT_STACK,
    RLIMIT_AS = libc::RLIMIT_AS,
    // BSDs and Linux
    #[cfg(all(unix, not(target_os = "solaris")))]
    RLIMIT_MEMLOCK = libc::RLIMIT_MEMLOCK,
    #[cfg(all(unix, not(target_os = "solaris")))]
    RLIMIT_NPROC = libc::RLIMIT_NPROC,
    #[cfg(all(unix, not(target_os = "solaris")))]
    RLIMIT_RSS = libc::RLIMIT_RSS,
    // Linux-only
    #[cfg(any(target_os = "linux", target_os = "android"))]
    RLIMIT_LOCKS = libc::RLIMIT_LOCKS,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    RLIMIT_MSGQUEUE = libc::RLIMIT_MSGQUEUE,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    RLIMIT_NICE = libc::RLIMIT_NICE,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    RLIMIT_RTPRIO = libc::RLIMIT_RTPRIO,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    RLIMIT_RTTIME = libc::RLIMIT_RTTIME,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    RLIMIT_SIGPENDING = libc::RLIMIT_SIGPENDING,
}

#[inline]
fn rlim_to_option(rlim: rlim_t) -> Option<rlim_t> {
    match rlim {
        RLIM_INFINITY => None,
        rlim => Some(rlim),
    }
}

pub fn getrlimit(resource: Resource) -> Result<(Option<rlim_t>, Option<rlim_t>)> {
    let mut rlim: rlimit = unsafe { mem::uninitialized() };
    let res = unsafe { libc::getrlimit(resource as c_int, &mut rlim as *mut _) };
    Errno::result(res)?;
    Ok((rlim_to_option(rlim.rlim_cur), rlim_to_option(rlim.rlim_max)))
}

pub fn setrlimit(resource: Resource, limit: (Option<rlim_t>, Option<rlim_t>)) -> Result<()> {
    let mut rlim: rlimit = unsafe { mem::uninitialized() };
    rlim.rlim_cur = limit.0.unwrap_or(RLIM_INFINITY);
    rlim.rlim_max = limit.1.unwrap_or(RLIM_INFINITY);

    let res = unsafe { libc::setrlimit(resource as c_int, &rlim as *const _) };
    Errno::result(res).map(drop)
}
