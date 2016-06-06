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

#[macro_use]
extern crate bitflags;

pub extern crate libc;

#[cfg(test)]
extern crate nix_test as nixtest;

// Re-exports
pub use libc::{c_int, c_void};
pub use errno::Errno;

pub mod errno;
pub mod features;
pub mod fcntl;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod mount;

#[cfg(any(target_os = "linux"))]
pub mod mqueue;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub mod poll;

pub mod net;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod sched;

pub mod sys;
pub mod unistd;

/*
 *
 * ===== Result / Error =====
 *
 */

use libc::c_char;
use std::{ptr, result};
use std::ffi::{CStr, OsStr};
use std::path::{Path, PathBuf};
use std::os::unix::ffi::OsStrExt;
use std::io;
use std::fmt;
use std::error;
use libc::PATH_MAX;

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Sys(errno::Errno),
    InvalidPath,
}

impl Error {
    pub fn from_errno(errno: errno::Errno) -> Error {
        Error::Sys(errno)
    }

    pub fn last() -> Error {
        Error::Sys(errno::Errno::last())
    }

    pub fn invalid_argument() -> Error {
        Error::Sys(errno::EINVAL)
    }

    pub fn errno(&self) -> errno::Errno {
        match *self {
            Error::Sys(errno) => errno,
            Error::InvalidPath => errno::Errno::EINVAL,
        }
    }
}

impl From<errno::Errno> for Error {
    fn from(errno: errno::Errno) -> Error { Error::from_errno(errno) }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::InvalidPath => "Invalid path",
            &Error::Sys(ref errno) => errno.desc(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::InvalidPath => write!(f, "Invalid path"),
            &Error::Sys(errno) => write!(f, "{:?}: {}", errno, errno.desc()),
        }
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::InvalidPath => io::Error::new(io::ErrorKind::InvalidInput, err),
            Error::Sys(errno) => io::Error::from_raw_os_error(errno as i32),
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
