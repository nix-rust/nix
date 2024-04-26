use crate::errno::Errno;
use crate::Result;
use libc::eventfd_t;
use std::mem::MaybeUninit;
use std::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};

libc_bitflags! {
    pub struct EfdFlags: libc::c_int {
        EFD_CLOEXEC; // Since Linux 2.6.27/FreeBSD 13.0
        EFD_NONBLOCK; // Since Linux 2.6.27/FreeBSD 13.0
        EFD_SEMAPHORE; // Since Linux 2.6.30/FreeBSD 13.0
    }
}

#[deprecated(since = "0.28.0", note = "Use EventFd::from_value_and_flags() instead")]
pub fn eventfd(initval: libc::c_uint, flags: EfdFlags) -> Result<OwnedFd> {
    let res = unsafe { libc::eventfd(initval, flags.bits()) };

    Errno::result(res).map(|r| unsafe { OwnedFd::from_raw_fd(r) })
}

#[derive(Debug)]
#[repr(transparent)]
pub struct EventFd(OwnedFd);
impl EventFd {
    /// [`EventFd::from_value_and_flags`] with `init_val = 0` and `flags = EfdFlags::empty()`.
    pub fn new() -> Result<Self> {
        Self::from_value_and_flags(0, EfdFlags::empty())
    }
    /// Constructs [`EventFd`] with the given `init_val` and `flags`.
    /// 
    /// Wrapper around [`libc::eventfd`].
    pub fn from_value_and_flags(init_val: u32, flags: EfdFlags) -> Result<Self> {
        let res = unsafe { libc::eventfd(init_val, flags.bits()) };
        Errno::result(res).map(|r| Self(unsafe { OwnedFd::from_raw_fd(r) }))
    }
    /// [`EventFd::from_value_and_flags`] with `init_val = 0` and given `flags`.
    pub fn from_flags(flags: EfdFlags) -> Result<Self> {
        Self::from_value_and_flags(0, flags)
    }
    /// [`EventFd::from_value_and_flags`] with given `init_val` and `flags = EfdFlags::empty()`.
    pub fn from_value(init_val: u32) -> Result<Self> {
        Self::from_value_and_flags(init_val, EfdFlags::empty())
    }
    /// Arms `self`, a following call to `poll`, `select` or `epoll` will return immediately.
    /// 
    /// [`EventFd::write`] with `1`.
    pub fn arm(&self) -> Result<usize> {
        self.write(1)
    }
    /// Defuses `self`, a following call to `poll`, `select` or `epoll` will block.
    /// 
    /// [`EventFd::write`] with `0`.
    pub fn defuse(&self) -> Result<usize> {
        self.write(0)
    }
    /// Enqueues `value` triggers.
    /// 
    /// The next `value` calls to `poll`, `select` or `epoll` will return immediately.
    /// 
    /// [`EventFd::write`] with `value`.
    pub fn write(&self, value: eventfd_t) -> Result<usize> {
        let res = unsafe { libc::eventfd_write(self.0.as_raw_fd(), value) };
        Errno::result(res).map(|res| res as usize)
    }
    // Reads the value from the file descriptor.
    pub fn read(&self) -> Result<eventfd_t> {
        let mut buf = MaybeUninit::uninit();
        let res =
            unsafe { libc::eventfd_read(self.0.as_raw_fd(), buf.as_mut_ptr()) };
        Errno::result(res)?;

        // SAFETY:
        // It is guaranteed to be initialized by the `eventfd_read()` function.
        Ok(unsafe { buf.assume_init() })
    }
}
impl AsFd for EventFd {
    fn as_fd(&self) -> BorrowedFd {
        self.0.as_fd()
    }
}
impl AsRawFd for EventFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}
impl From<EventFd> for OwnedFd {
    fn from(x: EventFd) -> OwnedFd {
        x.0
    }
}
