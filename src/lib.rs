//! Rust friendly bindings to the various *nix system functions.
//!
//! Modules are structured according to the C header file that they would be
//! defined in.
#![crate_name = "nix"]
#![cfg(unix)]
#![allow(non_camel_case_types)]
// latest bitflags triggers a rustc bug with cross-crate macro expansions causing dead_code
// warnings even though the macro expands into something with allow(dead_code)
#![allow(dead_code)]
#![cfg_attr(test, deny(warnings))]
#![recursion_limit = "500"]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate cfg_if;
extern crate void;

#[cfg(test)]
extern crate nix_test as nixtest;

#[macro_use] mod macros;

pub extern crate libc;

use errno::Errno;

pub mod errno;
pub mod features;
pub mod fcntl;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod mount;

#[cfg(target_os = "linux")]
pub mod mqueue;

pub mod pty;

pub mod poll;

pub mod net;

#[cfg(any(target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "linux",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "openbsd"))]
pub mod ifaddrs;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod sched;

pub mod sys;

// This can be implemented for other platforms as soon as libc
// provides bindings for them.
#[cfg(all(target_os = "linux",
          any(target_arch = "x86", target_arch = "x86_64")))]
pub mod ucontext;

pub mod unistd;

/*
 *
 * ===== Result / Error =====
 *
 */

use libc::c_char;
use std::{ptr, result};
use std::ffi::{CStr, CString, OsStr};
use std::path::{Path, PathBuf};
use std::os::unix::ffi::OsStrExt;
use std::fmt;
use std::error;
use libc::PATH_MAX;

/// Nix Result Type
pub type Result<T> = result::Result<T, Error>;

/// Nix Error Type
///
/// The nix error type provides a common way of dealing with
/// various system system/libc calls that might fail.  Each
/// error has a corresponding errno (usually the one from the
/// underlying OS) to which it can be mapped in addition to
/// implementing other common traits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Sys(errno::Errno),
    InvalidPath,
    /// The operation involved a conversion to Rust's native String type, which failed because the
    /// string did not contain all valid UTF-8.
    InvalidUtf8,
    /// The operation is not supported by Nix, in this instance either use the libc bindings or
    /// consult the module documentation to see if there is a more appropriate interface available.
    UnsupportedOperation,
}

impl Error {

    /// Create a nix Error from a given errno
    pub fn from_errno(errno: Errno) -> Error {
        Error::Sys(errno)
    }

    /// Get the current errno and convert it to a nix Error
    pub fn last() -> Error {
        Error::Sys(Errno::last())
    }

    /// Create a new invalid argument error (`EINVAL`)
    pub fn invalid_argument() -> Error {
        Error::Sys(Errno::EINVAL)
    }

}

impl From<Errno> for Error {
    fn from(errno: Errno) -> Error { Error::from_errno(errno) }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(_: std::string::FromUtf8Error) -> Error { Error::InvalidUtf8 }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::InvalidPath => "Invalid path",
            &Error::InvalidUtf8 => "Invalid UTF-8 string",
            &Error::UnsupportedOperation => "Unsupported Operation",
            &Error::Sys(ref errno) => errno.desc(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::InvalidPath => write!(f, "Invalid path"),
            &Error::InvalidUtf8 => write!(f, "Invalid UTF-8 string"),
            &Error::UnsupportedOperation => write!(f, "Unsupported Operation"),
            &Error::Sys(errno) => write!(f, "{:?}: {}", errno, errno.desc()),
        }
    }
}

pub trait NixPath {
    fn len(&self) -> usize;

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
        where F: FnOnce(&CStr) -> T;
}

