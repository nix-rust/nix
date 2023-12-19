//! Low level threading primitives

#[cfg(not(target_os = "redox"))]
use crate::errno::Errno;
#[cfg(not(target_os = "redox"))]
use crate::Result;
use libc::{self, pthread_t};
#[cfg(not(target_os = "redox"))]
use libc::c_int;

#[cfg(target_os = "linux")]
use std::cell::UnsafeCell;

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
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg(not(target_os = "redox"))]
pub fn pthread_kill<T>(thread: Pthread, signal: T) -> Result<()>
    where T: Into<Option<crate::sys::signal::Signal>>
{
    let sig = match signal.into() {
        Some(s) => s as c_int,
        None => 0,
    };
    let res = unsafe { libc::pthread_kill(thread, sig) };
    Errno::result(res).map(drop)
}
}

/// Mutex protocol.
#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Protocol {
    /// [`libc::PTHREAD_PRIO_NONE`]
    None = libc::PTHREAD_PRIO_NONE,
    /// [`libc::PTHREAD_PRIO_INHERIT`]
    Inherit = libc::PTHREAD_PRIO_INHERIT,
    /// [`libc::PTHREAD_PRIO_PROTECT`]
    Protect = libc::PTHREAD_PRIO_PROTECT
}
#[cfg(target_os = "linux")]
impl From<i32> for Protocol {
    fn from(x: i32) -> Self {
        match x {
            libc::PTHREAD_PRIO_NONE => Self::None,
            libc::PTHREAD_PRIO_INHERIT => Self::Inherit,
            libc::PTHREAD_PRIO_PROTECT => Self::Protect,
            _ => unreachable!()
        }
    }
}

/// Mutex attributes.
#[cfg(target_os = "linux")]
#[derive(Debug)]
#[repr(transparent)]
pub struct MutexAttr(libc::pthread_mutexattr_t);

#[cfg(target_os = "linux")]
impl MutexAttr {
    /// Wraps [`libc::pthread_mutexattr_init`].
    pub fn new() -> Result<Self> {
        let attr = unsafe {
            let mut uninit = std::mem::MaybeUninit::<libc::pthread_mutexattr_t>::uninit();
            Errno::result(libc::pthread_mutexattr_init(uninit.as_mut_ptr()))?;
            uninit.assume_init()
        };
        Ok(Self(attr))
    }

    /// Wraps [`libc::pthread_mutexattr_getpshared`].
    pub fn get_shared(&self) -> Result<bool> {
        let init = unsafe {
            let mut uninit = std::mem::MaybeUninit::uninit();
            Errno::result(libc::pthread_mutexattr_getpshared(&self.0,uninit.as_mut_ptr()))?;
            uninit.assume_init()
        };
        Ok(init == libc::PTHREAD_PROCESS_SHARED)
    }
    /// Wraps [`libc::pthread_mutexattr_setpshared`].
    pub fn set_shared(&mut self, shared: bool) -> Result<()> {
        let shared = if shared { libc::PTHREAD_PROCESS_SHARED} else { libc::PTHREAD_PROCESS_PRIVATE };
        unsafe {
            Errno::result(libc::pthread_mutexattr_setpshared(&mut self.0,shared)).map(drop)
        }
    }
    /// Wraps [`libc::pthread_mutexattr_getrobust`].
    pub fn get_robust(&self) -> Result<bool> {
        let init = unsafe {
            let mut uninit = std::mem::MaybeUninit::uninit();
            Errno::result(libc::pthread_mutexattr_getrobust(&self.0,uninit.as_mut_ptr()))?;
            uninit.assume_init()
        };
        Ok(init == libc::PTHREAD_MUTEX_ROBUST)
    }
    /// Wraps [`libc::pthread_mutexattr_setrobust`].
    pub fn set_robust(&mut self, robust: bool) -> Result<()> {
        let robust = if robust { libc::PTHREAD_MUTEX_ROBUST} else { libc::PTHREAD_MUTEX_STALLED };
        unsafe {
            Errno::result(libc::pthread_mutexattr_setrobust(&mut self.0,robust)).map(drop)
        }
    }
    /// Wraps [`libc::pthread_mutexattr_getprotocol`].
    pub fn get_protocol(&self) -> Result<Protocol> {
        let init = unsafe {
            let mut uninit = std::mem::MaybeUninit::uninit();
            Errno::result(libc::pthread_mutexattr_getprotocol(&self.0,uninit.as_mut_ptr()))?;
            uninit.assume_init()
        };
        Ok(Protocol::from(init))
    }
    /// Wraps [`libc::pthread_mutexattr_setprotocol`].
    pub fn set_protocol(&mut self, protocol: Protocol) -> Result<()> {
        unsafe {
            Errno::result(libc::pthread_mutexattr_setprotocol(&mut self.0,protocol as i32)).map(drop)
        }
    }
}

