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