impl NixPath for str {
    fn len(&self) -> usize {
        NixPath::len(OsStr::new(self))
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
        where F: FnOnce(&CStr) -> T {
            OsStr::new(self).with_nix_path(f)
        }
}

impl NixPath for OsStr {
    fn len(&self) -> usize {
        self.as_bytes().len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
        where F: FnOnce(&CStr) -> T {
            self.as_bytes().with_nix_path(f)
        }
}

impl NixPath for CStr {
    fn len(&self) -> usize {
        self.to_bytes().len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
            where F: FnOnce(&CStr) -> T {
        // Equivalence with the [u8] impl.
        if self.len() >= PATH_MAX as usize {
            return Err(Error::InvalidPath);
        }

        Ok(f(self))
    }
}

impl NixPath for CString {
    fn len(&self) -> usize {
        self.to_bytes().len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
            where F: FnOnce(&CStr) -> T {
        self.as_c_str().with_nix_path(f)
    }
}

impl NixPath for [u8] {
    fn len(&self) -> usize {
        self.len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
            where F: FnOnce(&CStr) -> T {
        let mut buf = [0u8; PATH_MAX as usize];

        if self.len() >= PATH_MAX as usize {
            return Err(Error::InvalidPath);
        }

        match self.iter().position(|b| *b == 0) {
            Some(_) => Err(Error::InvalidPath),
            None => {
                unsafe {
                    // TODO: Replace with bytes::copy_memory. rust-lang/rust#24028
                    ptr::copy_nonoverlapping(self.as_ptr(), buf.as_mut_ptr(), self.len());
                    Ok(f(CStr::from_ptr(buf.as_ptr() as *const c_char)))
                }

            }
        }
    }
}

impl NixPath for Path {
    fn len(&self) -> usize {
        NixPath::len(self.as_os_str())
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T> where F: FnOnce(&CStr) -> T {
        self.as_os_str().with_nix_path(f)
    }
}

impl NixPath for PathBuf {
    fn len(&self) -> usize {
        NixPath::len(self.as_os_str())
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T> where F: FnOnce(&CStr) -> T {
        self.as_os_str().with_nix_path(f)
    }
}

/// Treats `None` as an empty string.
impl<'a, NP: ?Sized + NixPath>  NixPath for Option<&'a NP> {
    fn len(&self) -> usize {
        self.map_or(0, NixPath::len)
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T> where F: FnOnce(&CStr) -> T {
        if let Some(nix_path) = *self {
            nix_path.with_nix_path(f)
        } else {
            unsafe { CStr::from_ptr("\0".as_ptr() as *const _).with_nix_path(f) }
        }
    }
}

/*
 *
 * ===== Null terminated slices for exec =====
 *
 */

use std::ops::{Deref, DerefMut};
use std::mem::transmute;
use std::iter;

/// A conversion trait that may borrow or allocate memory depending on the input.
/// Used to convert between terminated slices and `Vec`s.
pub trait IntoRef<'a, T: ?Sized> {
    type Target: 'a + AsRef<T> + Deref<Target=T>;

    fn into_ref(self) -> Self::Target;
}

/// A slice of references terminated by `None`. Used by API calls that accept
/// null-terminated arrays such as the `exec` family of functions.
pub struct TerminatedSlice<T> {
    inner: [Option<T>],
}

impl<T> TerminatedSlice<T> {
    /// Instantiate a `TerminatedSlice` from a slice ending in `None`. Returns
    /// `None` if the provided slice is not properly terminated.
    pub fn from_slice(slice: &[Option<T>]) -> Option<&Self> {
        if slice.last().map(Option::is_none).unwrap_or(false) {
            Some(unsafe { Self::from_slice_unchecked(slice) })
        } else {
            None
        }
    }

    /// Instantiate a `TerminatedSlice` from a mutable slice ending in `None`.
    /// Returns `None` if the provided slice is not properly terminated.
    pub fn from_slice_mut(slice: &mut [Option<T>]) -> Option<&mut Self> {
        if slice.last().map(Option::is_none).unwrap_or(false) {
            Some(unsafe { Self::from_slice_mut_unchecked(slice) })
        } else {
            None
        }
    }

    /// Instantiate a `TerminatedSlice` from a slice ending in `None`.
    ///
    /// ## Unsafety
    ///
    /// This assumes that the slice is properly terminated, and can cause
    /// undefined behaviour if that invariant is not upheld.
    pub unsafe fn from_slice_unchecked(slice: &[Option<T>]) -> &Self {
        transmute(slice)
    }