#[cfg(target_os = "linux")]
impl std::ops::Drop for MutexAttr {
    /// Wraps [`libc::pthread_mutexattr_destroy`].
    fn drop(&mut self) {
        unsafe {
            Errno::result(libc::pthread_mutexattr_destroy(&mut self.0)).unwrap();
        }
    }
}

/// Pthread Mutex.
/// 
/// ### Getting started
/// ```
/// # use std::{
/// #   sync::Arc,
/// #   time::{Instant, Duration},
/// #   thread::{sleep, spawn},
/// #   mem::size_of,
/// #   num::NonZeroUsize,
/// #   os::unix::io::OwnedFd
/// # };
/// # use nix::{
/// #   sys::{pthread::{Mutex, MutexAttr}, mman::{mmap, MapFlags, ProtFlags}},
/// #   unistd::{fork,ForkResult},
/// # };
/// const TIMEOUT: Duration = Duration::from_millis(500);
/// const DELTA: Duration = Duration::from_millis(100);
/// # fn main() -> nix::Result<()> {
/// let mutex = Mutex::new(None)?;
/// 
/// // The mutex is initialized unlocked, so an attempt to unlock it should
/// // return immediately.
/// unsafe { mutex.unlock()? };
/// // The mutex is unlocked, so `try_lock` will lock.
/// let guard = mutex.try_lock()?.unwrap();
/// // Unlock the mutex.
/// drop(guard);
/// // The mutex is unlocked, so `lock` will lock and exit immediately.
/// let guard = mutex.lock()?;
/// // Unlock the mutex.
/// guard.try_unlock()?;
/// # Ok(())
/// # }
/// ```
/// 
/// ### Multi-thread
/// 
/// ```
/// # use std::{
/// #   sync::Arc,
/// #   time::{Instant, Duration},
/// #   thread::{sleep, spawn},
/// #   mem::size_of,
/// #   num::NonZeroUsize,
/// #   os::unix::io::OwnedFd
/// # };
/// # use nix::{
/// #   sys::{pthread::{Mutex, MutexAttr}, mman::{mmap, MapFlags, ProtFlags}},
/// #   unistd::{fork,ForkResult},
/// # };
/// const TIMEOUT: Duration = Duration::from_millis(500);
/// const DELTA: Duration = Duration::from_millis(100);
/// # fn main() -> nix::Result<()> {
/// let mutex = Mutex::new(None)?;
/// let mutex_arc = Arc::new(mutex);
/// let mutex_clone = mutex_arc.clone();
/// let instant = Instant::now();
/// let handle = spawn(move || -> nix::Result<(),> {
///     let guard = mutex_clone.lock()?;
///     sleep(TIMEOUT);
///     guard.try_unlock()?;
///     Ok(())
/// });
/// sleep(DELTA);
/// let guard = mutex_arc.lock()?;
/// assert!(instant.elapsed() > TIMEOUT && instant.elapsed() < TIMEOUT + DELTA);
/// assert_eq!(handle.join().unwrap(), Ok(()));
/// # Ok(())
/// # }
/// ```
/// 
/// ### Multi-process
/// 
/// ```
/// # use std::{
/// #   sync::Arc,
/// #   time::{Instant, Duration},
/// #   thread::{sleep, spawn},
/// #   mem::{size_of, MaybeUninit},
/// #   num::NonZeroUsize,
/// #   os::unix::io::OwnedFd
/// # };
/// # use nix::{
/// #   sys::{pthread::{Mutex, MutexAttr}, mman::{mmap_anonymous, MapFlags, ProtFlags}},
/// #   unistd::{fork,ForkResult},
/// # };
/// const TIMEOUT: Duration = Duration::from_millis(500);
/// const DELTA: Duration = Duration::from_millis(100);
/// # fn main() -> nix::Result<()> {
/// let mutex = unsafe {
///     let mut ptr = mmap_anonymous(
///         None,
///         NonZeroUsize::new_unchecked(size_of::<Mutex>()),
///         ProtFlags::PROT_WRITE | ProtFlags::PROT_READ,
///         MapFlags::MAP_SHARED | MapFlags::MAP_ANONYMOUS,
///     )?.cast::<Mutex>();
/// 
///     // A mutex must be initialized.
///     // By default mutex's are process private, so we also need to initialize with the
///     // `MutexAttr` with shared.
///     let mut mutex_attr = MutexAttr::new()?;
///     mutex_attr.set_shared(true)?;
///     *ptr.as_mut() = Mutex::new(Some(mutex_attr))?;
///     ptr.as_ref()
/// };
/// 
/// match unsafe { fork()? } {
///     ForkResult::Parent { child } => {
///         let guard = mutex.lock()?;
///         sleep(TIMEOUT);
///         guard.try_unlock();
///         // Wait for child process to exit
///         unsafe {
///             assert_eq!(libc::waitpid(child.as_raw(), std::ptr::null_mut(),0), child.as_raw());
///         }
///     },
///     ForkResult::Child => {
///         let now = Instant::now();
///         sleep(DELTA);
///         mutex.lock()?;
///         assert!(now.elapsed() > TIMEOUT && now.elapsed() < TIMEOUT + DELTA);
///     }
/// }
/// 
/// # Ok(())
/// # }
/// ```
#[cfg(target_os = "linux")]
#[derive(Debug)]
#[repr(transparent)]
pub struct Mutex(UnsafeCell<libc::pthread_mutex_t>);

