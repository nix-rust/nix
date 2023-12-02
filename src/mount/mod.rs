//! Mount file systems
#[cfg(linux_android)]
mod linux;

#[cfg(linux_android)]
pub use self::linux::*;

#[cfg(any(freebsdlike, netbsdlike, target_os = "macos"))]
mod bsd;

#[cfg(any(freebsdlike, netbsdlike, target_os = "macos"))]
pub use self::bsd::*;
