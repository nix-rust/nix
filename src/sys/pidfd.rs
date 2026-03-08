//! PID file descriptor APIs.

use std::{
    hint::unreachable_unchecked,
    os::fd::{FromRawFd as _, OwnedFd, RawFd},
};

use bitflags::bitflags;

use crate::{errno::Errno, unistd::Pid};

bitflags! {
    /// Flags for [`pidfd_open`].
    #[derive(Copy, Clone)]
    pub struct PidfdFlags: libc::c_uint {
        /// See [`pidfd_open(2)`] for details.
        ///
        /// [`pidfd_open(2)`]: https://man7.org/linux/man-pages/man2/pidfd_open.2.html
        const PIDFD_NONBLOCK = libc::PIDFD_NONBLOCK;

        /// See [`pidfd_open(2)`] for details.
        ///
        /// [`pidfd_open(2)`]: https://man7.org/linux/man-pages/man2/pidfd_open.2.html
        const PIDFD_THREAD = libc::PIDFD_THREAD;
    }
}

/// Open a PIDFD of the provided PID.
///
/// See [`pidfd_open(2)`] for details.
///
/// [`pidfd_open(2)`]: https://man7.org/linux/man-pages/man2/pidfd_open.2.html
///
/// # Safety
///
/// This can race with the PID in `pid` getting released or recycled, unless the
/// PID is that of the calling process, in which case the only way a race could
/// happen is if the current process exits.
///
/// This function is [async-signal-safe], although it may modify `errno`.
///
/// [async-signal-safe]: https://man7.org/linux/man-pages/man7/signal-safety.7.html
pub fn pidfd_open(pid: Pid, flags: PidfdFlags) -> Result<OwnedFd, Errno> {
    // SAFETY:
    //
    // * Arguments passed to the syscall have the correct types.
    // * The kernel should not return a value that cannot fit in `RawFd`.
    // * The file descriptor returned by the kernel is open, owned, and requires
    //   only `close` to release its resources.
    // * The kernel should not return any negative value other than `-1`.
    unsafe {
        match libc::syscall(libc::SYS_pidfd_open, pid.as_raw(), flags.bits()) {
            fd if fd >= 0 => {
                Ok(OwnedFd::from_raw_fd(RawFd::try_from(fd).unwrap_unchecked()))
            }
            -1 => Err(Errno::last()),
            _ => unreachable_unchecked(),
        }
    }
}
