#![crate_name = "nix"]

#![feature(globs)]
#![feature(linkage)]
#![allow(non_camel_case_types)]

extern crate libc;

// Re-export some libc constants
pub use libc::{c_int, c_void};

#[cfg(unix)]
pub use errno::{SysResult, SysError};

#[cfg(unix)]
pub mod errno;

#[cfg(unix)]
pub mod features;

#[cfg(unix)]
pub mod fcntl;

#[cfg(target_os = "linux")]
pub mod mount;

#[cfg(target_os = "linux")]
pub mod sched;

#[cfg(unix)]
pub mod sys;

#[cfg(target_os = "linux")]
pub mod syscall;

#[cfg(unix)]
pub mod unistd;
