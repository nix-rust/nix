//! Wait for events to trigger on specific file descriptors
use std::os::unix::io::{AsFd, AsRawFd, BorrowedFd};
use std::time::Duration;

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
    ///
    /// # Examples
    /// ```no_run
    /// # use std::os::unix::io::{AsFd, AsRawFd, FromRawFd};
    /// # use nix::{
    /// #     poll::{PollTimeout, PollFd, PollFlags, poll},
    /// #     unistd::{pipe, read}
    /// # };
    /// let (r, w) = pipe().unwrap();
    /// let pfd = PollFd::new(r.as_fd(), PollFlags::POLLIN);
    /// let mut fds = [pfd];
    /// poll(&mut fds, PollTimeout::NONE).unwrap();
    /// let mut buf = [0u8; 80];
    /// read(r.as_raw_fd(), &mut buf[..]);
    /// ```
    // Unlike I/O functions, constructors like this must take `BorrowedFd`
    // instead of AsFd or &AsFd.  Otherwise, an `OwnedFd` argument would be
    // dropped at the end of the method, leaving the structure referencing a
    // closed file descriptor.  For example:
    //
    // ```rust
    // let (r, _) = pipe().unwrap();
    // let pollfd = PollFd::new(r, flag);  // Drops the OwnedFd
    // // Do something with `pollfd`, which uses the CLOSED fd.
    // ```
    pub fn new(fd: BorrowedFd<'fd>, events: PollFlags) -> PollFd<'fd> {
        PollFd {
            pollfd: libc::pollfd {
                fd: fd.as_raw_fd(),
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
        POLLRDNORM;
        #[cfg(not(target_os = "redox"))]
        /// Equivalent to [`POLLOUT`](constant.POLLOUT.html)
        POLLWRNORM;
        /// Priority band data can be read (generally unused on Linux).
        #[cfg(not(target_os = "redox"))]
        POLLRDBAND;
        /// Priority data may be written.
        #[cfg(not(target_os = "redox"))]
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

/// Timeout argument for [`poll`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct PollTimeout(i32);

impl PollTimeout {
    /// Blocks indefinitely.
    ///
    /// > Specifying a negative value in timeout means an infinite timeout.
    pub const NONE: Self = Self(-1);
    /// Returns immediately.
    ///
    /// > Specifying a timeout of zero causes poll() to return immediately, even if no file
    /// > descriptors are ready.
    pub const ZERO: Self = Self(0);
    /// Blocks for at most [`std::i32::MAX`] milliseconds.
    pub const MAX: Self = Self(i32::MAX);
    /// Returns if `self` equals [`PollTimeout::NONE`].
    pub fn is_none(&self) -> bool {
        // > Specifying a negative value in timeout means an infinite timeout.
        *self <= Self::NONE
    }
    /// Returns if `self` does not equal [`PollTimeout::NONE`].
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
    /// Returns the timeout in milliseconds if there is some, otherwise returns `None`.
    pub fn as_millis(&self) -> Option<u32> {
        self.is_some().then_some(u32::try_from(self.0).unwrap())
    }
    /// Returns the timeout as a `Duration` if there is some, otherwise returns `None`.
    pub fn timeout(&self) -> Option<Duration> {
        self.as_millis()
            .map(|x| Duration::from_millis(u64::from(x)))
    }
}

/// Error type for integer conversions into `PollTimeout`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollTimeoutTryFromError {
    /// Passing a value less than -1 is invalid on some systems, see
    /// <https://man.freebsd.org/cgi/man.cgi?poll#end>.
    TooNegative,
    /// Passing a value greater than `i32::MAX` is invalid.
    TooPositive,
}

impl std::fmt::Display for PollTimeoutTryFromError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooNegative => write!(f, "Passed a negative timeout less than -1."),
            Self::TooPositive => write!(f, "Passed a positive timeout greater than `i32::MAX` milliseconds.")
        }
    }
}

impl std::error::Error for PollTimeoutTryFromError {}

