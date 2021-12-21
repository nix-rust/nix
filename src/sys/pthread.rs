//! Low level threading primitives

#[cfg(not(target_os = "redox"))]
use crate::errno::Errno;
#[cfg(not(target_os = "redox"))]
use crate::Result;
#[cfg(not(target_os = "redox"))]
use crate::sys::signal::Signal;
use libc::{self, pthread_t};
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
use std::ffi::CString;

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

/// Send a signal to a thread (see [`pthread_kill(3)`]).
///
/// If `signal` is `None`, `pthread_kill` will only preform error checking and
/// won't send any signal.
///
/// [`pthread_kill(3)`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/pthread_kill.html
#[cfg(not(target_os = "redox"))]
pub fn pthread_kill<T: Into<Option<Signal>>>(thread: Pthread, signal: T) -> Result<()> {
    let sig = match signal.into() {
        Some(s) => s as libc::c_int,
        None => 0,
    };
    let res = unsafe { libc::pthread_kill(thread, sig) };
    Errno::result(res).map(drop)
}

/// Obtain the name of the thread
///
/// On linux the name cannot exceed 16 length including null terminator.
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
pub fn pthread_getname_np(thread: Pthread) -> Result<CString> {
    let mut name = vec![0u8; 16];
    unsafe { libc::pthread_getname_np(thread, name.as_mut_ptr() as _, name.len()) };
    let zi = name.iter().position(|x| *x == b'\0').unwrap();
    name.truncate(zi);
    Ok(unsafe { CString::from_vec_unchecked(name) })
}

/// Set the name of the thread
/// [`pthread_setname_np(3)`](https://man7.org/linux/man-pages/man3/pthread_setname_np.3.html)
///
/// To override the default name of the thread, should not exceed 16 bytes.
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
pub fn pthread_setname_np<T: AsRef<[u8]>>(thread: Pthread, name: T) -> Result<libc::c_int> {
    match CString::new(name.as_ref()) {
        Ok(cname) => {
            let nameptr = cname.as_ptr();
            let res = unsafe { libc::pthread_setname_np(thread, nameptr) };
            Errno::result(res).map(|r| r)
        },
        Err(err) => Errno::result(err.nul_position()).map(|r| r as libc::c_int),
    }
}
