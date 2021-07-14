//! Timer API via file descriptors.
//!
//! Timer FD is a Linux-only API to create timers and get expiration
//! notifications through file descriptors.
//!
//! For more documentation, please read [timerfd_create(2)](https://man7.org/linux/man-pages/man2/timerfd_create.2.html).
//!
//! # Examples
//!
//! Create a new one-shot timer that expires after 1 second.
//! ```
//! # use std::os::unix::io::AsRawFd;
//! # use nix::sys::timerfd::{TimerFd, ClockId, TimerFlags, TimerSetTimeFlags,
//! #    Expiration};
//! # use nix::sys::time::{TimeSpec, TimeValLike};
//! # use nix::unistd::read;
//! #
//! // We create a new monotonic timer.
//! let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty())
//!     .unwrap();
//!
//! // We set a new one-shot timer in 1 seconds.
//! timer.set(
//!     Expiration::OneShot(TimeSpec::seconds(1)),
//!     TimerSetTimeFlags::empty()
//! ).unwrap();
//!
//! // We wait for the timer to expire.
//! timer.wait().unwrap();
//! ```
use crate::sys::time::TimeSpec;
use crate::unistd::read;
use crate::{errno::Errno, Result};
use bitflags::bitflags;
use libc::c_int;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

/// A timerfd instance. This is also a file descriptor, you can feed it to
/// other interfaces consuming file descriptors, epoll for example.
#[derive(Debug)]
pub struct TimerFd {
    fd: RawFd,
}

impl AsRawFd for TimerFd {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl FromRawFd for TimerFd {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        TimerFd { fd }
    }
}

libc_enum! {
    /// The type of the clock used to mark the progress of the timer. For more
    /// details on each kind of clock, please refer to [timerfd_create(2)](https://man7.org/linux/man-pages/man2/timerfd_create.2.html).
    #[repr(i32)]
    pub enum ClockId {
        CLOCK_REALTIME,
        CLOCK_MONOTONIC,
        CLOCK_BOOTTIME,
        CLOCK_REALTIME_ALARM,
        CLOCK_BOOTTIME_ALARM,
    }
}

libc_bitflags! {
    /// Additional flags to change the behaviour of the file descriptor at the
    /// time of creation.
    pub struct TimerFlags: c_int {
        TFD_NONBLOCK;
        TFD_CLOEXEC;
    }
}

bitflags! {
    /// Flags that are used for arming the timer.
    pub struct TimerSetTimeFlags: libc::c_int {
        const TFD_TIMER_ABSTIME = libc::TFD_TIMER_ABSTIME;
    }
}

#[derive(Debug, Clone, Copy)]
struct TimerSpec(libc::itimerspec);

impl TimerSpec {
    pub fn none() -> Self {
        Self(libc::itimerspec {
            it_interval: libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            it_value: libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
        })
    }

    pub fn is_none(&self) -> bool {
        self.0.it_interval.tv_sec == 0
            && self.0.it_interval.tv_nsec == 0
            && self.0.it_value.tv_sec == 0
            && self.0.it_value.tv_nsec == 0
    }
}

impl AsRef<libc::itimerspec> for TimerSpec {
    fn as_ref(&self) -> &libc::itimerspec {
        &self.0
    }
}

impl AsMut<libc::itimerspec> for TimerSpec {
    fn as_mut(&mut self) -> &mut libc::itimerspec {
        &mut self.0
    }
}

impl From<Expiration> for TimerSpec {
    fn from(expiration: Expiration) -> TimerSpec {
        match expiration {
            Expiration::OneShot(t) => TimerSpec(libc::itimerspec {
                it_interval: libc::timespec {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                it_value: *t.as_ref(),
            }),
            Expiration::IntervalDelayed(start, interval) => TimerSpec(libc::itimerspec {
                it_interval: *interval.as_ref(),
                it_value: *start.as_ref(),
            }),
            Expiration::Interval(t) => TimerSpec(libc::itimerspec {
                it_interval: *t.as_ref(),
                it_value: *t.as_ref(),
            }),
        }
    }
}

impl From<Option<Expiration>> for TimerSpec {
    fn from(expiration: Option<Expiration>) -> Self {
        match expiration {
            None => Self::none(),
            Some(e) => e.into(),
        }
    }
}

impl From<TimerSpec> for Expiration {
    fn from(timerspec: TimerSpec) -> Expiration {
        match timerspec {
            TimerSpec(libc::itimerspec {
                it_interval:
                    libc::timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    },
                it_value: ts,
            }) => Expiration::OneShot(ts.into()),
            TimerSpec(libc::itimerspec {
                it_interval: int_ts,
                it_value: val_ts,
            }) => {
                if (int_ts.tv_sec == val_ts.tv_sec) && (int_ts.tv_nsec == val_ts.tv_nsec) {
                    Expiration::Interval(int_ts.into())
                } else {
                    Expiration::IntervalDelayed(val_ts.into(), int_ts.into())
                }
            }
        }
    }
}

