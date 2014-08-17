#![crate_name = "nix"]
#![feature(globs)]

extern crate libc;

pub use errno::{SysResult, SysError};

pub mod errno;
pub mod features;
pub mod fcntl;
pub mod mount;
pub mod sched;
pub mod sys;
pub mod syscall;
pub mod unistd;
