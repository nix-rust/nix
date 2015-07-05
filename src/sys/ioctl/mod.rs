//! Provide helpers for making ioctl system calls
//!
//! # Overview of IOCTLs
//!
//! The `ioctl` system call is a widely support system
//! call on *nix systems providing access to functions
//! and data that do not fit nicely into the standard
//! read and write operations on a file itself.  It is
//! common to see ioctls used for the following purposes:
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
//! Ioctls are synchronous system calls and are similar to read and
//! write calls in that regard.
//!
//! The prototype for the ioctl system call in libc is as follows:
//!
//! ```c
//! int ioctl(int fd, unsigned long request, ...);
//! ```
//!
//! Typically, an ioctl takes 3 parameters as arguments:
//!
//! 1. An open file descriptor, `fd`.
//! 2. An device-dependennt request code or operation.  This request
//!    code is referred to as `op` in this module.
//! 3. Either a pointer to a location in memory or an integer.  This
//!    number of pointer may either be used by the kernel or written
//!    to by the kernel depending on how the operation is documented
//!    to work.
//!
//! The `op` request code is essentially an arbitrary integer having
//! a device-driver specific meaning.  Over time, it proved difficult
//! for various driver implementors to use this field sanely, so a
//! convention with macros was introduced to the Linux Kernel that
//! is used by most newer drivers.  See
//! https://github.com/torvalds/linux/blob/master/Documentation/ioctl/ioctl-number.txt
//! for additional details.  The macros exposed by the kernel for
//! consumers are implemented in this module and may be used to
//! instead of calls like `_IOC`, `_IO`, `_IOR`, and `_IOW`.
//!
//! # Interface Overview
//!
//! This ioctl module seeks to tame the ioctl beast by providing
//! a set of safer (although not safe) functions
//! implementing the most common ioctl access patterns.
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

//#[cfg(not(target_os = "linux"))]
//use platform_not_supported;
