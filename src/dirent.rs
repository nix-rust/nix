//! Directory Stream functions
//!
//! [Further reading and details on the C API](http://man7.org/linux/man-pages/man3/opendir.3.html)

use {Result, Error, Errno, NixPath};
use errno;
use libc::{self, DIR, c_long};
use std::convert::{AsRef, Into};
use std::ffi::CStr;
use std::mem;

#[cfg(any(target_os = "linux"))]
use libc::ino64_t;

#[cfg(any(target_os = "android"))]
use libc::ino_t as ino64_t;

#[cfg(any(target_os = "linux", target_os = "android"))]
use libc::{dirent64, readdir64};

#[cfg(not(any(target_os = "linux", target_os = "android")))]
use libc::{dirent as dirent64, ino_t as ino64_t, readdir as readdir64};

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
use std::os::unix::io::RawFd;

/// Directory Stream object
pub struct DirectoryStream(*mut DIR);

impl AsRef<DIR> for DirectoryStream {
    fn as_ref(&self) -> &DIR {
        unsafe { &*self.0 }
    }
}

/// Consumes directory stream and return underlying directory pointer.
///
/// The pointer must be deallocated manually using `libc::closedir`
impl Into<*mut DIR> for DirectoryStream {
    fn into(self) -> *mut DIR {
        let dirp = self.0;
        mem::forget(self);
        dirp
    }
}

impl Drop for DirectoryStream {
    fn drop(&mut self) {
        unsafe { libc::closedir(self.0) };
    }
}

/// A directory entry
pub struct DirectoryEntry<'a>(&'a dirent64);

impl<'a> DirectoryEntry<'a> {
    /// File name
    pub fn name(&self) -> &CStr {
        unsafe{
            CStr::from_ptr(self.0.d_name.as_ptr())
        }
    }

    /// Inode number
    pub fn inode(&self) -> ino64_t {
        #[cfg(not(any(target_os = "freebsd", target_os = "netbsd", target_os="dragonfly")))]
        return self.0.d_ino;
        #[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os="dragonfly"))]
        return self.0.d_fileno;
    }
}

impl<'a> AsRef<dirent64> for DirectoryEntry<'a> {
    fn as_ref(&self) -> &dirent64 {
        self.0
    }
}

/// Opens a directory stream corresponding to the directory name.
///
/// The stream is positioned at the first entry in the directory.
pub fn opendir<P: ?Sized + NixPath>(name: &P) -> Result<DirectoryStream> {
    let dirp = try!(name.with_nix_path(|cstr| unsafe { libc::opendir(cstr.as_ptr()) }));
    if dirp.is_null() {
        Err(Error::last().into())
    } else {
        Ok(DirectoryStream(dirp))
    }
}

/// Returns directory stream corresponding to the open file descriptor `fd`
///
/// After a successful call to this function, `fd` is used internally by
/// the implementation, and should not otherwise be used by the application
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub fn fdopendir(fd: RawFd) -> Result<DirectoryStream> {
    let dirp = unsafe { libc::fdopendir(fd) };
    if dirp.is_null() {
        Err(Error::last().into())
    } else {
        Ok(DirectoryStream(dirp))
    }
}

/// Returns the next directory entry in the directory stream.
///
/// It returns `Some(None)` on reaching the end of the directory stream.
pub fn readdir<'a>(dir: &'a mut DirectoryStream) -> Result<Option<DirectoryEntry>> {
    let dirent = unsafe {
        Errno::clear();
        readdir64(dir.0)
    };
    if dirent.is_null() {
        match Errno::last() {
            errno::UnknownErrno => Ok(None),
            _ => Err(Error::last().into()),
        }
    } else {
        Ok(Some(DirectoryEntry(unsafe { &*dirent })))
    }
}

/// Sets the location in the directory stream from which the next `readdir` call will start.
///
/// The `loc` argument should be a value returned by a previous call to `telldir`
#[cfg(not(any(target_os = "android")))]
pub fn seekdir<'a>(dir: &'a mut DirectoryStream, loc: c_long) {
    unsafe { libc::seekdir(dir.0, loc) };
}

/// Returns the current location associated with the directory stream.
#[cfg(not(any(target_os = "android")))]
pub fn telldir<'a>(dir: &'a mut DirectoryStream) -> c_long {
    unsafe { libc::telldir(dir.0) }
}
