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
//! This library provides the `ioctl!` macro, for binding `ioctl`s.
//! Here's a few examples of how that can work for SPI under Linux
//! from [rust-spidev](https://github.com/posborne/rust-spidev).
//!
//! ```
//! #[allow(non_camel_case_types)]
//! pub struct spi_ioc_transfer {
//!     pub tx_buf: u64,
//!     pub rx_buf: u64,
//!     pub len: u32,
//!
//!     // optional overrides
//!     pub speed_hz: u32,
//!     pub delay_usecs: u16,
//!     pub bits_per_word: u8,
//!     pub cs_change: u8,
//!     pub pad: u32,
//! }
//!
//! #[cfg(linux)]
//! mod ioctl {
//!     use super::*;
//!
//!     const SPI_IOC_MAGIC: u8 = 'k' as u8;
//!     const SPI_IOC_NR_TRANSFER: u8 = 0;
//!     const SPI_IOC_NR_MODE: u8 = 1;
//!     const SPI_IOC_NR_LSB_FIRST: u8 = 2;
//!     const SPI_IOC_NR_BITS_PER_WORD: u8 = 3;
//!     const SPI_IOC_NR_MAX_SPEED_HZ: u8 = 4;
//!     const SPI_IOC_NR_MODE32: u8 = 5;
//!
//!     ioctl!(read  get_mode_u8 with SPI_IOC_MAGIC, SPI_IOC_NR_MODE; u8);
//!     ioctl!(read  get_mode_u32 with SPI_IOC_MAGIC, SPI_IOC_NR_MODE; u32);
//!     ioctl!(write set_mode_u8 with SPI_IOC_MAGIC, SPI_IOC_NR_MODE; u8);
//!     ioctl!(write set_mode_u32 with SPI_IOC_MAGIC, SPI_IOC_NR_MODE32; u32);
//!     ioctl!(read  get_lsb_first with SPI_IOC_MAGIC, SPI_IOC_NR_LSB_FIRST; u8);
//!     ioctl!(write set_lsb_first with SPI_IOC_MAGIC, SPI_IOC_NR_LSB_FIRST; u8);
//!     ioctl!(read  get_bits_per_word with SPI_IOC_MAGIC, SPI_IOC_NR_BITS_PER_WORD; u8);
//!     ioctl!(write set_bits_per_word with SPI_IOC_MAGIC, SPI_IOC_NR_BITS_PER_WORD; u8);
//!     ioctl!(read  get_max_speed_hz with SPI_IOC_MAGIC, SPI_IOC_NR_MAX_SPEED_HZ; u32);
//!     ioctl!(write set_max_speed_hz with SPI_IOC_MAGIC, SPI_IOC_NR_MAX_SPEED_HZ; u32);
//!     ioctl!(write spidev_transfer with SPI_IOC_MAGIC, SPI_IOC_NR_TRANSFER; spi_ioc_transfer);
//!     ioctl!(write buf spidev_transfer_buf with SPI_IOC_MAGIC, SPI_IOC_NR_TRANSFER; spi_ioc_transfer);
//! }
//!
//! // doctest workaround
//! fn main() {}
//! ```
//!
//! Spidev uses the `_IOC` macros that are encouraged (as far as
//! `ioctl` can be encouraged at all) for newer drivers.  Many
//! drivers, however, just use magic numbers with no attached
//! semantics.  For those, the `ioctl!(bad ...)` variant should be
//! used (the "bad" terminology is from the Linux kernel).
//!
//! How do I get the magic numbers?
//! ===============================
//!
//! For Linux, look at your system's headers. For example, `/usr/include/linux/input.h` has a lot
//! of lines defining macros which use `_IOR`, `_IOW`, `_IOC`, and `_IORW`.  These macros
//! correspond to the `ior!`, `iow!`, `ioc!`, and `iorw!` macros defined in this crate.
//! Additionally, there is the `ioctl!` macro for creating a wrapper around `ioctl` that is
//! somewhat more type-safe.
//!
//! Most `ioctl`s have no or little documentation. You'll need to scrounge through
//! the source to figure out what they do and how they should be used.
//!
#[cfg(any(target_os = "linux", target_os = "android"))]
#[path = "platform/linux.rs"]
#[macro_use]
mod platform;

#[cfg(any(target_os = "macos",
          target_os = "ios",
          target_os = "netbsd",
          target_os = "openbsd",
          target_os = "freebsd",
          target_os = "dragonfly"))]
#[path = "platform/bsd.rs"]
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

/// Convert raw ioctl return value to a Nix result
#[macro_export]
macro_rules! convert_ioctl_res {
    ($w:expr) => (
        {
            $crate::Errno::result($w)
        }
    );
}

#[macro_export]
macro_rules! ioctl {
    ($name:ident with $nr:expr) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int,
                            data: *mut u8)
                            -> $crate::Result<$crate::sys::ioctl::libc::c_int> {
            convert_ioctl_res!($crate::sys::ioctl::ioctl(fd, $nr as $crate::sys::ioctl::libc::c_ulong, data))
        }
        );
    (none $name:ident with $ioty:expr, $nr:expr) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int)
                            -> $crate::Result<$crate::sys::ioctl::libc::c_int> {
            convert_ioctl_res!($crate::sys::ioctl::ioctl(fd, io!($ioty, $nr) as $crate::sys::ioctl::libc::c_ulong))
        }
        );
    (read $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int,
                            val: *mut $ty)
                            -> $crate::Result<$crate::sys::ioctl::libc::c_int> {
            convert_ioctl_res!($crate::sys::ioctl::ioctl(fd, ior!($ioty, $nr, ::std::mem::size_of::<$ty>()) as $crate::sys::ioctl::libc::c_ulong, val))
        }
        );
    (write $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int,
                            val: $ty)
                            -> $crate::Result<$crate::sys::ioctl::libc::c_int> {
            convert_ioctl_res!($crate::sys::ioctl::ioctl(fd, iow!($ioty, $nr, ::std::mem::size_of::<$ty>()) as $crate::sys::ioctl::libc::c_ulong, val))
        }
        );
    (readwrite $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int,
                            val: *mut $ty)
                            -> $crate::Result<$crate::sys::ioctl::libc::c_int> {
            convert_ioctl_res!($crate::sys::ioctl::ioctl(fd, iorw!($ioty, $nr, ::std::mem::size_of::<$ty>()) as $crate::sys::ioctl::libc::c_ulong, val))
        }
        );
    (read buf $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int,
                            val: *mut $ty,
                            len: usize)
                            -> $crate::Result<$crate::sys::ioctl::libc::c_int> {
            convert_ioctl_res!($crate::sys::ioctl::ioctl(fd, ior!($ioty, $nr, len) as $crate::sys::ioctl::libc::c_ulong, val))
        }
        );
    (write buf $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int,
                            val: *const $ty,
                            len: usize) -> $crate::Result<$crate::sys::ioctl::libc::c_int> {
            convert_ioctl_res!($crate::sys::ioctl::ioctl(fd, iow!($ioty, $nr, len) as $crate::sys::ioctl::libc::c_ulong, val))
        }
        );
    (readwrite buf $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int,
                            val: *mut $ty,
                            len: usize)
                            -> $crate::Result<$crate::sys::ioctl::libc::c_int> {
            convert_ioctl_res!($crate::sys::ioctl::ioctl(fd, iorw!($ioty, $nr, len) as $crate::sys::ioctl::libc::c_ulong, val))
        }
        );
}
