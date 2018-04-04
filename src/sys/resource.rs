//! Configure the process resource limits.
use std::mem;

use libc::{self, c_int, rlimit, RLIM_INFINITY};
pub use libc::rlim_t;

use {Errno, Result};

libc_enum!{
    #[repr(i32)]
    pub enum Resource {
        // POSIX
        RLIMIT_AS,
        RLIMIT_CORE,
        RLIMIT_CPU,
        RLIMIT_DATA,
        RLIMIT_FSIZE,
        RLIMIT_NOFILE,
        RLIMIT_STACK,

        // BSDs and Linux
        #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux", target_os = "openbsd"))]
        RLIMIT_MEMLOCK,
        #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux", target_os = "openbsd"))]
        RLIMIT_NPROC,
        #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux", target_os = "openbsd"))]
        RLIMIT_RSS,

        // Android and Linux only
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_LOCKS,
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_MSGQUEUE,
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_NICE,
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_RTPRIO,
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_RTTIME,
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_SIGPENDING,

        // Available on some BSD
        #[cfg(target_os = "freebsd")]
        RLIMIT_KQUEUES,
        #[cfg(target_os = "freebsd")]
        RLIMIT_NPTS,
        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        RLIMIT_SBSIZE,
        #[cfg(target_os = "freebsd")]
        RLIMIT_SWAP,
    }
}

/// Get the current processes resource limits
///
/// A value of None indicates that there's no limit.
///
/// # Parameters
///
/// * `resource`: The [`Resource`] that we want to get the limits of.
///
/// # Examples
///
/// ```
/// # use nix::sys::resource::{getrlimit, Resource};
///
/// let (soft_limit, hard_limit) = getrlimit(Resource::RLIMIT_NOFILE).unwrap();
/// println!("current soft_limit: {:?}", soft_limit);
/// println!("current hard_limit: {:?}", hard_limit);
/// ```
///
/// # References
///
/// [getrlimit(2)](https://linux.die.net/man/2/getrlimit)
///
/// [`Resource`]: enum.Resource.html
pub fn getrlimit(resource: Resource) -> Result<(Option<rlim_t>, Option<rlim_t>)> {
    let mut rlim: rlimit = unsafe { mem::uninitialized() };
    let res = unsafe { libc::getrlimit(resource as c_int, &mut rlim as *mut _) };
    // TODO: use Option::filter after it has been stabilized
    Errno::result(res).map(|_| {
        (if rlim.rlim_cur != RLIM_INFINITY { Some(rlim.rlim_cur) } else { None },
         if rlim.rlim_max != RLIM_INFINITY { Some(rlim.rlim_max) } else { None })
    })
}

/// Set the current processes resource limits
///
/// A value of None indicates that there's no limit.
///
/// # Parameters
///
/// * `resource`: The [`Resource`] that we want to set the limits of.
/// * `soft_limit`: The value that the kernel enforces for the corresponding resource.
/// * `hard_limit`: The ceiling for the soft limit. Must be lower or equal to the current hard limit
///   for non-root users.
///
/// # Examples
///
/// ```no_run
/// # use nix::sys::resource::{setrlimit, Resource};
///
/// let soft_limit = Some(1024);
/// let hard_limit = None;
/// setrlimit(Resource::RLIMIT_NOFILE, soft_limit, hard_limit).unwrap();
/// ```
///
/// # References
///
/// [setrlimit(2)](https://linux.die.net/man/2/setrlimit)
///
/// [`Resource`]: enum.Resource.html
pub fn setrlimit(resource: Resource, soft_limit: Option<rlim_t>, hard_limit: Option<rlim_t>) -> Result<()> {
    let mut rlim: rlimit = unsafe { mem::uninitialized() };
    rlim.rlim_cur = soft_limit.unwrap_or(RLIM_INFINITY);
    rlim.rlim_max = hard_limit.unwrap_or(RLIM_INFINITY);

    let res = unsafe { libc::setrlimit(resource as c_int, &rlim as *const _) };
    Errno::result(res).map(|_| ())
}
