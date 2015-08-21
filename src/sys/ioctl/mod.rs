//! Provide helpers for making ioctl system calls
//!
//! Currently supports Linux on all architectures. Other platforms welcome!
//!
//! This library is pretty low-level and messy. `ioctl` is not fun.
//!
//! What is an `ioctl`?
//! ===================
//!
//! The `ioctl` syscall is the grab-bag syscall on POSIX systems. Don't want
//! to add a new syscall? Make it an `ioctl`! `ioctl` refers to both the syscall,
//! and the commands that can be send with it. `ioctl` stands for "IO control",
//! and the commands are always sent to a file descriptor.
//!
//! It is common to see `ioctl`s used for the following purposes:
//!
//! * Provide read/write access to out-of-band data related
//!   to a device such as configuration (for instance, setting
//!   serial port options)
//! * Provide a mechanism for performing full-duplex data
//!   transfers (for instance, xfer on SPI devices).
//! * Provide access to control functions on a device (for example,
//!   on Linux you can send commands like pause, resume, and eject
//!   to the CDROM device.
//! * Do whatever else the device driver creator thought made most sense.
//!
//! `ioctl`s are synchronous system calls and are similar to read and
//! write calls in that regard.
//!
//! What does this module support?
//! ===============================
//!
//! This library provides the `ioctl!` macro, for binding `ioctl`s. It also tries
//! to bind every `ioctl` supported by the system with said macro, but
//! some `ioctl`s requires some amount of manual work (usually by
//! providing `struct` declaration) that this library does not support yet.
//!
//! Additionally, in `etc`, there are scripts for scraping system headers for
//! `ioctl` definitions, and generating calls to `ioctl!` corresponding to them.
//!
//! How do I get the magic numbers?
//! ===============================
//!
//! For Linux, look at your system's headers. For example, `/usr/include/linxu/input.h` has a lot
//! of lines defining macros which use `_IOR`, `_IOW`, `_IOC`, and `_IORW`.  These macros
//! correspond to the `ior!`, `iow!`, `ioc!`, and `iorw!` macros defined in this crate.
//! Additionally, there is the `ioctl!` macro for creating a wrapper around `ioctl` that is
//! somewhat more type-safe.
//!
//! Most `ioctl`s have no or little documentation. You'll need to scrounge through
//! the source to figure out what they do and how they should be used.
//!
//! # Interface Overview
//!
//! This ioctl module seeks to tame the ioctl beast by providing a set of safer (although not safe)
//! functions implementing the most common ioctl access patterns.
//!
//! The most common access patterns for ioctls are as follows:
//!
//! 1. `read`: A pointer is provided to the kernel which is populated
//!    with a value containing the "result" of the operation.  The
//!    result may be an integer or structure.  The kernel may also
//!    read values from the provided pointer (usually a structure).
//! 2. `write`: A pointer is provided to the kernel containing values
//!    that the kernel will read in order to perform the operation.
//! 3. `execute`: The operation is passed to the kernel but no
//!    additional pointer is passed.  The operation is enough
//!    and it either succeeds or results in an error.
//!
//! Where appropriate, versions of these interface function are provided
//! taking either refernces or pointers.  The pointer versions are
//! necessary for cases (notably slices) where a reference cannot
//! be generically cast to a pointer.

#[cfg(any(target_os = "linux", target_os = "android"))]
#[path = "platform/linux.rs"]
#[macro_use]
mod platform;

#[cfg(target_os = "macos")]
#[path = "platform/macos.rs"]
#[macro_use]
mod platform;

#[cfg(target_os = "ios")]
#[path = "platform/ios.rs"]
#[macro_use]
mod platform;

#[cfg(target_os = "freebsd")]
#[path = "platform/freebsd.rs"]
#[macro_use]
mod platform;

#[cfg(target_os = "openbsd")]
#[path = "platform/openbsd.rs"]
#[macro_use]
mod platform;

#[cfg(target_os = "dragonfly")]
#[path = "platform/dragonfly.rs"]
#[macro_use]
mod platform;

pub use self::platform::*;

// liblibc has the wrong decl for linux :| hack until #26809 lands.
extern "C" {
    #[doc(hidden)]
    pub fn ioctl(fd: libc::c_int, req: libc::c_ulong, ...) -> libc::c_int;
}

/// A hack to get the macros to work nicely.
#[doc(hidden)]
pub use ::libc as libc;
