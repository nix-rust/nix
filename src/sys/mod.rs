#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "ios",
          target_os = "netbsd", target_os = "macos", target_os = "linux"))]
pub mod aio;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod epoll;

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd",
    target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd"))]
pub mod event;

#[cfg(target_os = "linux")]
pub mod eventfd;

#[cfg(target_os = "linux")]
pub mod memfd;

#[macro_use]
pub mod ioctl;

// TODO: Add support for dragonfly, freebsd, and ios/macos.
#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod sendfile;

pub mod signal;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod signalfd;

pub mod socket;

pub mod stat;

#[cfg(any(target_os = "linux"))]
pub mod reboot;

pub mod termios;

pub mod utsname;

pub mod wait;

pub mod mman;

pub mod uio;

pub mod time;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod ptrace;

pub mod select;

#[cfg(target_os = "linux")]
pub mod quota;


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
pub mod pthread;
