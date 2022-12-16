//! Wait for events to trigger on specific file descriptors
use std::os::unix::io::{AsFd, AsRawFd, BorrowedFd};

use crate::errno::Errno;
use crate::Result;

/// This is a wrapper around `libc::pollfd`.
///
/// It's meant to be used as an argument to the [`poll`](fn.poll.html) and
/// [`ppoll`](fn.ppoll.html) functions to specify the events of interest
/// for a specific file descriptor.
///
/// After a call to `poll` or `ppoll`, the events that occurred can be
/// retrieved by calling [`revents()`](#method.revents) on the `PollFd`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PollFd<'fd> {
    pollfd: libc::pollfd,
    _fd: std::marker::PhantomData<BorrowedFd<'fd>>,
}

impl<'fd> PollFd<'fd> {
    /// Creates a new `PollFd` specifying the events of interest
    /// for a given file descriptor.
    //
    // Different from other I/O-safe interfaces, here, we have to take `AsFd`
    // by reference to prevent the case where the `fd` is closed but it is
    // still in use. For example:
    //
    // ```rust
    // let (reader, _) = pipe().unwrap();
    //
    // // If `PollFd::new()` takes `AsFd` by value, then `reader` will be consumed,
    // // but the file descriptor of `reader` will still be in use.
    // let pollfd = PollFd::new(reader, flag);
    //
    // // Do something with `pollfd`, which uses the CLOSED fd.
    // ```
    pub fn new<Fd: AsFd>(fd: &'fd Fd, events: PollFlags) -> PollFd<'fd> {
        PollFd {
            pollfd: libc::pollfd {
                fd: fd.as_fd().as_raw_fd(),
                events: events.bits(),
                revents: PollFlags::empty().bits(),
            },
            _fd: std::marker::PhantomData,
        }
    }

    /// Returns the events that occurred in the last call to `poll` or `ppoll`.  Will only return
    /// `None` if the kernel provides status flags that Nix does not know about.
    pub fn revents(self) -> Option<PollFlags> {
        PollFlags::from_bits(self.pollfd.revents)
    }

    /// Returns if any of the events of interest occured in the last call to `poll` or `ppoll`. Will
    /// only return `None` if the kernel provides status flags that Nix does not know about.
    ///
    /// Equivalent to `x.revents()? != PollFlags::empty()`.
    ///
    /// This is marginally more efficient than [`PollFd::all`].
    pub fn any(self) -> Option<bool> {
        Some(self.revents()? != PollFlags::empty())
    }

    /// Returns if all the events of interest occured in the last call to `poll` or `ppoll`. Will
    /// only return `None` if the kernel provides status flags that Nix does not know about.
    ///
    /// Equivalent to `x.revents()? & x.events() == x.events()`.
    ///
    /// This is marginally less efficient than [`PollFd::any`].
    pub fn all(self) -> Option<bool> {
        Some(self.revents()? & self.events() == self.events())
    }

    /// The events of interest for this `PollFd`.
    pub fn events(self) -> PollFlags {
        PollFlags::from_bits(self.pollfd.events).unwrap()
    }

    /// Modify the events of interest for this `PollFd`.
    pub fn set_events(&mut self, events: PollFlags) {
        self.pollfd.events = events.bits();
    }
}

impl<'fd> AsFd for PollFd<'fd> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        // Safety:
        //
        // BorrowedFd::borrow_raw(RawFd) requires that the raw fd being passed
        // must remain open for the duration of the returned BorrowedFd, this is
        // guaranteed as the returned BorrowedFd has the lifetime parameter same
        // as `self`:
        // "fn as_fd<'self>(&'self self) -> BorrowedFd<'self>"
        // which means that `self` (PollFd) is guaranteed to outlive the returned
        // BorrowedFd. (Lifetime: PollFd > BorrowedFd)
        //
        // And the lifetime parameter of PollFd::new(fd, ...) ensures that `fd`
        // (an owned file descriptor) must outlive the returned PollFd:
        // "pub fn new<Fd: AsFd>(fd: &'fd Fd, events: PollFlags) -> PollFd<'fd>"
        // (Lifetime: Owned fd > PollFd)
        //
        // With two above relationships, we can conclude that the `Owned file
        // descriptor` will outlive the returned BorrowedFd,
        // (Lifetime: Owned fd > BorrowedFd)
        // i.e., the raw fd being passed will remain valid for the lifetime of
        // the returned BorrowedFd.
        unsafe { BorrowedFd::borrow_raw(self.pollfd.fd) }
    }
}