impl Into<Option<Expiration>> for TimerSpec {
    fn into(self) -> Option<Expiration> {
        if self.is_none() {
            None
        } else {
            Some(self.into())
        }
    }
}

/// An enumeration allowing the definition of the expiration time of an alarm,
/// recurring or not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Expiration {
    OneShot(TimeSpec),
    IntervalDelayed(TimeSpec, TimeSpec),
    Interval(TimeSpec),
}

/// Directly wraps [`libc::timerfd_create`].
///
/// It may be more convenient to use [`TimerFd`].
pub fn timerfd_create(clockid: ClockId, flags: TimerFlags) -> Result<RawFd> {
    Errno::result(unsafe { libc::timerfd_create(clockid as i32, flags.bits()) })
}

/// Directly wraps [`libc::timerfd_settime`].
///
/// It may be more convenient to use [`TimerFd`].
pub fn timerfd_settime(
    fd: RawFd,
    flags: TimerSetTimeFlags,
    new_value: Option<Expiration>,
) -> Result<Option<Expiration>> {
    let new_value: TimerSpec = new_value.into();
    let mut old_value = TimerSpec::none();

    Errno::result(unsafe {
        libc::timerfd_settime(fd, flags.bits(), new_value.as_ref(), old_value.as_mut())
    })?;

    Ok(old_value.into())
}

/// Directly wraps [`libc::timerfd_gettime`].
///
/// It may be more convenient to use [`TimerFd`].
pub fn timerfd_gettime(fd: RawFd) -> Result<Option<Expiration>> {
    let mut value = TimerSpec::none();

    Errno::result(unsafe { libc::timerfd_gettime(fd, value.as_mut()) })?;

    Ok(value.into())
}

impl TimerFd {
    /// Creates a new timer based on the clock defined by `clockid`. The
    /// underlying fd can be assigned specific flags with `flags` (CLOEXEC,
    /// NONBLOCK). The underlying fd will be closed on drop.
    pub fn new(clockid: ClockId, flags: TimerFlags) -> Result<Self> {
        timerfd_create(clockid, flags).map(|fd| Self { fd })
    }

    /// Sets a new alarm on the timer.
    ///
    /// # Types of alarm
    ///
    /// There are 3 types of alarms you can set:
    ///
    ///   - one shot: the alarm will trigger once after the specified amount of
    /// time.
    ///     Example: I want an alarm to go off in 60s and then disables itself.
    ///
    ///   - interval: the alarm will trigger every specified interval of time.
    ///     Example: I want an alarm to go off every 60s. The alarm will first
    ///     go off 60s after I set it and every 60s after that. The alarm will
    ///     not disable itself.
    ///
    ///   - interval delayed: the alarm will trigger after a certain amount of
    ///     time and then trigger at a specified interval.
    ///     Example: I want an alarm to go off every 60s but only start in 1h.
    ///     The alarm will first trigger 1h after I set it and then every 60s
    ///     after that. The alarm will not disable itself.
    ///
    /// # Relative vs absolute alarm
    ///
    /// If you do not set any `TimerSetTimeFlags`, then the `TimeSpec` you pass
    /// to the `Expiration` you want is relative. If however you want an alarm
    /// to go off at a certain point in time, you can set `TFD_TIMER_ABSTIME`.
    /// Then the one shot TimeSpec and the delay TimeSpec of the delayed
    /// interval are going to be interpreted as absolute.
    ///
    /// # Disabling alarms
    ///
    /// Note: Only one alarm can be set for any given timer. Setting a new alarm
    /// actually removes the previous one.
    ///
    /// Note: Setting a one shot alarm with a 0s TimeSpec disables the alarm
    /// altogether.
    pub fn set(&self, expiration: Expiration, flags: TimerSetTimeFlags) -> Result<()> {
        timerfd_settime(self.fd, flags, Some(expiration)).map(drop)
    }

    /// Get the parameters for the alarm currently set, if any.
    pub fn get(&self) -> Result<Option<Expiration>> {
        timerfd_gettime(self.fd)
    }

    /// Remove the alarm if any is set.
    pub fn unset(&self) -> Result<()> {
        timerfd_settime(self.fd, TimerSetTimeFlags::empty(), None).map(drop)
    }

    /// Wait for the configured alarm to expire.
    ///
    /// Note: If the alarm is unset, then you will wait forever.
    pub fn wait(&self) -> Result<()> {
        loop {
            if let Err(e) = read(self.fd, &mut [0u8; 8]) {
                match e {
                    Errno::EINTR => continue,
                    _ => return Err(e),
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl Drop for TimerFd {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            let result = Errno::result(unsafe { libc::close(self.fd) });
            if let Err(Errno::EBADF) = result {
                panic!("close of TimerFd encountered EBADF");
            }
        }
    }
}
