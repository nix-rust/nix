//! Create master and slave virtual pseudo-terminals (PTYs)

use std::ffi::CStr;
use std::mem;
use std::os::unix::prelude::*;

use libc;

use {Error, fcntl, Result};

/// Representation of the Master device in a master/slave pty pair
///
/// While this datatype is a thin wrapper around `RawFd`, it enforces that the available PTY
/// functions are given the correct file descriptor. Additionally this type implements `Drop`,
/// so that when it's consumed or goes out of scope, it's automatically cleaned-up.
#[derive(Debug)]
pub struct PtyMaster(RawFd);

impl AsRawFd for PtyMaster {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl IntoRawFd for PtyMaster {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.0;
        mem::forget(self);
        fd
    }
}

impl Drop for PtyMaster {
    fn drop(&mut self) {
        // Errors when closing are ignored because we don't actually know if the file descriptor
        // was closed. If we retried, it's possible that descriptor was reallocated in the mean
        // time and the wrong file descriptor could be closed.
        let _ = ::unistd::close(self.0);
    }
}

/// Grant access to a slave pseudoterminal (see
/// [grantpt(3)](http://man7.org/linux/man-pages/man3/grantpt.3.html))
///
/// `grantpt()` changes the mode and owner of the slave pseudoterminal device corresponding to the
/// master pseudoterminal referred to by `fd`. This is a necessary step towards opening the slave.
#[inline]
pub fn grantpt(fd: &PtyMaster) -> Result<()> {
    if unsafe { libc::grantpt(fd.as_raw_fd()) } < 0 {
        return Err(Error::last().into());
    }

    Ok(())
}

/// Open a pseudoterminal device (see
/// [posix_openpt(3)](http://man7.org/linux/man-pages/man3/posix_openpt.3.html))
///
/// `posix_openpt()` returns a file descriptor to an existing unused pseuterminal master device.
///
/// # Examples
///
/// A common use case with this function is to open both a master and slave PTY pair. This can be
/// done as follows:
///
/// ```
/// use std::path::Path;
/// use nix::fcntl::{O_RDWR, open};
/// use nix::pty::*;
/// use nix::sys::stat;
///
/// # #[allow(dead_code)]
/// # fn run() -> nix::Result<()> {
/// // Open a new PTY master
/// let master_fd = posix_openpt(O_RDWR)?;
///
/// // Allow a slave to be generated for it
/// grantpt(&master_fd)?;
/// unlockpt(&master_fd)?;
///
/// // Get the name of the slave
/// let slave_name = ptsname(&master_fd)?;
///
/// // Try to open the slave
/// # #[allow(unused_variables)]
/// let slave_fd = open(Path::new(&slave_name), O_RDWR, stat::Mode::empty())?;
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn posix_openpt(flags: fcntl::OFlag) -> Result<PtyMaster> {
    let fd = unsafe {
        libc::posix_openpt(flags.bits())
    };

    if fd < 0 {
        return Err(Error::last().into());
    }

    Ok(PtyMaster(fd))
}

/// Get the name of the slave pseudoterminal (see
/// [ptsname(3)](http://man7.org/linux/man-pages/man3/ptsname.3.html))
///
/// `ptsname()` returns the name of the slave pseudoterminal device corresponding to the master
/// referred to by `fd`. Note that this function is *not* threadsafe. For that see `ptsname_r()`.
///
/// This value is useful for opening the slave pty once the master has already been opened with
/// `posix_openpt()`.
#[inline]
pub fn ptsname(fd: &PtyMaster) -> Result<String> {
    let name_ptr = unsafe { libc::ptsname(fd.as_raw_fd()) };
    if name_ptr.is_null() {
        return Err(Error::last().into());
    }

    let name = unsafe {
        CStr::from_ptr(name_ptr)
    };
    Ok(name.to_string_lossy().into_owned())
}

/// Get the name of the slave pseudoterminal (see
/// [ptsname(3)](http://man7.org/linux/man-pages/man3/ptsname.3.html))
///
/// `ptsname_r()` returns the name of the slave pseudoterminal device corresponding to the master
/// referred to by `fd`. This is the threadsafe version of `ptsname()`, but it is not part of the
/// POSIX standard and is instead a Linux-specific extension.
///
/// This value is useful for opening the slave ptty once the master has already been opened with
/// `posix_openpt()`.
#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub fn ptsname_r(fd: &PtyMaster) -> Result<String> {
    let mut name_buf = vec![0u8; 64];
    let name_buf_ptr = name_buf.as_mut_ptr() as *mut libc::c_char;
    if unsafe { libc::ptsname_r(fd.as_raw_fd(), name_buf_ptr, name_buf.capacity()) } != 0 {
        return Err(Error::last().into());
    }

    // Find the first null-character terminating this string. This is guaranteed to succeed if the
    // return value of `libc::ptsname_r` is 0.
    let null_index = name_buf.iter().position(|c| *c == b'\0').unwrap();
    name_buf.truncate(null_index);

    let name = String::from_utf8(name_buf)?;
    Ok(name)
}

/// Unlock a pseudoterminal master/slave pseudoterminal pair (see
/// [unlockpt(3)](http://man7.org/linux/man-pages/man3/unlockpt.3.html))
///
/// `unlockpt()` unlocks the slave pseudoterminal device corresponding to the master pseudoterminal
/// referred to by `fd`. This must be called before trying to open the slave side of a
/// pseuoterminal.
#[inline]
pub fn unlockpt(fd: &PtyMaster) -> Result<()> {
    if unsafe { libc::unlockpt(fd.as_raw_fd()) } < 0 {
        return Err(Error::last().into());
    }

    Ok(())
}
