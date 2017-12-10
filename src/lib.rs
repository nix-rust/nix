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
use std::{iter, io};

/// An error returned from the [`TerminatedSlice::from_slice`] family of
/// functions when the provided data is not terminated by `None`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NotTerminatedError {
    // private initializers only
    _inner: (),
}

impl NotTerminatedError {
    fn not_terminated() -> Self {
        NotTerminatedError {
            _inner: (),
        }
    }
}

impl error::Error for NotTerminatedError {
    fn description(&self) -> &str { "data not terminated by None" }
}

impl fmt::Display for NotTerminatedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl From<NotTerminatedError> for io::Error {
    fn from(e: NotTerminatedError) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidInput,
                       error::Error::description(&e))
    }
}

/// An error returned from [`TerminatedVec::from_vec`] when the provided data is
/// not terminated by `None`.
#[derive(Clone, PartialEq, Eq)]
pub struct NotTerminatedVecError<T> {
    inner: Vec<Option<T>>,
}

impl<T> NotTerminatedVecError<T> {
    fn not_terminated(vec: Vec<Option<T>>) -> Self {
        NotTerminatedVecError {
            inner: vec,
        }
    }

    pub fn into_vec(self) -> Vec<Option<T>> {
        self.inner
    }
}

impl<T> fmt::Debug for NotTerminatedVecError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NotTerminatedVecError").finish()
    }
}

impl<T> error::Error for NotTerminatedVecError<T> {
    fn description(&self) -> &str { "data not terminated by None" }
}

impl<T> fmt::Display for NotTerminatedVecError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl<T> From<NotTerminatedVecError<T>> for io::Error {
    fn from(e: NotTerminatedVecError<T>) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidInput,
                       error::Error::description(&e))
    }
}

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
    /// Instantiate a `TerminatedSlice` from a slice ending in `None`. Fails if
    /// the provided slice is not properly terminated.
    pub fn from_slice(slice: &[Option<T>]) -> result::Result<&Self, NotTerminatedError> {
        if slice.last().map(Option::is_none).unwrap_or(false) {
            Ok(unsafe { Self::from_slice_unchecked(slice) })
        } else {
            Err(NotTerminatedError::not_terminated())
        }
    }

    /// Instantiate a `TerminatedSlice` from a mutable slice ending in `None`.
    /// Fails if the provided slice is not properly terminated.
    pub fn from_slice_mut(slice: &mut [Option<T>]) -> result::Result<&mut Self, NotTerminatedError> {
        if slice.last().map(Option::is_none).unwrap_or(false) {
            Ok(unsafe { Self::from_slice_mut_unchecked(slice) })
        } else {
            Err(NotTerminatedError::not_terminated())
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

/// Coercion into an argument that can be passed to `exec`.
impl<'a, T: 'a> IntoRef<'a, TerminatedSlice<&'a T>> for &'a TerminatedSlice<&'a T> {
    type Target = &'a TerminatedSlice<&'a T>;

    fn into_ref(self) -> Self::Target {
        self
    }
}

/// A `Vec` of references terminated by `None`. Used by API calls that accept
/// null-terminated arrays such as the `exec` family of functions. Owned variant
/// of [`TerminatedSlice`].
pub struct TerminatedVec<T> {
    inner: Vec<Option<T>>,
}

impl<T> TerminatedVec<T> {
    /// Instantiates a `TerminatedVec` from a `None` terminated `Vec`. Fails if
    /// the provided `Vec` is not properly terminated.
    pub fn from_vec(vec: Vec<Option<T>>) -> result::Result<Self, NotTerminatedVecError<T>> {
        if vec.last().map(Option::is_none).unwrap_or(false) {
            Ok(unsafe { Self::from_vec_unchecked(vec) })
        } else {
            Err(NotTerminatedVecError::not_terminated(vec))
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

    /// Converts an iterator into a `None` terminated `Vec`.
    pub fn terminate<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let terminated = iter.into_iter()
            .map(Some).chain(iter::once(None)).collect();

        unsafe {
            Self::from_vec_unchecked(terminated)
        }
    }
}

impl<'a> TerminatedVec<&'a c_char> {
    /// Converts an iterator of `AsRef<CStr>` into a `TerminatedVec` to
    /// be used by the `exec` functions. This allows for preallocation of the
    /// array when memory allocation could otherwise cause problems (such as
    /// when combined with `fork`).
    ///
    /// ```
    /// use std::iter;
    /// use std::ffi::CString;
    /// use nix::{TerminatedVec, unistd};
    /// use nix::sys::wait;
    ///
    /// # #[cfg(target_os = "android")]
    /// # fn exe_path() -> CString {
    /// # CString::new("/system/bin/sh").unwrap()
    /// # }
    /// # #[cfg(not(target_os = "android"))]
    /// # fn exe_path() -> CString {
    /// let exe = CString::new("/bin/sh").unwrap();
    /// #    exe
    /// # }
    /// # let exe = exe_path();
    /// let args = [
    ///     exe.clone(),
    ///     CString::new("-c").unwrap(),
    ///     CString::new("echo hi").unwrap(),
    /// ];
    /// let args_p = TerminatedVec::terminate_cstr(&args);
    /// let env = TerminatedVec::terminate(iter::empty());
    ///
    /// match unistd::fork().unwrap() {
    ///     unistd::ForkResult::Child => {
    ///         unistd::execve(exe.as_c_str(), &args_p, env).unwrap();
    ///     },
    ///     unistd::ForkResult::Parent { child } => {
    ///         let status = wait::waitpid(child, None).unwrap();
    ///         assert_eq!(status, wait::WaitStatus::Exited(child, 0));
    ///     },
    /// }
    /// ```
    pub fn terminate_cstr<T: AsRef<CStr> + 'a, I: IntoIterator<Item=T>>(iter: I) -> Self {
        fn cstr_char<'a, S: AsRef<CStr> + 'a>(s: S) -> &'a c_char {
            unsafe {
                &*s.as_ref().as_ptr()
            }
        }

        Self::terminate(iter.into_iter().map(cstr_char))
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

impl<'a, T: 'a> IntoRef<'a, TerminatedSlice<T>> for TerminatedVec<T> {
    type Target = TerminatedVec<T>;

    fn into_ref(self) -> Self::Target {
        self
    }
}

impl<'a, T> IntoRef<'a, TerminatedSlice<T>> for &'a TerminatedVec<T> {
    type Target = &'a TerminatedSlice<T>;

    fn into_ref(self) -> Self::Target {
        self
    }
}

/// Coercion of `CStr` iterators into an argument that can be passed to `exec`.
impl<'a, T: AsRef<CStr> + 'a, I: IntoIterator<Item=T>> IntoRef<'a, TerminatedSlice<&'a c_char>> for I {
    type Target = TerminatedVec<&'a c_char>;

    fn into_ref(self) -> Self::Target {
        TerminatedVec::terminate_cstr(self)
    }
}
