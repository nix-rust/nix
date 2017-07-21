/// The datatype used for the ioctl number
#[doc(hidden)]
pub type ioctl_num_type = ::libc::c_ulong;

mod consts {
    use ::sys::ioctl::platform::ioctl_num_type;
    #[doc(hidden)]
    pub const VOID: ioctl_num_type = 0x20000000;
    #[doc(hidden)]
    pub const OUT: ioctl_num_type = 0x40000000;
    #[doc(hidden)]
    pub const IN: ioctl_num_type = 0x80000000;
    #[doc(hidden)]
    pub const INOUT: ioctl_num_type = (IN|OUT);
    #[doc(hidden)]
    pub const IOCPARM_MASK: ioctl_num_type = 0x1fff;
}

pub use self::consts::*;

#[macro_export]
#[doc(hidden)]
macro_rules! ioc {
    ($inout:expr, $group:expr, $num:expr, $len:expr) => (
        $inout | (($len as $crate::sys::ioctl::ioctl_num_type & $crate::sys::ioctl::IOCPARM_MASK) << 16) | (($group as $crate::sys::ioctl::ioctl_num_type) << 8) | ($num as $crate::sys::ioctl::ioctl_num_type)
    )
}

#[macro_export]
#[doc(hidden)]
macro_rules! io {
    ($g:expr, $n:expr) => (ioc!($crate::sys::ioctl::VOID, $g, $n, 0))
}

#[macro_export]
#[doc(hidden)]
macro_rules! ior {
    ($g:expr, $n:expr, $len:expr) => (ioc!($crate::sys::ioctl::OUT, $g, $n, $len))
}

#[macro_export]
#[doc(hidden)]
macro_rules! iow {
    ($g:expr, $n:expr, $len:expr) => (ioc!($crate::sys::ioctl::IN, $g, $n, $len))
}

#[macro_export]
#[doc(hidden)]
macro_rules! iorw {
    ($g:expr, $n:expr, $len:expr) => (ioc!($crate::sys::ioctl::INOUT, $g, $n, $len))
}
