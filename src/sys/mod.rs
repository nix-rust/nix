#[cfg(all(feature = "aio",
    any(target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "linux",
          target_os = "macos",
          target_os = "netbsd")))]
pub mod aio;

#[cfg(all(feature = "epoll", any(target_os = "android", target_os = "linux")))]
pub mod epoll;

#[cfg(all(feature = "event",
    any(target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd")))]
pub mod event;

#[cfg(all(feature = "eventfd", target_os = "linux"))]
pub mod eventfd;

#[cfg(all(feature = "ioctl",
    any(target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd")))]
#[macro_use]
pub mod ioctl;

#[cfg(all(feature = "memfd", target_os = "linux"))]
pub mod memfd;

#[cfg(feature = "mman")]
pub mod mman;

#[cfg(feature = "pthread")]
pub mod pthread;

#[cfg(all(feature = "ptrace",
    any(target_os = "android",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "linux",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "openbsd")))]
pub mod ptrace;

#[cfg(all(feature = "quota", target_os = "linux"))]
pub mod quota;

#[cfg(all(feature = "reboot", target_os = "linux"))]
pub mod reboot;

#[cfg(feature = "select")]
pub mod select;

#[cfg(all(feature = "sendfile",
    any(target_os = "android",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "linux",
          target_os = "macos")))]
pub mod sendfile;

#[cfg(feature = "signal")]
pub mod signal;

#[cfg(all(feature = "signalfd", any(target_os = "android", target_os = "linux")))]
pub mod signalfd;

#[cfg(feature = "socket")]
pub mod socket;

#[cfg(feature = "stat")]
pub mod stat;

#[cfg(all(feature = "statfs",
    any(target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "openbsd")))]
pub mod statfs;

#[cfg(feature = "statvfs")]
pub mod statvfs;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod sysinfo;

#[cfg(feature = "termios")]
pub mod termios;

#[cfg(feature = "time")]
pub mod time;

#[cfg(feature = "uio")]
pub mod uio;

#[cfg(feature = "utsname")]
pub mod utsname;

#[cfg(feature = "wait")]
pub mod wait;

#[cfg(all(feature = "inotify", any(target_os = "android", target_os = "linux")))]
pub mod inotify;
