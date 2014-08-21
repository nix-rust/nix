#![crate_name = "nix"]
#![feature(globs)]

extern crate libc;

pub use errno::{SysResult, SysError};

#[cfg(target_os = "linux")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "ios")]
pub mod errno;

#[cfg(target_os = "linux")]
pub mod features;

#[cfg(target_os = "linux")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "ios")]
pub mod fcntl;

#[cfg(target_os = "linux")]
pub mod mount;

#[cfg(target_os = "linux")]
pub mod sched;

#[cfg(target_os = "linux")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "ios")]
pub mod sys;

#[cfg(target_os = "linux")]
pub mod syscall;

#[cfg(target_os = "linux")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "ios")]
pub mod unistd;
