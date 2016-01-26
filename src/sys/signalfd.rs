//! Interface for the `signalfd` syscall.
//!
//! # Signal discarding
//! When a signal can't be delivered to a process (or thread), it will become a pending signal.
//! Failure to deliver could happen if the signal is blocked by every thread in the process or if
//! the signal handler is still handling a previous signal.
//!
//! If a signal is sent to a process (or thread) that already has a pending signal of the same
//! type, it will be discarded. This means that if signals of the same type are received faster than
//! they are processed, some of those signals will be dropped. Because of this limitation,
//! `signalfd` in itself cannot be used for reliable communication between processes or threads.
//!
//! Once the signal is unblocked, or the signal handler is finished, and a signal is still pending
//! (ie. not consumed from a signalfd) it will be delivered to the signal handler.
//!
//! Please note that signal discarding is not specific to `signalfd`, but also happens with regular
//! signal handlers.
use libc::{c_int, pid_t, uid_t};
use unistd;
use {Errno, Result};
use sys::signal::signal::siginfo as signal_siginfo;
pub use sys::signal::{self, SigSet};

use std::os::unix::io::{RawFd, AsRawFd};
use std::mem;

mod ffi {
    use libc::c_int;
    use sys::signal::sigset_t;

    extern {
        pub fn signalfd(fd: c_int, mask: *const sigset_t, flags: c_int) -> c_int;
    }
}

bitflags!{
    flags SfdFlags: c_int {
        const SFD_NONBLOCK  = 0o00004000, // O_NONBLOCK
        const SFD_CLOEXEC   = 0o02000000, // O_CLOEXEC
    }
}

pub const CREATE_NEW_FD: RawFd = -1;

/// Creates a new file descriptor for reading signals.
///
/// **Important:** please read the module level documentation about signal discarding before using
/// this function!
///
/// The `mask` parameter specifies the set of signals that can be accepted via this file descriptor.
///
/// A signal must be blocked on every thread in a process, otherwise it won't be visible from
/// signalfd (the default handler will be invoked instead).
///
/// See [the signalfd man page for more information](http://man7.org/linux/man-pages/man2/signalfd.2.html)
pub fn signalfd(fd: RawFd, mask: &SigSet, flags: SfdFlags) -> Result<RawFd> {
    unsafe {
        Errno::result(ffi::signalfd(fd as c_int, mask.as_ref(), flags.bits()))
    }
}

/// A helper struct for creating, reading and closing a `signalfd` instance.
///
/// **Important:** please read the module level documentation about signal discarding before using
/// this struct!
///
/// # Examples
///
/// ```
/// use nix::sys::signalfd::*;
///
/// let mut mask = SigSet::empty();
/// mask.add(signal::SIGUSR1).unwrap();
///
/// // Block the signal, otherwise the default handler will be invoked instead.
/// mask.thread_block().unwrap();
///
/// // Signals are queued up on the file descriptor
/// let mut sfd = SignalFd::with_flags(&mask, SFD_NONBLOCK).unwrap();
///
/// match sfd.read_signal() {
///     // we caught a signal
///     Ok(Some(sig)) => (),
///
///     // there were no signals waiting (only happens when the SFD_NONBLOCK flag is set,
///     // otherwise the read_signal call blocks)
///     Ok(None) => (),
///
///     Err(err) => (), // some error happend
/// }
/// ```
#[derive(Debug)]
pub struct SignalFd(RawFd);

impl SignalFd {
    pub fn new(mask: &SigSet) -> Result<SignalFd> {
        Self::with_flags(mask, SfdFlags::empty())
    }

    pub fn with_flags(mask: &SigSet, flags: SfdFlags) -> Result<SignalFd> {
        let fd = try!(signalfd(CREATE_NEW_FD, mask, flags));

        Ok(SignalFd(fd))
    }

    pub fn set_mask(&mut self, mask: &SigSet) -> Result<()> {
        signalfd(self.0, mask, SfdFlags::empty()).map(|_| ())
    }

    pub fn read_signal(&mut self) -> Result<Option<siginfo>> {
        let mut buffer: [u8; SIGINFO_SIZE] = unsafe { mem::uninitialized() };

        match unistd::read(self.0, &mut buffer) {
            Ok(SIGINFO_SIZE) => Ok(Some(unsafe { mem::transmute_copy(&buffer) })),
            Ok(_) => unreachable!("partial read on signalfd"),
            Err(Error::Sys(Errno::EAGAIN)) => Ok(None),
            Err(error) => Err(error)
        }
    }
}

impl Drop for SignalFd {
    fn drop(&mut self) {
        let _ = unistd::close(self.0);
    }
}

impl AsRawFd for SignalFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Iterator for SignalFd {
    type Item = siginfo;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_signal() {
            Ok(Some(sig)) => Some(sig),
            Ok(None) => None,
            Err(..) => None,
        }
    }
}

pub const SIGINFO_SIZE: usize = 128;
pub const SIGINFO_PADDING: usize = 48;

#[derive(Debug, Clone, PartialEq)]
#[repr(C, packed)]
pub struct siginfo {
    pub ssi_signo: u32,
    pub ssi_errno: i32,
    pub ssi_code: i32,
    pub ssi_pid: u32,
    pub ssi_uid: u32,
    pub ssi_fd: i32,
    pub ssi_tid: u32,
    pub ssi_band: u32,
    pub ssi_overrun: u32,
    pub ssi_trapno: u32,
    pub ssi_status: i32,
    pub ssi_int: i32,
    pub ssi_ptr: u64,
    pub ssi_utime: u64,
    pub ssi_stime: u64,
    pub ssi_addr: u64,
}

impl Into<signal_siginfo> for siginfo {
    fn into(self) -> signal_siginfo {
        signal_siginfo {
            si_signo: self.ssi_signo as c_int,
            si_errno: self.ssi_errno as c_int,
            si_code: self.ssi_code as c_int,
            pid: self.ssi_pid as pid_t,
            uid: self.ssi_uid as uid_t,
            status: self.ssi_status as c_int,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn check_siginfo_size() {
        assert_eq!(mem::size_of::<siginfo>() + SIGINFO_PADDING, SIGINFO_SIZE);
    }

    #[test]
    fn create_signalfd() {
        let mask = SigSet::empty();
        let fd = SignalFd::new(&mask);
        assert!(fd.is_ok());
    }

    #[test]
    fn create_signalfd_with_opts() {
        let mask = SigSet::empty();
        let fd = SignalFd::with_flags(&mask, SFD_CLOEXEC | SFD_NONBLOCK);
        assert!(fd.is_ok());
    }

    #[test]
    fn read_empty_signalfd() {
        let mask = SigSet::empty();
        let mut fd = SignalFd::with_flags(&mask, SFD_NONBLOCK).unwrap();

        let res = fd.read_signal();
        assert_eq!(res, Ok(None));
    }
}
