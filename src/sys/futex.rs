//! Fast user-space locking.

use crate::{Errno, Result};
use libc::{syscall, SYS_futex};
use std::cell::UnsafeCell;
use std::convert::TryFrom;
use std::os::unix::io::{FromRawFd, OwnedFd};
use std::time::Duration;

fn timespec(duration: Duration) -> libc::timespec {
    let tv_sec = duration.as_secs().try_into().unwrap();
    let tv_nsec = duration.subsec_nanos().try_into().unwrap();
    libc::timespec { tv_sec, tv_nsec }
}

fn unwrap_or_null<T>(option: Option<&T>) -> *const T {
    match option {
        Some(t) => t,
        None => std::ptr::null(),
    }
}

/// Fast user-space locking.
///
/// By default we presume the futex is not process-private, that is, it is used across processes. If
/// you know it is process-private you can set `PRIVATE` to `true`  which allows some additional
/// optimizations.
/// ```
/// # use nix::{
/// #   sys::{futex::Futex, mman::{mmap_anonymous, MapFlags, ProtFlags}},
/// #   errno::Errno,
/// #   unistd::{fork,ForkResult},
/// # };
/// # use std::{
/// #   time::{Instant, Duration},
/// #   num::NonZeroUsize,
/// #   mem::{ManuallyDrop, size_of},
/// #   os::unix::io::OwnedFd,
/// #   sync::Arc,
/// #   thread::{spawn, sleep},
/// # };
/// const TIMEOUT: Duration = Duration::from_millis(500);
/// const DELTA: Duration = Duration::from_millis(100);
/// # fn main() -> nix::Result<()> {
/// let futex: Futex = Futex::new(0);
///
/// // If the value of the futex is 0, wait for wake. Since the value is 0 and no wake occurs,
/// // we expect the timeout will pass.
///
/// let instant = Instant::now();
/// assert_eq!(futex.wait(0, Some(TIMEOUT)),Err(Errno::ETIMEDOUT));
/// assert!(instant.elapsed() > TIMEOUT);
///
/// // If the value of the futex is 1, wait for wake. Since the value is 0, not 1, this will
/// // return immediately.
///
/// let instant = Instant::now();
/// assert_eq!(futex.wait(1, Some(TIMEOUT)),Err(Errno::EAGAIN));
/// assert!(instant.elapsed() < DELTA);
///
/// // Test across threads
/// // -------------------------------------------------------------------------
///
/// let futex = Arc::new(futex);
/// let futex_clone = futex.clone();    
/// let instant = Instant::now();
/// spawn(move || {
///     sleep(TIMEOUT);
///     assert_eq!(futex_clone.wake(1),Ok(1));
/// });
/// assert_eq!(futex.wait(0, Some(2 * TIMEOUT)), Ok(()));
/// assert!(instant.elapsed() > TIMEOUT && instant.elapsed() < TIMEOUT + DELTA);
///
/// // Test across processes
/// // -------------------------------------------------------------------------
///
/// let shared_memory = unsafe { mmap_anonymous(
///     None,
///     NonZeroUsize::new_unchecked(size_of::<Futex<false>>()),
///     ProtFlags::PROT_WRITE | ProtFlags::PROT_READ,
///     MapFlags::MAP_SHARED | MapFlags::MAP_ANONYMOUS,
/// )? };
/// let futex_ptr = shared_memory.cast::<Futex<false>>();
/// let futex = unsafe { futex_ptr.as_ref() };
/// match unsafe { fork()? } {
///     ForkResult::Parent { child } => {
///         sleep(TIMEOUT);
///         assert_eq!(futex.wake(1),Ok(1));
///         // Wait for child process to exit
///         unsafe {
///             assert_eq!(libc::waitpid(child.as_raw(), std::ptr::null_mut(), 0), child.as_raw());
///         }
///     },
///     ForkResult::Child => {
///         let now = Instant::now();
///         assert_eq!(futex.wait(0, Some(2 * TIMEOUT)),Ok(()));
///         assert!(now.elapsed() > TIMEOUT && now.elapsed() < TIMEOUT + DELTA);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Futex<const PRIVATE: bool = false>(pub UnsafeCell<u32>);

impl<const PRIVATE: bool> Futex<PRIVATE> {
    const MASK: i32 = if PRIVATE { libc::FUTEX_PRIVATE_FLAG } else { 0 };

    /// Constructs new futex with a given `val`.
    pub fn new(val: u32) -> Self {
        Self(UnsafeCell::new(val))
    }
    
