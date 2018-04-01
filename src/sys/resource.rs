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
        #[cfg(all(unix, not(target_os = "solaris")))]
        RLIMIT_MEMLOCK,
        #[cfg(all(unix, not(target_os = "solaris")))]
        RLIMIT_NPROC,
        #[cfg(all(unix, not(target_os = "solaris")))]
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

        // Non-Linux
        #[cfg(target_os = "freebsd")]
        RLIMIT_KQUEUES,
        #[cfg(target_os = "freebsd")]
        RLIMIT_NPTS,
        #[cfg(target_os = "freebsd")]
        RLIMIT_SBSIZE,
        #[cfg(target_os = "freebsd")]
        RLIMIT_SWAP,
    }
}

pub fn getrlimit(resource: Resource) -> Result<(Option<rlim_t>, Option<rlim_t>)> {
    let mut rlim: rlimit = unsafe { mem::uninitialized() };
    let res = unsafe { libc::getrlimit(resource as c_int, &mut rlim as *mut _) };
    Errno::result(res).map(|_| {
        (if rlim.rlim_cur != RLIM_INFINITY { Some(rlim.rlim_cur) } else { None },
         if rlim.rlim_max != RLIM_INFINITY { Some(rlim.rlim_max) } else { None })
    })
}

pub fn setrlimit(resource: Resource, soft_limit: Option<rlim_t>, hard_limit: Option<rlim_t>) -> Result<()> {
    let mut rlim: rlimit = unsafe { mem::uninitialized() };
    rlim.rlim_cur = soft_limit.unwrap_or(RLIM_INFINITY);
    rlim.rlim_max = hard_limit.unwrap_or(RLIM_INFINITY);

    let res = unsafe { libc::setrlimit(resource as c_int, &rlim as *const _) };
    Errno::result(res).map(drop)
}
