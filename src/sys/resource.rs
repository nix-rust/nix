//! Configure the process resource limits.
use std::mem;

use libc::{self, c_int, rlimit, RLIM_INFINITY};
pub use libc::rlim_t;

use {Errno, Result};

libc_enum!{
    /// A resource that limits apply to
    #[repr(i32)]
    pub enum Resource {
        // POSIX
        /// This is the maximum size of the process's virtual memory (address space). The limit is specified in bytes, and is rounded down to the system page size.
        RLIMIT_AS,
        /// This is the maximum size of a core file (see core(5)) in bytes that the process may dump.
        RLIMIT_CORE,
        /// This is a limit, in seconds, on the amount of CPU time that the process can consume.
        RLIMIT_CPU,
        /// This is the maximum size of the process's data segment (initialized data, uninitialized data, and heap). The limit is specified in bytes, and is rounded down to the system page size.
        RLIMIT_DATA,
        /// This is the maximum size in bytes of files that the process may create. Attempts to extend a file beyond this limit result in delivery of a SIGXFSZ signal.
        RLIMIT_FSIZE,
        /// This specifies a value one greater than the maximum file descriptor number that can be opened by this process.
        RLIMIT_NOFILE,
        /// This is the maximum size of the process stack, in bytes. Upon reaching this limit, a SIGSEGV signal is generated.
        RLIMIT_STACK,

        // BSDs and Linux
        /// This is the maximum number of bytes of memory that may be locked into RAM. This limit is in effect rounded down to the nearest multiple of the system page size.
        #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux", target_os = "openbsd"))]
        RLIMIT_MEMLOCK,
        /// This is a limit on the number of extant process (or, more precisely on Linux, threads) for the real user ID of the calling process.
        #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux", target_os = "openbsd"))]
        RLIMIT_NPROC,
        /// This is a limit (in bytes) on the process's resident set (the number of virtual pages resident in RAM).
        #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux", target_os = "openbsd"))]
        RLIMIT_RSS,

        // Android and Linux only
        /// This is a limit on the combined number of flock(2) locks and fcntl(2) leases that this process may establish.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_LOCKS,
        /// This is a limit on the number of bytes that can be allocated for POSIX message queues for the real user ID of the calling process.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_MSGQUEUE,
        /// This specifies a ceiling to which the process's nice value can be raised using setpriority(2) or nice(2). The actual ceiling for the nice value is calculated as 20 - rlim_cur.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_NICE,
        /// This specifies a ceiling on the real-time priority that may be set for this process using sched_setscheduler(2) and sched_setparam(2).
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_RTPRIO,
        /// This is a limit (in microseconds) on the amount of CPU time that a process scheduled under a real-time scheduling policy may consume without making a blocking system call.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_RTTIME,
        /// This is a limit on the number of signals that may be queued for the real user ID of the calling process. Both standard and real-time signals are counted for the purpose of checking this limit.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        RLIMIT_SIGPENDING,

        // Available on some BSD
        /// The maximum number of kqueues this user id is allowed to create.
        #[cfg(target_os = "freebsd")]
        RLIMIT_KQUEUES,
        /// The maximum number of pseudo-terminals this user id is allowed to create.
        #[cfg(target_os = "freebsd")]
        RLIMIT_NPTS,
        /// The maximum size (in bytes) of socket buffer usage for this user.
        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        RLIMIT_SBSIZE,
        /// The maximum size (in bytes) of the swap space that may be reserved or used by all of this user id's processes.
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
    // TODO: How do we handle the case where soft_limit isn't Some()?
    rlim.rlim_cur = soft_limit.unwrap_or(RLIM_INFINITY);
    rlim.rlim_max = hard_limit.unwrap_or(RLIM_INFINITY);

    let res = unsafe { libc::setrlimit(resource as c_int, &rlim as *const _) };
    Errno::result(res).map(|_| ())
}
