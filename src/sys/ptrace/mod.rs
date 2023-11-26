//! Provides helpers for making ptrace system calls

#[cfg(linux_android)]
mod linux;

#[cfg(linux_android)]
pub use self::linux::*;

#[cfg(any(
    freebsdlike,
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod bsd;

#[cfg(any(
    freebsdlike,
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub use self::bsd::*;