#[cfg(target_os = "linux")]
impl Mutex {
    /// Wraps [`libc::pthread_mutex_init`].
    /// 
    /// # Safety
    /// 
    /// Attempting to initialize an already initialized mutex results in undefined behavior.
    pub unsafe fn init(mutex: *mut Mutex, attr: Option<MutexAttr>) -> Result<()> {
        let attr = match attr {
            Some(mut x) => &mut x.0,
            None => std::ptr::null_mut()
        };
        unsafe {
            Errno::result(libc::pthread_mutex_init((*mutex).0.get(),attr))?;
        }
        
        Ok(())
    }
    /// Wraps [`libc::pthread_mutex_init`].
    pub fn new(attr: Option<MutexAttr>) -> Result<Self> {
        let attr = match attr {
            Some(mut x) => &mut x.0,
            None => std::ptr::null_mut()
        };
        let init = unsafe {
            let mut uninit = std::mem::MaybeUninit::<libc::pthread_mutex_t>::uninit();
            Errno::result(libc::pthread_mutex_init(uninit.as_mut_ptr(),attr))?;
            uninit.assume_init()
        };
        Ok(Self(UnsafeCell::new(init)))
    }
    /// Wraps [`libc::pthread_mutex_lock`].
    /// 
    /// <https://man7.org/linux/man-pages/man3/pthread_mutex_lock.3p.html>
    pub fn lock(&self) -> Result<MutexGuard<'_>> {
        unsafe {
            Errno::result(libc::pthread_mutex_lock(self.0.get())).map(|_| MutexGuard(self))
        }
    }
    /// Wraps [`libc::pthread_mutex_trylock`].
    /// 
    /// <https://man7.org/linux/man-pages/man3/pthread_mutex_lock.3p.html>
    pub fn try_lock(&self) -> Result<Option<MutexGuard<'_>>> {
        unsafe {
            match Errno::result(libc::pthread_mutex_trylock(self.0.get())) {
                Ok(_) => Ok(Some(MutexGuard(self))),
                Err(Errno::EBUSY) => Ok(None),
                Err(err) => Err(err)
            }
            
        }
    }
    /// Wraps [`libc::pthread_mutex_unlock`].
    /// 
    /// <https://man7.org/linux/man-pages/man3/pthread_mutex_lock.3p.html>
    /// 
    /// Prefer unlocking by dropping the [`MutexGuard`] returned by [`Mutex::lock`] or [`Mutex::try_lock`].
    /// 
    /// # Safety
    /// 
    /// Results in UB if not called from the thread that locked the mutex.
    pub unsafe fn unlock(&self) -> Result<()> {
        unsafe {
            Errno::result(libc::pthread_mutex_unlock(self.0.get())).map(drop)
        }
    }
}

#[cfg(target_os = "linux")]
unsafe impl Sync for Mutex {}

#[cfg(target_os = "linux")]
impl std::ops::Drop for Mutex {
    /// Wraps [`libc::pthread_mutex_destroy`].
    fn drop(&mut self) {
        let res = unsafe { libc::pthread_mutex_destroy(self.0.get()) };
        if !std::thread::panicking() {
            Errno::result(res).unwrap();
        }
    }
}

/// Mutex guard to prevent unlocking a mutex from a different thread than the thread that locked it.
#[cfg(target_os = "linux")]
#[derive(Debug)]
pub struct MutexGuard<'a>(&'a Mutex);

#[cfg(target_os = "linux")]
impl MutexGuard<'_> {
    /// Calls [`Mutex::unlock`].
    pub fn try_unlock(self) -> Result<()> {
        // Prevent calling `Self::Drop` which would attempt to unlock twice.
        unsafe { std::mem::ManuallyDrop::new(self).0.unlock() }
    }
}

#[cfg(target_os = "linux")]
impl std::ops::Drop for MutexGuard<'_> {
    /// Calls [`Mutex::unlock`].
    fn drop(&mut self) {
        let res = unsafe { self.0.unlock() };
        if !std::thread::panicking() {
            res.unwrap();
        }
    }
}