impl<T: Into<PollTimeout>> From<Option<T>> for PollTimeout {
    fn from(x: Option<T>) -> Self {
        x.map_or(Self::NONE, |x| x.into())
    }
}
impl TryFrom<Duration> for PollTimeout {
    type Error = PollTimeoutTryFromError;
    fn try_from(x: Duration) -> std::result::Result<Self, Self::Error> {
        Ok(Self(
            i32::try_from(x.as_millis())
                .map_err(|_| PollTimeoutTryFromError::TooPositive)?,
        ))
    }
}
impl TryFrom<u128> for PollTimeout {
    type Error = PollTimeoutTryFromError;
    fn try_from(x: u128) -> std::result::Result<Self, Self::Error> {
        Ok(Self(
            i32::try_from(x)
                .map_err(|_| PollTimeoutTryFromError::TooPositive)?,
        ))
    }
}
impl TryFrom<u64> for PollTimeout {
    type Error = PollTimeoutTryFromError;
    fn try_from(x: u64) -> std::result::Result<Self, Self::Error> {
        Ok(Self(
            i32::try_from(x)
                .map_err(|_| PollTimeoutTryFromError::TooPositive)?,
        ))
    }
}
impl TryFrom<u32> for PollTimeout {
    type Error = PollTimeoutTryFromError;
    fn try_from(x: u32) -> std::result::Result<Self, Self::Error> {
        Ok(Self(
            i32::try_from(x)
                .map_err(|_| PollTimeoutTryFromError::TooPositive)?,
        ))
    }
}
impl From<u16> for PollTimeout {
    fn from(x: u16) -> Self {
        Self(i32::from(x))
    }
}
impl From<u8> for PollTimeout {
    fn from(x: u8) -> Self {
        Self(i32::from(x))
    }
}
impl TryFrom<i128> for PollTimeout {
    type Error = PollTimeoutTryFromError;
    fn try_from(x: i128) -> std::result::Result<Self, Self::Error> {
        match x {
            ..=-2 => Err(PollTimeoutTryFromError::TooNegative),
            -1.. => Ok(Self(
                i32::try_from(x)
                    .map_err(|_| PollTimeoutTryFromError::TooPositive)?,
            )),
        }
    }
}
impl TryFrom<i64> for PollTimeout {
    type Error = PollTimeoutTryFromError;
    fn try_from(x: i64) -> std::result::Result<Self, Self::Error> {
        match x {
            ..=-2 => Err(PollTimeoutTryFromError::TooNegative),
            -1.. => Ok(Self(
                i32::try_from(x)
                    .map_err(|_| PollTimeoutTryFromError::TooPositive)?,
            )),
        }
    }
}
impl TryFrom<i32> for PollTimeout {
    type Error = PollTimeoutTryFromError;
    fn try_from(x: i32) -> std::result::Result<Self, Self::Error> {
        match x {
            ..=-2 => Err(PollTimeoutTryFromError::TooNegative),
            -1.. => Ok(Self(x)),
        }
    }
}
impl TryFrom<i16> for PollTimeout {
    type Error = PollTimeoutTryFromError;
    fn try_from(x: i16) -> std::result::Result<Self, Self::Error> {
        match x {
            ..=-2 => Err(PollTimeoutTryFromError::TooNegative),
            -1.. => Ok(Self(i32::from(x))),
        }
    }
}
impl TryFrom<i8> for PollTimeout {
    type Error = PollTimeoutTryFromError;
    fn try_from(x: i8) -> std::result::Result<Self, Self::Error> {
        match x {
            ..=-2 => Err(PollTimeoutTryFromError::TooNegative),
            -1.. => Ok(Self(i32::from(x))),
        }
    }
}
impl TryFrom<PollTimeout> for Duration {
    type Error = ();
    fn try_from(x: PollTimeout) -> std::result::Result<Self, ()> {
        x.timeout().ok_or(())
    }
}
impl TryFrom<PollTimeout> for u128 {
    type Error = <Self as TryFrom<i32>>::Error;
    fn try_from(x: PollTimeout) -> std::result::Result<Self, Self::Error> {
        Self::try_from(x.0)
    }
}
impl TryFrom<PollTimeout> for u64 {
    type Error = <Self as TryFrom<i32>>::Error;
    fn try_from(x: PollTimeout) -> std::result::Result<Self, Self::Error> {
        Self::try_from(x.0)
    }
}
impl TryFrom<PollTimeout> for u32 {
    type Error = <Self as TryFrom<i32>>::Error;
    fn try_from(x: PollTimeout) -> std::result::Result<Self, Self::Error> {
        Self::try_from(x.0)
    }
}
impl TryFrom<PollTimeout> for u16 {
    type Error = <Self as TryFrom<i32>>::Error;
    fn try_from(x: PollTimeout) -> std::result::Result<Self, Self::Error> {
        Self::try_from(x.0)
    }
}
impl TryFrom<PollTimeout> for u8 {
    type Error = <Self as TryFrom<i32>>::Error;
    fn try_from(x: PollTimeout) -> std::result::Result<Self, Self::Error> {
        Self::try_from(x.0)
    }
}
impl From<PollTimeout> for i128 {
    fn from(x: PollTimeout) -> Self {
        Self::from(x.0)
    }
}
impl From<PollTimeout> for i64 {
    fn from(x: PollTimeout) -> Self {
        Self::from(x.0)
    }
}
impl From<PollTimeout> for i32 {
    fn from(x: PollTimeout) -> Self {
        x.0
    }
}
impl TryFrom<PollTimeout> for i16 {
    type Error = <Self as TryFrom<i32>>::Error;
    fn try_from(x: PollTimeout) -> std::result::Result<Self, Self::Error> {
        Self::try_from(x.0)
    }
}
impl TryFrom<PollTimeout> for i8 {
    type Error = <Self as TryFrom<i32>>::Error;
    fn try_from(x: PollTimeout) -> std::result::Result<Self, Self::Error> {
        Self::try_from(x.0)
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
/// interval may overrun by a small amount.  Specifying a [`PollTimeout::NONE`]
/// in timeout means an infinite timeout.  Specifying a timeout of
/// [`PollTimeout::ZERO`] causes `poll()` to return immediately, even if no file
/// descriptors are ready.
pub fn poll<T: Into<PollTimeout>>(
    fds: &mut [PollFd],
    timeout: T,
) -> Result<libc::c_int> {
    let res = unsafe {
        libc::poll(
            fds.as_mut_ptr().cast(),
            fds.len() as libc::nfds_t,
            i32::from(timeout.into()),
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
        libc::ppoll(fds.as_mut_ptr().cast(),
                    fds.len() as libc::nfds_t,
                    timeout,
                    sigmask)
    };
    Errno::result(res)
}
}
