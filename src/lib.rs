#![crate_name = "linux"]
#![feature(globs)]

extern crate libc;

pub use errno::{SysResult, SysError};

pub mod errno;
pub mod mount;
pub mod sched;
pub mod syscall;
pub mod unistd;
