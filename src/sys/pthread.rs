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

/// Value to pass with a signal. Can be either integer or pointer.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SigVal {
    /// Use this variant to pass a integer to the sigval union
    Int(libc::c_int),
    /// Use this variant to pass a pointer to the sigval union
    Ptr(*mut libc::c_void),
}

// Because of macro/trait machinery in libc, libc doesn't provide this union.
// Only used to ensure that union type conversion is done exactly as C would.
#[repr(C)]
union sigval_union {
    ptr: *mut libc::c_void,
    int: libc::c_int,
}

use std::convert::From;

impl From<SigVal> for libc::sigval {
    fn from(sigval: SigVal) -> Self {
        match sigval {
            SigVal::Int(int) => {
                let as_ptr = unsafe { sigval_union { int }.ptr };
                libc::sigval { sival_ptr: as_ptr }
            }
            SigVal::Ptr(ptr) => {
                libc::sigval { sival_ptr: ptr }
            }
        }
    }
}

/// Queue a signal and data to a thread (see [`pthread_sigqueue(3)`]).
///
/// If `signal` is `None`, `pthread_sigqueue` will only preform error checking and
/// won't send any signal.
///
/// `pthread_sigqueue` is a GNU extension and is not available on other libcs
///
/// [`pthread_sigqueue(3)`]: https://man7.org/linux/man-pages/man3/pthread_sigqueue.3.html
#[cfg(all(any(target_os = "linux", target_os = "android"), target_env = "gnu"))]
pub fn pthread_sigqueue<T>(thread: Pthread, signal: T, sigval: SigVal) -> Result<()>
    where T: Into<Option<crate::sys::signal::Signal>>
{
    let sig = match signal.into() {
        Some(s) => s as libc::c_int,
        None => 0,
    };
    let res = unsafe { libc::pthread_sigqueue(thread, sig, sigval.into()) };
    Errno::result(res).map(drop)
}
}
