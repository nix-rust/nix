//! Rust friendly bindings to the various *nix system functions.
//!
//! Modules are structured according to the C header file that they would be
//! defined in.
#![crate_name = "nix"]
#![cfg(unix)]
#![allow(non_camel_case_types)]
#![deny(warnings)]

#[macro_use]
extern crate bitflags;

extern crate libc;

#[cfg(test)]
extern crate nix_test as nixtest;

// Re-export some libc constants
pub use libc::{c_int, c_void};

#[cfg(unix)]
pub mod errno;

#[cfg(unix)]
pub mod features;

#[cfg(unix)]
pub mod fcntl;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod mount;

#[cfg(any(target_os = "linux"))]
pub mod mq;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod sched;

#[cfg(unix)]
pub mod sys;

#[cfg(unix)]
pub mod unistd;

/*
 *
 * ===== Result / Error =====
 *
 */

use std::{ptr, result};
use std::path::{Path, PathBuf};

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

pub trait NixPath {
    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
        where F: FnOnce(&OsStr) -> T;
}

impl NixPath for [u8] {
    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
            where F: FnOnce(&OsStr) -> T {
        // TODO: Extract this size as a const
        let mut buf = [0u8; 4096];

        if self.len() >= 4096 {
            return Err(Error::InvalidPath);
        }

        match self.iter().position(|b| *b == 0) {
            Some(_) => Err(Error::InvalidPath),
            None => {
                unsafe {
                    // TODO: Replace with bytes::copy_memory. rust-lang/rust#24028
                    ptr::copy_nonoverlapping(self.as_ptr(), buf.as_mut_ptr(), self.len());
                }

                Ok(f(<OsStr as OsStrExt>::from_bytes(&buf[..self.len()])))
            }
        }
    }
}

impl NixPath for Path {
    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
            where F: FnOnce(&OsStr) -> T {
        Ok(f(self.as_os_str()))
    }
}

impl NixPath for PathBuf {
    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
            where F: FnOnce(&OsStr) -> T {
        Ok(f(self.as_os_str()))
    }
}

#[inline]
pub fn from_ffi(res: libc::c_int) -> Result<()> {
    if res != 0 {
        return Err(Error::Sys(errno::Errno::last()));
    }

    Ok(())
}

/*
 *
 * ===== Impl utilities =====
 *
 */

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

/// Converts a value to an external (FFI) string representation
trait AsExtStr {
    fn as_ext_str(&self) -> *const libc::c_char;
}

impl AsExtStr for OsStr {
    fn as_ext_str(&self) -> *const libc::c_char {
        self.as_bytes().as_ptr() as *const libc::c_char
    }
}