    /// Instantiate a `TerminatedSlice` from a mutable slice ending in `None`.
    ///
    /// ## Unsafety
    ///
    /// This assumes that the slice is properly terminated, and can cause
    /// undefined behaviour if that invariant is not upheld.
    pub unsafe fn from_slice_mut_unchecked(slice: &mut [Option<T>]) -> &mut Self {
        transmute(slice)
    }
}

impl<'a, U: Sized> TerminatedSlice<&'a U> {
    pub fn as_ptr(&self) -> *const *const U {
        self.inner.as_ptr() as *const _
    }
}

impl<T> Deref for TerminatedSlice<T> {
    type Target = [Option<T>];

    fn deref(&self) -> &Self::Target {
        &self.inner[..self.inner.len() - 1]
    }
}

impl<T> DerefMut for TerminatedSlice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.inner.len();
        &mut self.inner[..len - 1]
    }
}

impl<T> AsRef<TerminatedSlice<T>> for TerminatedSlice<T> {
    fn as_ref(&self) -> &Self {
        self
    }
}

/// Owned variant of `TerminatedSlice`.
pub struct TerminatedVec<T> {
    inner: Vec<Option<T>>,
}

impl<T> TerminatedVec<T> {
    /// Instantiates a `TerminatedVec` from a `None` terminated `Vec`. Returns
    /// `None` if the provided `Vec` is not properly terminated.
    pub fn from_vec(vec: Vec<Option<T>>) -> Option<Self> {
        if vec.last().map(Option::is_none).unwrap_or(false) {
            Some(unsafe { Self::from_vec_unchecked(vec) })
        } else {
            None
        }
    }

    /// Instantiates a `TerminatedVec` from a `None` terminated `Vec`.
    ///
    /// ## Unsafety
    ///
    /// This assumes that the `Vec` is properly terminated, and can cause
    /// undefined behaviour if that invariant is not upheld.
    pub unsafe fn from_vec_unchecked(vec: Vec<Option<T>>) -> Self {
        TerminatedVec {
            inner: vec,
        }
    }

    /// Consume `self` to return the inner wrapped `Vec`.
    pub fn into_inner(self) -> Vec<Option<T>> {
        self.inner
    }
}

impl<'a> TerminatedVec<&'a c_char> {
    fn terminate<T: AsRef<CStr> + 'a, I: IntoIterator<Item=T>>(iter: I) -> Self {
        fn cstr_char<'a, S: AsRef<CStr> + 'a>(s: S) -> &'a c_char {
            unsafe {
                &*s.as_ref().as_ptr()
            }
        }

        let terminated = iter.into_iter()
            .map(cstr_char)
            .map(Some).chain(iter::once(None)).collect();

        unsafe {
            TerminatedVec::from_vec_unchecked(terminated)
        }
    }
}

impl<T> Deref for TerminatedVec<T> {
    type Target = TerminatedSlice<T>;

    fn deref(&self) -> &Self::Target {
        unsafe {
            TerminatedSlice::from_slice_unchecked(&self.inner)
        }
    }
}

impl<T> DerefMut for TerminatedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            TerminatedSlice::from_slice_mut_unchecked(&mut self.inner)
        }
    }
}

impl<T> AsRef<TerminatedSlice<T>> for TerminatedVec<T> {
    fn as_ref(&self) -> &TerminatedSlice<T> {
        self
    }
}

impl<'a, T: 'a> IntoRef<'a, TerminatedSlice<&'a T>> for &'a TerminatedSlice<&'a T> {
    type Target = &'a TerminatedSlice<&'a T>;

    fn into_ref(self) -> Self::Target {
        self
    }
}

impl<'a, T: AsRef<CStr> + 'a, I: IntoIterator<Item=T>> IntoRef<'a, TerminatedSlice<&'a c_char>> for I {
    type Target = TerminatedVec<&'a c_char>;

    fn into_ref(self) -> Self::Target {
        TerminatedVec::terminate(self)
    }
}
