//! Rust friendly bindings to the various *nix system functions.
//!
//! Modules are structured according to the C header file that they would be
//! defined in.
#![crate_name = "nix"]

#![feature(collections, core, net, linkage, libc, os, path, std_misc)]
#![allow(non_camel_case_types)]

#[macro_use]
extern crate bitflags;

extern crate libc;
extern crate core;

#[cfg(test)]
extern crate "nix-test" as nixtest;

// Re-export some libc constants
pub use libc::{c_int, c_void};

mod nix;
pub use nix::{NixResult, NixError, NixPath, from_ffi};


#[cfg(unix)]
pub mod errno;

#[cfg(unix)]
pub mod features;

#[cfg(unix)]
pub mod fcntl;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod mount;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod sched;

#[cfg(unix)]
pub mod sys;

#[cfg(unix)]
pub mod unistd;

/*
 *
 * ===== Impl utilities =====
 *
 */

use std::ffi::OsStr;
use std::os::unix::OsStrExt;

/// Converts a value to an external (FFI) string representation
trait AsExtStr {
    fn as_ext_str(&self) -> *const libc::c_char;
}

impl AsExtStr for OsStr {
    fn as_ext_str(&self) -> *const libc::c_char {
        self.as_bytes().as_ptr() as *const libc::c_char
    }
}