libc_bitflags! {
    /// These flags define the different events that can be monitored by `poll` and `ppoll`
    pub struct PollFlags: libc::c_short {
        /// There is data to read.
        POLLIN;
        /// There is some exceptional condition on the file descriptor.
        ///
        /// Possibilities include:
        ///
        /// *  There is out-of-band data on a TCP socket (see
        ///    [tcp(7)](https://man7.org/linux/man-pages/man7/tcp.7.html)).
        /// *  A pseudoterminal master in packet mode has seen a state
        ///    change on the slave (see
        ///    [ioctl_tty(2)](https://man7.org/linux/man-pages/man2/ioctl_tty.2.html)).
        /// *  A cgroup.events file has been modified (see
        ///    [cgroups(7)](https://man7.org/linux/man-pages/man7/cgroups.7.html)).
        POLLPRI;
        /// Writing is now possible, though a write larger that the
        /// available space in a socket or pipe will still block (unless
        /// `O_NONBLOCK` is set).
        POLLOUT;
        /// Equivalent to [`POLLIN`](constant.POLLIN.html)
        #[cfg(not(target_os = "redox"))]
        #[cfg_attr(docsrs, doc(cfg(all())))]
        POLLRDNORM;
        #[cfg(not(target_os = "redox"))]
        #[cfg_attr(docsrs, doc(cfg(all())))]
        /// Equivalent to [`POLLOUT`](constant.POLLOUT.html)
        POLLWRNORM;
        /// Priority band data can be read (generally unused on Linux).
        #[cfg(not(target_os = "redox"))]
        #[cfg_attr(docsrs, doc(cfg(all())))]
        POLLRDBAND;
        /// Priority data may be written.
        #[cfg(not(target_os = "redox"))]
        #[cfg_attr(docsrs, doc(cfg(all())))]
        POLLWRBAND;
        /// Error condition (only returned in
        /// [`PollFd::revents`](struct.PollFd.html#method.revents);
        /// ignored in [`PollFd::new`](struct.PollFd.html#method.new)).
        /// This bit is also set for a file descriptor referring to the
        /// write end of a pipe when the read end has been closed.
        POLLERR;
        /// Hang up (only returned in [`PollFd::revents`](struct.PollFd.html#method.revents);
        /// ignored in [`PollFd::new`](struct.PollFd.html#method.new)).
        /// Note that when reading from a channel such as a pipe or a stream
        /// socket, this event merely indicates that the peer closed its
        /// end of the channel.  Subsequent reads from the channel will
        /// return 0 (end of file) only after all outstanding data in the
        /// channel has been consumed.
        POLLHUP;
        /// Invalid request: `fd` not open (only returned in
        /// [`PollFd::revents`](struct.PollFd.html#method.revents);
        /// ignored in [`PollFd::new`](struct.PollFd.html#method.new)).
        POLLNVAL;
    }
}

/// `poll` waits for one of a set of file descriptors to become ready to perform I/O.
/// ([`poll(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/poll.html))
///
/// `fds` contains all [`PollFd`](struct.PollFd.html) to poll.
/// The function will return as soon as any event occur for any of these `PollFd`s.
///
/// The `timeout` argument specifies the number of milliseconds that `poll()`
/// should block waiting for a file descriptor to become ready.  The call
/// will block until either:
///
/// *  a file descriptor becomes ready;
/// *  the call is interrupted by a signal handler; or
/// *  the timeout expires.
///
/// Note that the timeout interval will be rounded up to the system clock
/// granularity, and kernel scheduling delays mean that the blocking
/// interval may overrun by a small amount.  Specifying a negative value
/// in timeout means an infinite timeout.  Specifying a timeout of zero
/// causes `poll()` to return immediately, even if no file descriptors are
/// ready.
pub fn poll(fds: &mut [PollFd], timeout: libc::c_int) -> Result<libc::c_int> {
    let res = unsafe {
        libc::poll(
            fds.as_mut_ptr() as *mut libc::pollfd,
            fds.len() as libc::nfds_t,
            timeout,
        )
    };

    Errno::result(res)
}

feature! {
#![feature = "signal"]
/// `ppoll()` allows an application to safely wait until either a file
/// descriptor becomes ready or until a signal is caught.
/// ([`poll(2)`](https://man7.org/linux/man-pages/man2/poll.2.html))
///
/// `ppoll` behaves like `poll`, but let you specify what signals may interrupt it
/// with the `sigmask` argument. If you want `ppoll` to block indefinitely,
/// specify `None` as `timeout` (it is like `timeout = -1` for `poll`).
/// If `sigmask` is `None`, then no signal mask manipulation is performed,
/// so in that case `ppoll` differs from `poll` only in the precision of the
/// timeout argument.
///
#[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd", target_os = "linux"))]
pub fn ppoll(
    fds: &mut [PollFd],
    timeout: Option<crate::sys::time::TimeSpec>,
    sigmask: Option<crate::sys::signal::SigSet>
    ) -> Result<libc::c_int>
{
    let timeout = timeout.as_ref().map_or(core::ptr::null(), |r| r.as_ref());
    let sigmask = sigmask.as_ref().map_or(core::ptr::null(), |r| r.as_ref());
    let res = unsafe {
        libc::ppoll(fds.as_mut_ptr() as *mut libc::pollfd,
                    fds.len() as libc::nfds_t,
                    timeout,
                    sigmask)
    };
    Errno::result(res)
}
}
