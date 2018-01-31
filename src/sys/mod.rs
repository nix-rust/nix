#[cfg(any(target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "linux",
          target_os = "macos",
          target_os = "netbsd"))]
pub mod aio;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod epoll;

#[cfg(any(target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "openbsd"))]
pub mod event;

#[cfg(target_os = "linux")]
pub mod eventfd;

#[macro_use]
pub mod ioctl;

#[cfg(target_os = "linux")]
pub mod memfd;

pub mod mman;

pub mod pthread;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod ptrace;

#[cfg(target_os = "linux")]
pub mod quota;

#[cfg(any(target_os = "linux"))]
pub mod reboot;

pub mod select;

// TODO: Add support for dragonfly, freebsd, and ios/macos.
#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod sendfile;

pub mod signal;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod signalfd;

pub mod socket;

pub mod stat;

#[cfg(all(target_os = "linux",
          any(target_arch = "x86",
              target_arch = "x86_64",
              target_arch = "arm")),
          )]
pub mod statfs;

#[cfg(all(any(target_os = "linux",
              target_os = "macos"),
          any(target_arch = "x86",
              target_arch = "x86_64",
              target_arch = "arm")),
          )]
pub mod statvfs;

pub mod termios;

pub mod time;

pub mod uio;

pub mod utsname;

pub mod wait;
