
#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod epoll;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod event;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod eventfd;

#[cfg(not(target_os = "ios"))]
pub mod ioctl;

pub mod signal;

pub mod socket;

pub mod stat;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod syscall;

#[cfg(not(target_os = "ios"))]
pub mod termios;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod utsname;

pub mod wait;

pub mod mman;

pub mod uio;
