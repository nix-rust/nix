//! Mostly platform-specific functionality
#[cfg(any(
    freebsdlike,
    all(target_os = "linux", not(target_env = "uclibc")),
    apple_targets,
    target_os = "netbsd"
))]
feature! {
    #![feature = "aio"]
    pub mod aio;
}

feature! {
    #![feature = "event"]

    #[cfg(linux_android)]
    #[allow(missing_docs)]
    pub mod epoll;

    #[cfg(bsd)]
    pub mod event;

    #[cfg(linux_android)]
    #[allow(missing_docs)]
    pub mod eventfd;
}

#[cfg(target_os = "linux")]
feature! {
    #![feature = "fanotify"]
    pub mod fanotify;
}

#[cfg(any(bsd, linux_android, target_os = "redox", target_os = "illumos"))]
#[cfg(feature = "ioctl")]
#[cfg_attr(docsrs, doc(cfg(feature = "ioctl")))]
#[macro_use]
pub mod ioctl;

#[cfg(any(linux_android, target_os = "freebsd"))]
feature! {
    #![feature = "fs"]
    pub mod memfd;
}

#[cfg(not(target_os = "redox"))]
feature! {
    #![feature = "mman"]
    pub mod mman;
}

#[cfg(target_os = "linux")]
feature! {
    #![feature = "personality"]
    pub mod personality;
}

#[cfg(target_os = "linux")]
feature! {
    #![feature = "process"]
    pub mod prctl;
}

feature! {
    #![feature = "pthread"]
    pub mod pthread;
}

#[cfg(any(
    linux_android,
    freebsdlike,
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
feature! {
    #![feature = "ptrace"]
    #[allow(missing_docs)]
    pub mod ptrace;
}

#[cfg(target_os = "linux")]
feature! {
    #![feature = "quota"]
    pub mod quota;
}

#[cfg(target_os = "linux")]
feature! {
    #![feature = "reboot"]
    pub mod reboot;
}

#[cfg(not(any(
    target_os = "redox",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "haiku"
)))]
feature! {
    #![feature = "resource"]
    pub mod resource;
}

feature! {
    #![feature = "poll"]
    pub mod select;
}

#[cfg(any(linux_android, freebsdlike, apple_targets, solarish))]
feature! {
    #![feature = "zerocopy"]
    pub mod sendfile;
}

pub mod signal;

#[cfg(linux_android)]
feature! {
    #![feature = "signal"]
    #[allow(missing_docs)]
    pub mod signalfd;
}

feature! {
    #![feature = "socket"]
    #[allow(missing_docs)]
    pub mod socket;
}

feature! {
    #![feature = "fs"]
    #[allow(missing_docs)]
    pub mod stat;
}

#[cfg(any(linux_android, freebsdlike, apple_targets, target_os = "openbsd"))]
feature! {
    #![feature = "fs"]
    pub mod statfs;
}

feature! {
    #![feature = "fs"]
    pub mod statvfs;
}

#[cfg(linux_android)]
#[allow(missing_docs)]
pub mod sysinfo;

feature! {
    #![feature = "term"]
    #[allow(missing_docs)]
    pub mod termios;
}

#[allow(missing_docs)]
pub mod time;

feature! {
    #![feature = "uio"]
    pub mod uio;
}

feature! {
    #![feature = "feature"]
    pub mod utsname;
}

feature! {
    #![feature = "process"]
    pub mod wait;
}

#[cfg(linux_android)]
feature! {
    #![feature = "inotify"]
    pub mod inotify;
}

#[cfg(linux_android)]
feature! {
    #![feature = "time"]
    pub mod timerfd;
}

#[cfg(all(
    any(
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd"
    ),
    feature = "time",
    feature = "signal"
))]
feature! {
    #![feature = "time"]
    pub mod timer;
}