    /// If the value of the futex:
    /// - `== val`, the thread sleeps waiting for a [`Futex::wake`] call, in this case this thread
    ///   is considered a waiter on this futex.
    /// - `!= val`, then `Err` with [`Errno::EAGAIN`] is immediately returned.
    ///
    /// If the timeout is:
    /// - `Some(_)` it specifies a timeout for the wait.
    /// - `None` it will block indefinitely.
    ///
    /// Wraps [`libc::FUTEX_WAIT`].
    pub fn wait(&self, val: u32, timeout: Option<Duration>) -> Result<()> {
        let timespec = timeout.map(timespec);
        let timespec_ptr = unwrap_or_null(timespec.as_ref());

        let res = unsafe {
            syscall(
                SYS_futex,
                self.0.get(),
                Self::MASK | libc::FUTEX_WAIT,
                val,
                timespec_ptr,
            )
        };
        Errno::result(res).map(drop)
    }
    
    /// Wakes at most `val` waiters.
    ///
    /// - `val == 1` wakes a single waiter.
    /// - `val == u32::MAX` wakes all waiters.
    ///
    /// No guarantee is provided about which waiters are awoken. A waiter with a higher scheduling
    /// priority is not guaranteed to be awoken in preference to a waiter with a lower priority.
    ///
    /// Wraps [`libc::FUTEX_WAKE`].
    pub fn wake(&self, val: u32) -> Result<u32> {
        let res = unsafe {
            syscall(SYS_futex, self.0.get(), Self::MASK | libc::FUTEX_WAKE, val)
        };
        Errno::result(res).map(|x| u32::try_from(x).unwrap())
    }
    
    /// Creates a file descriptor associated with the futex.
    ///
    /// When [`Futex::wake`] is performed on the futex this file indicates being readable with
    /// `select`, `poll` and `epoll`.
    ///
    /// The file descriptor can be used to obtain asynchronous notifications: if val is nonzero,
    /// then, when another process or thread executes a FUTEX_WAKE, the caller will receive the
    /// signal number that was passed in val.
    ///
    /// **Because it was inherently racy, this is unsupported from Linux 2.6.26 onward.**
    ///
    /// Wraps [`libc::FUTEX_FD`].
    pub fn fd(&self, val: u32) -> Result<OwnedFd> {
        let res = unsafe {
            syscall(SYS_futex, self.0.get(), Self::MASK | libc::FUTEX_WAKE, val)
        };

        // On a 32 bit arch `x` will be an `i32` and will trigger this lint.
        #[allow(clippy::useless_conversion)]
        Errno::result(res)
            .map(|x| unsafe { OwnedFd::from_raw_fd(i32::try_from(x).unwrap()) })
    }
    
    /// [`Futex::cmp_requeue`] without the check being made using `val3`.
    ///
    /// Wraps [`libc::FUTEX_REQUEUE`].
    pub fn requeue(&self, val: u32, val2: u32, uaddr2: &Self) -> Result<u32> {
        let res = unsafe {
            syscall(
                SYS_futex,
                self.0.get(),
                Self::MASK | libc::FUTEX_CMP_REQUEUE,
                val,
                val2,
                &uaddr2.0,
            )
        };
        Errno::result(res).map(|x| u32::try_from(x).unwrap())
    }
    
    /// Wakes `val` waiters, moving remaining (up to `val2`) waiters to `uaddr2`.
    ///
    /// If the value of this futex `== val3` returns `Err` with [`Errno::EAGAIN`].
    ///
    /// Typical values to specify for `val` are `0` or `1` (Specifying `u32::MAX` makes the
    /// [`Futex::cmp_requeue`] equivalent to [`Futex::wake`]).
    ///
    /// Typical values to specify for `val2` are `1` or `u32::MAX` (Specifying `0` makes
    /// [`Futex::cmp_requeue`] equivalent to [`Futex::wait`]).
    ///
    /// Wraps [`libc::FUTEX_CMP_REQUEUE`].
    pub fn cmp_requeue(
        &self,
        val: u32,
        val2: u32,
        uaddr2: &Self,
        val3: u32,
    ) -> Result<u32> {
        let res = unsafe {
            syscall(
                SYS_futex,
                self.0.get(),
                Self::MASK | libc::FUTEX_CMP_REQUEUE,
                val,
                val2,
                &uaddr2.0,
                val3,
            )
        };
        Errno::result(res).map(|x| u32::try_from(x).unwrap())
    }
    
