//! Low level threading primitives

#[cfg(not(target_os = "redox"))]
use crate::errno::Errno;
#[cfg(not(target_os = "redox"))]
use crate::Result;
use libc::{self, pthread_t};

/// Identifies an individual thread.
pub type Pthread = pthread_t;

/// Obtain ID of the calling thread (see
/// [`pthread_self(3)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/pthread_self.html)
///
/// The thread ID returned by `pthread_self()` is not the same thing as
/// the kernel thread ID returned by a call to `gettid(2)`.
#[inline]
pub fn pthread_self() -> Pthread {
    unsafe { libc::pthread_self() }
}

feature! {
#![feature = "signal"]

/// Send a signal to a thread (see [`pthread_kill(3)`]).
///
/// If `signal` is `None`, `pthread_kill` will only preform error checking and
/// won't send any signal.
///
/// [`pthread_kill(3)`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/pthread_kill.html
#[cfg(not(target_os = "redox"))]
pub fn pthread_kill<T>(thread: Pthread, signal: T) -> Result<()>
    where T: Into<Option<crate::sys::signal::Signal>>
{
    let sig = match signal.into() {
        Some(s) => s as libc::c_int,
        None => 0,
    };
    let res = unsafe { libc::pthread_kill(thread, sig) };
    Errno::result(res).map(drop)
}
}

/// Checks whether the system supports per-thread protection on region created with MAP_JIT
///
/// Mostly meaningful only on arm64 architecture.
#[cfg(target_vendor = "apple")]
pub fn pthread_jit_write_protect_supported_np() -> Result<bool> {
    Ok(unsafe { libc::pthread_jit_write_protect_supported_np() == 1 })
}

/// Enable/disable per-thread protection on region created with MAP_JIT
///
/// Mostly meaningful only on arm64 architecture.
#[cfg(target_vendor = "apple")]
pub fn pthread_jit_write_protect_np(enabled: bool) {
    unsafe { libc::pthread_jit_write_protect_np(enabled as i32) };
}
