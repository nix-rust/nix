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
#![deny(warnings)]

#[macro_use]
extern crate bitflags;

extern crate libc;

#[cfg(test)]
extern crate nix_test as nixtest;

// Re-export some libc constants
pub use libc::{c_int, c_void};

pub mod errno;
pub mod features;
pub mod fcntl;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod mount;

#[cfg(any(target_os = "linux"))]
pub mod mqueue;

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
use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::os::unix::ffi::OsStrExt;

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
    fn len(&self) -> usize;

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
        where F: FnOnce(&CStr) -> T;
}

impl NixPath for [u8] {
    fn len(&self) -> usize {
        self.len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
            where F: FnOnce(&CStr) -> T {
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
                    Ok(f(CStr::from_ptr(buf.as_ptr() as *const c_char)))
                }

            }
        }
    }
}

impl NixPath for Path {
    fn len(&self) -> usize {
        self.as_os_str().as_bytes().len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T> where F: FnOnce(&CStr) -> T {
        self.as_os_str().as_bytes().with_nix_path(f)
    }
}

impl NixPath for PathBuf {
    fn len(&self) -> usize {
        self.as_os_str().as_bytes().len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T> where F: FnOnce(&CStr) -> T {
        self.as_os_str().as_bytes().with_nix_path(f)
    }
}

#[inline]
pub fn from_ffi(res: libc::c_int) -> Result<()> {
    if res != 0 {
        return Err(Error::Sys(errno::Errno::last()));
    }

    Ok(())
}