    /// Wraps [`libc::FUTEX_WAKE_OP`].
    pub fn wake_op(
        &self,
        val: u32,
        val2: u32,
        uaddr2: &Self,
        val3: u32,
    ) -> Result<u32> {
        let res = unsafe {
            syscall(
                SYS_futex,
                self.0.get(),
                Self::MASK | libc::FUTEX_WAKE_OP,
                val,
                val2,
                &uaddr2.0,
                val3,
            )
        };
        Errno::result(res).map(|x| u32::try_from(x).unwrap())
    }
    
    /// Wraps [`libc::FUTEX_WAIT_BITSET`].
    pub fn wait_bitset(
        &self,
        val: u32,
        timeout: Option<Duration>,
        val3: u32,
    ) -> Result<()> {
        let timespec = timeout.map(timespec);
        let timespec_ptr = unwrap_or_null(timespec.as_ref());

        let res = unsafe {
            syscall(
                SYS_futex,
                self.0.get(),
                Self::MASK | libc::FUTEX_WAIT_BITSET,
                val,
                timespec_ptr,
                val3,
            )
        };
        Errno::result(res).map(drop)
    }
    
    /// Wraps [`libc::FUTEX_WAKE_BITSET`].
    pub fn wake_bitset(&self, val: u32, val3: u32) -> Result<u32> {
        let res = unsafe {
            syscall(SYS_futex, self.0.get(), libc::FUTEX_WAKE_BITSET, val, val3)
        };
        Errno::result(res).map(|x| u32::try_from(x).unwrap())
    }
    
    /// Wraps [`libc::FUTEX_LOCK_PI`].
    pub fn lock_pi(&self, timeout: Option<Duration>) -> Result<()> {
        let timespec = timeout.map(timespec);
        let timespec_ptr = unwrap_or_null(timespec.as_ref());

        let res = unsafe {
            syscall(
                SYS_futex,
                self.0.get(),
                Self::MASK | libc::FUTEX_LOCK_PI,
                timespec_ptr,
            )
        };
        Errno::result(res).map(drop)
    }
    
    /// Wraps [`libc::FUTEX_LOCK_PI2`].
    #[cfg(target_os = "linux")]
    pub fn lock_pi2(&self, timeout: Option<Duration>) -> Result<()> {
        let timespec = timeout.map(timespec);
        let timespec_ptr = unwrap_or_null(timespec.as_ref());

        let res = unsafe {
            syscall(
                SYS_futex,
                self.0.get(),
                Self::MASK | libc::FUTEX_LOCK_PI2,
                timespec_ptr,
            )
        };
        Errno::result(res).map(drop)
    }
    
    /// Wraps [`libc::FUTEX_TRYLOCK_PI`].
    pub fn trylock_pi(&self) -> Result<()> {
        let res = unsafe {
            syscall(
                SYS_futex,
                self.0.get(),
                Self::MASK | libc::FUTEX_TRYLOCK_PI,
            )
        };
        Errno::result(res).map(drop)
    }
    
    /// `libc::FUTEX_UNLOCK_PI`
    pub fn unlock_pi(&self) -> Result<()> {
        let res = unsafe {
            syscall(SYS_futex, self.0.get(), Self::MASK | libc::FUTEX_UNLOCK_PI)
        };
        Errno::result(res).map(drop)
    }
    
    /// Wraps [`libc::FUTEX_CMP_REQUEUE_PI`].
    pub fn cmp_requeue_pi(
        &self,
        val: u32,
        val2: u32,
        uaddr2: &Self,
        val3: u32,
    ) -> Result<u32> {
        let res = unsafe {
            syscall(
                SYS_futex,
                self.0.get(),
                Self::MASK | libc::FUTEX_CMP_REQUEUE_PI,
                val,
                val2,
                &uaddr2.0,
                val3,
            )
        };
        Errno::result(res).map(|x| u32::try_from(x).unwrap())
    }
    
    /// Wraps [`libc::FUTEX_WAIT_REQUEUE_PI`].
    pub fn wait_requeue_pi(
        &self,
        val: u32,
        timeout: Option<Duration>,
        uaddr2: &Self,
    ) -> Result<()> {
        let timespec = timeout.map(timespec);
        let timespec_ptr = unwrap_or_null(timespec.as_ref());

        let res = unsafe {
            syscall(
                SYS_futex,
                self.0.get(),
                Self::MASK | libc::FUTEX_WAIT_REQUEUE_PI,
                val,
                timespec_ptr,
                &uaddr2.0,
            )
        };
        Errno::result(res).map(drop)
    }
}

unsafe impl Sync for Futex {}
