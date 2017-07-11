/// The datatype used for the ioctl number
#[cfg(any(target_os = "android", target_env = "musl"))]
#[doc(hidden)]
pub type ioctl_num_type = ::libc::c_int;
#[cfg(not(any(target_os = "android", target_env = "musl")))]
#[doc(hidden)]
pub type ioctl_num_type = ::libc::c_ulong;

#[doc(hidden)]
pub const NRBITS: ioctl_num_type = 8;
#[doc(hidden)]
pub const TYPEBITS: ioctl_num_type = 8;

#[cfg(any(target_arch = "mips", target_arch = "mips64", target_arch = "powerpc", target_arch = "powerpc64"))]
mod consts {
    #[doc(hidden)]
    pub const NONE: u8 = 1;
    #[doc(hidden)]
    pub const READ: u8 = 2;
    #[doc(hidden)]
    pub const WRITE: u8 = 4;
    #[doc(hidden)]
    pub const SIZEBITS: u8 = 13;
    #[doc(hidden)]
    pub const DIRBITS: u8 = 3;
}

#[cfg(not(any(target_arch = "powerpc",
              target_arch = "mips",
              target_arch = "mips64",
              target_arch = "x86",
              target_arch = "arm",
              target_arch = "x86_64",
              target_arch = "powerpc64",
              target_arch = "s390x",
              target_arch = "aarch64")))]
use this_arch_not_supported;

// "Generic" ioctl protocol
#[cfg(any(target_arch = "x86",
          target_arch = "arm",
          target_arch = "s390x",
          target_arch = "x86_64",
          target_arch = "aarch64"))]
mod consts {
    #[doc(hidden)]
    pub const NONE: u8 = 0;
    #[doc(hidden)]
    pub const READ: u8 = 2;
    #[doc(hidden)]
    pub const WRITE: u8 = 1;
    #[doc(hidden)]
    pub const SIZEBITS: u8 = 14;
    #[doc(hidden)]
    pub const DIRBITS: u8 = 2;
}

pub use self::consts::*;

#[doc(hidden)]
pub const NRSHIFT: ioctl_num_type = 0;
#[doc(hidden)]
pub const TYPESHIFT: ioctl_num_type = NRSHIFT + NRBITS as ioctl_num_type;
#[doc(hidden)]
pub const SIZESHIFT: ioctl_num_type = TYPESHIFT + TYPEBITS as ioctl_num_type;
#[doc(hidden)]
pub const DIRSHIFT: ioctl_num_type = SIZESHIFT + SIZEBITS as ioctl_num_type;

#[doc(hidden)]
pub const NRMASK: ioctl_num_type = (1 << NRBITS) - 1;
#[doc(hidden)]
pub const TYPEMASK: ioctl_num_type = (1 << TYPEBITS) - 1;
#[doc(hidden)]
pub const SIZEMASK: ioctl_num_type = (1 << SIZEBITS) - 1;
#[doc(hidden)]
pub const DIRMASK: ioctl_num_type = (1 << DIRBITS) - 1;

/// Encode an ioctl command.
#[macro_export]
#[doc(hidden)]
macro_rules! ioc {
    ($dir:expr, $ty:expr, $nr:expr, $sz:expr) => (
        (($dir as $crate::sys::ioctl::ioctl_num_type & $crate::sys::ioctl::DIRMASK) << $crate::sys::ioctl::DIRSHIFT) |
        (($ty as $crate::sys::ioctl::ioctl_num_type & $crate::sys::ioctl::TYPEMASK) << $crate::sys::ioctl::TYPESHIFT) |
        (($nr as $crate::sys::ioctl::ioctl_num_type & $crate::sys::ioctl::NRMASK) << $crate::sys::ioctl::NRSHIFT) |
        (($sz as $crate::sys::ioctl::ioctl_num_type & $crate::sys::ioctl::SIZEMASK) << $crate::sys::ioctl::SIZESHIFT))
}

/// Encode an ioctl command that has no associated data.
#[macro_export]
#[doc(hidden)]
macro_rules! io {
    ($ty:expr, $nr:expr) => (ioc!($crate::sys::ioctl::NONE, $ty, $nr, 0))
}

/// Encode an ioctl command that reads.
#[macro_export]
#[doc(hidden)]
macro_rules! ior {
    ($ty:expr, $nr:expr, $sz:expr) => (ioc!($crate::sys::ioctl::READ, $ty, $nr, $sz))
}

/// Encode an ioctl command that writes.
#[macro_export]
#[doc(hidden)]
macro_rules! iow {
    ($ty:expr, $nr:expr, $sz:expr) => (ioc!($crate::sys::ioctl::WRITE, $ty, $nr, $sz))
}

/// Encode an ioctl command that both reads and writes.
#[macro_export]
#[doc(hidden)]
macro_rules! iorw {
    ($ty:expr, $nr:expr, $sz:expr) => (ioc!($crate::sys::ioctl::READ | $crate::sys::ioctl::WRITE, $ty, $nr, $sz))
}

#[doc(hidden)]
pub const IN: u32 = (WRITE as u32) << DIRSHIFT;
#[doc(hidden)]
pub const OUT: u32 = (READ as u32) << DIRSHIFT;
#[doc(hidden)]
pub const INOUT: u32 = ((READ|WRITE) as u32) << DIRSHIFT;
#[doc(hidden)]
pub const SIZE_MASK: u32 = SIZEMASK << SIZESHIFT;
