
#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod epoll;

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd",
    target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd"))]
pub mod event;

// TODO: switch from feature flags to conditional builds
#[cfg(feature = "eventfd")]
pub mod eventfd;

#[cfg(target_os = "linux")]
pub mod memfd;

#[cfg(not(any(target_os = "ios", target_os = "freebsd", target_os = "dragonfly")))]
pub mod ioctl;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod sendfile;

pub mod signal;

#[cfg(any(target_os = "linux", target_os = "android"))]
#[cfg(feature = "signalfd")]
pub mod signalfd;

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

pub mod time;

#[cfg(all(target_os = "linux",
          any(target_arch = "x86",
              target_arch = "x86_64",
              target_arch = "arm")),
          )]
pub mod ptrace;

pub mod select;

#[cfg(all(target_os = "linux",
          any(target_arch = "x86",
              target_arch = "x86_64",
              target_arch = "arm")),
          )]
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
