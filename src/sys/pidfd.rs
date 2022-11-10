//! Interfaces for Linux's PID file descriptors.

use std::os::unix::io::RawFd;
use crate::Result;
use crate::errno::Errno;
use crate::unistd::Pid;

libc_bitflags!(
    /// Options that change the behavior of [`pidfd_open`].
    pub struct PidFdOpenFlag: libc::c_uint {
        /// Return a nonblocking file descriptor. (since Linux 5.10)
        ///
        /// If the process referred to by the file descriptor has not yet terminated,
        /// then an attempt to wait on the file descriptor using [`waitid(2)`] will
        /// immediately return the error `EAGAIN` rather than blocking.
        ///
        /// [`waitid(2)`]: https://man7.org/linux/man-pages/man2/waitid.2.html
        PIDFD_NONBLOCK;
    }
);

/// Obtain a file descriptor that refers to a process. (since Linux 5.3)
///
/// The `pidfd_open(2)` creates a file descriptor that refers to the process
/// whose PID is specified in pid. The file descriptor is returned as the function
/// result; the close-on-exec flag is set on the file descriptor.
///
/// For more information, see [`pidfd_open(2)`].
///
/// [`pidfd_open(2)`]: https://man7.org/linux/man-pages/man2/pidfd_open.2.html
pub fn pidfd_open(pid: Pid, flags: PidFdOpenFlag) -> Result<RawFd> {
    let ret = unsafe {
        libc::syscall(libc::SYS_pidfd_open, pid, flags)
    };
    Errno::result(ret).map(|r| r as RawFd)
}

/// Obtain a duplicate of another process's file descriptor. (since Linux 5.6)
///
/// The `pidfd_getfd(2)` system call allocates a new file descriptor in the calling
/// process. This new file descriptor is a duplicate of an existing file descriptor,
/// `target_fd`, in the process referred to by the PID file descriptor pidfd.
///
/// For more information, see [`pidfd_getfd(2)`].
///
/// [`pidfd_getfd(2)`]: https://man7.org/linux/man-pages/man2/pidfd_getfd.2.html
pub fn pidfd_getfd(pidfd: RawFd, target_fd: RawFd) -> Result<RawFd> {
    let ret = unsafe {
        libc::syscall(libc::SYS_pidfd_getfd, pidfd, target_fd, 0)
    };
    Errno::result(ret).map(|r| r as RawFd)
}

/// Send a signal to a process specified by a file descriptor.
///
/// The `pidfd_send_signal(2)` system call sends the signal sig to the target process
/// referred to by pidfd, a PID file descriptor that refers to a process.
///
/// For more information, see [`pidfd_send_signal(2)`].
///
/// [`pidfd_send_signal(2)`]: https://man7.org/linux/man-pages/man2/pidfd_send_signal.2.html
#[cfg(feature = "signal")]
pub fn pidfd_send_signal<T: Into<Option<crate::sys::signal::Signal>>>(pidfd: RawFd, signal: T, sig_info: Option<&libc::siginfo_t>) -> Result<()> {
    let signal = match signal.into() {
        Some(signal) => signal as libc::c_int,
        _ => 0,
    };
    let ret = unsafe {
        libc::syscall(libc::SYS_pidfd_send_signal, pidfd, signal, sig_info, 0)
    };
    Errno::result(ret).map(drop)
}
