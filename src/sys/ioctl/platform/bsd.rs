mod consts {
    pub const VOID: u32 = 0x20000000;
    pub const OUT: u32 = 0x40000000;
    pub const IN: u32 = 0x80000000;
    pub const INOUT: u32 = (IN|OUT);
    pub const IOCPARM_MASK: u32 = 0x1fff;
}

pub use self::consts::*;

#[macro_export]
macro_rules! ioc {
    ($inout:expr, $group:expr, $num:expr, $len:expr) => (
        $inout | (($len as u32 & $crate::sys::ioctl::IOCPARM_MASK) << 16) | (($group as u32) << 8) | ($num as u32)
    )
}

#[macro_export]
macro_rules! io {
    ($g:expr, $n:expr) => (ioc!($crate::sys::ioctl::VOID, $g, $n, 0))
}

#[macro_export]
macro_rules! ior {
    ($g:expr, $n:expr, $len:expr) => (ioc!($crate::sys::ioctl::OUT, $g, $n, $len))
}

#[macro_export]
macro_rules! iow {
    ($g:expr, $n:expr, $len:expr) => (ioc!($crate::sys::ioctl::IN, $g, $n, $len))
}

#[macro_export]
macro_rules! iorw {
    ($g:expr, $n:expr, $len:expr) => (ioc!($crate::sys::ioctl::INOUT, $g, $n, $len))
}
