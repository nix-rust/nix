#[doc(hidden)]
pub const NRBITS: u32 = 8;
#[doc(hidden)]
pub const TYPEBITS: u32 = 8;

#[cfg(any(target_arch = "mips", target_arch = "powerpc"))]
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

#[cfg(not(any(target_arch = "powerpc", target_arch = "mips", target_arch = "x86", target_arch = "arm", target_arch = "x86_64", target_arch = "aarch64")))]
use this_arch_not_supported;

// "Generic" ioctl protocol
#[cfg(any(target_arch = "x86", target_arch = "arm", target_arch = "x86_64", target_arch = "aarch64"))]
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

#[doc(hidden)]
pub use self::consts::*;

#[doc(hidden)]
pub const NRSHIFT: u32 = 0;
#[doc(hidden)]
pub const TYPESHIFT: u32 = NRSHIFT + NRBITS as u32;
#[doc(hidden)]
pub const SIZESHIFT: u32 = TYPESHIFT + TYPEBITS as u32;
#[doc(hidden)]
pub const DIRSHIFT: u32 = SIZESHIFT + SIZEBITS as u32;

#[doc(hidden)]
pub const NRMASK: u32 = (1 << NRBITS) - 1;
#[doc(hidden)]
pub const TYPEMASK: u32 = (1 << TYPEBITS) - 1;
#[doc(hidden)]
pub const SIZEMASK: u32 = (1 << SIZEBITS) - 1;
#[doc(hidden)]
pub const DIRMASK: u32 = (1 << DIRBITS) - 1;

/// Encode an ioctl command.
#[macro_export]
macro_rules! ioc {
    ($dir:expr, $ty:expr, $nr:expr, $sz:expr) => (
        (($dir as u32) << $crate::sys::ioctl::DIRSHIFT) |
        (($ty as u32) << $crate::sys::ioctl::TYPESHIFT) |
        (($nr as u32) << $crate::sys::ioctl::NRSHIFT) |
        (($sz as u32) << $crate::sys::ioctl::SIZESHIFT))
}

/// Encode an ioctl command that has no associated data.
#[macro_export]
macro_rules! io {
    ($ty:expr, $nr:expr) => (ioc!($crate::sys::ioctl::NONE, $ty, $nr, 0))
}

/// Encode an ioctl command that reads.
#[macro_export]
macro_rules! ior {
    ($ty:expr, $nr:expr, $sz:expr) => (ioc!($crate::sys::ioctl::READ, $ty, $nr, $sz))
}

/// Encode an ioctl command that writes.
#[macro_export]
macro_rules! iow {
    ($ty:expr, $nr:expr, $sz:expr) => (ioc!($crate::sys::ioctl::WRITE, $ty, $nr, $sz))
}

/// Encode an ioctl command that both reads and writes.
#[macro_export]
macro_rules! iorw {
    ($ty:expr, $nr:expr, $sz:expr) => (ioc!($crate::sys::ioctl::READ | $crate::sys::ioctl::WRITE, $ty, $nr, $sz))
}

/// Declare a wrapper function around an ioctl.
#[macro_export]
macro_rules! ioctl {
    (bad $name:ident with $nr:expr) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int, data: *mut u8) -> $crate::sys::ioctl::libc::c_int {
            $crate::sys::ioctl::ioctl(fd, $nr as $crate::sys::ioctl::libc::c_ulong, data)
        }
    );
    (none $name:ident with $ioty:expr, $nr:expr) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int) -> $crate::sys::ioctl::libc::c_int {
            $crate::sys::ioctl::ioctl(fd, io!($ioty, $nr) as $crate::sys::ioctl::libc::c_ulong)
        }
    );
    (read $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int, val: *mut $ty) -> $crate::sys::ioctl::libc::c_int {
            $crate::sys::ioctl::ioctl(fd, ior!($ioty, $nr, ::std::mem::size_of::<$ty>()) as $crate::sys::ioctl::libc::c_ulong, val)
        }
        );
    (write $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int, val: *const $ty) -> $crate::sys::ioctl::libc::c_int {
            $crate::sys::ioctl::ioctl(fd, ior!($ioty, $nr, ::std::mem::size_of::<$ty>()) as $crate::sys::ioctl::libc::c_ulong, val)
        }
        );
    (readwrite $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int, val: *mut $ty) -> $crate::sys::ioctl::libc::c_int {
            $crate::sys::ioctl::ioctl(fd, iorw!($ioty, $nr, ::std::mem::size_of::<$ty>()) as $crate::sys::ioctl::libc::c_ulong, val)
        }
        );
    (read buf $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int, val: *mut $ty, len: usize) -> $crate::sys::ioctl::libc::c_int {
            $crate::sys::ioctl::ioctl(fd, ior!($ioty, $nr, len) as $crate::sys::ioctl::libc::c_ulong, val)
        }
        );
    (write buf $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int, val: *const $ty, len: usize) -> $crate::sys::ioctl::libc::c_int {
            $crate::sys::ioctl::ioctl(fd, ior!($ioty, $nr, len) as $crate::sys::ioctl::libc::c_ulong, val)
        }
        );
    (readwrite buf $name:ident with $ioty:expr, $nr:expr; $ty:ty) => (
        pub unsafe fn $name(fd: $crate::sys::ioctl::libc::c_int, val: *const $ty, len: usize) -> $crate::sys::ioctl::libc::c_int {
            $crate::sys::ioctl::ioctl(fd, iorw!($ioty, $nr, len) as $crate::sys::ioctl::libc::c_ulong, val)
        }
        );
}

/// Extracts the "direction" (read/write/none) from an encoded ioctl command.
#[inline(always)]
pub fn ioc_dir(nr: u32) -> u8 {
    ((nr >> DIRSHIFT) & DIRMASK) as u8
}

/// Extracts the type from an encoded ioctl command.
#[inline(always)]
pub fn ioc_type(nr: u32) -> u32 {
    (nr >> TYPESHIFT) & TYPEMASK
}

/// Extracts the ioctl number from an encoded ioctl command.
#[inline(always)]
pub fn ioc_nr(nr: u32) -> u32 {
    (nr >> NRSHIFT) & NRMASK
}

/// Extracts the size from an encoded ioctl command.
#[inline(always)]
pub fn ioc_size(nr: u32) -> u32 {
    ((nr >> SIZESHIFT) as u32) & SIZEMASK
}

#[doc(hidden)]
pub const IN: u32 = (WRITE as u32) << DIRSHIFT;
#[doc(hidden)]
pub const OUT: u32 = (READ as u32) << DIRSHIFT;
#[doc(hidden)]
pub const INOUT: u32 = ((READ|WRITE) as u32) << DIRSHIFT;
#[doc(hidden)]
pub const SIZE_MASK: u32 = SIZEMASK << SIZESHIFT;

const WATCHDOG_IOCTL_BASE: u32 = b'W' as u32;
const BASE_VIDIOC_PRIVATE: u32 = 192;
const USBTMC_IOC_NR: u32 = 91;
const UBI_VOL_IOC_MAGIC: u32 = b'O' as u32;
const UBI_IOC_MAGIC: u32 = b'o' as u32;
const UBI_CTRL_IOC_MAGIC: u32 = b'o' as u32;
const SPI_IOC_MAGIC: u32 = b'k' as u32;
const MGSL_MAGIC_IOC: u32 = b'm' as u32;
const DRM_IOCTL_BASE: u32 = b'd' as u32;
const DRM_COMMAND_BASE: u32 = 0x40;
const BTRFS_IOCTL_MAGIC: u32 = 0x94;
const BTRFS_LABEL_SIZE: usize = 256;
//const SIOCPARM_MASK: u32 = 0x1fff;

const ATMIOC_ITF: u32 = 0x80;
const ATMIOC_CLIP: u32 = 0xe0;
const RFKILL_IOC_NOINPUT: u32 = 1;
const RFKILL_IOC_MAGIC: u32 = b'R' as u32;
const IFNAMSIZ: usize = 16;
const DECNET_IOCTL_BASE: u32 = 0x89;
const ABS_MAX: usize = 0x3f;
const ABS_CNT: usize = ABS_MAX+1;
const CXL_MAGIC: u32 = 0xCA;
const CM_IOC_MAGIC: u32 = b'c' as u32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct input_event {
    pub time: ::libc::timeval,
    pub _type: u16,
    pub code: u16,
    pub value: i32,
}
impl ::std::default::Default for input_event {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
impl ::std::fmt::Debug for input_event {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "input_event {{ time: {{ tv_sec: {}, tv_usec: {} }}, _type: {}, code: {}, value: {}",
               self.time.tv_sec, self.time.tv_usec, self._type, self.code, self.value)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct input_id {
    pub bustype: u16,
    pub vendor: u16,
    pub product: u16,
    pub version: u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ff_effect {
    pub _type: u16,
    pub id: i16,
    pub direction: u16,
    pub trigger: ff_trigger,
    pub replay: ff_replay,
    pub u: ff_effect_data,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ff_effect_data {
    pub _bindgen_data_: [u64; 4usize],
}
impl Union_Unnamed16 {
    pub unsafe fn constant(&mut self) -> *mut ff_constant_effect {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
    pub unsafe fn ramp(&mut self) -> *mut ff_ramp_effect {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
    pub unsafe fn periodic(&mut self) -> *mut ff_periodic_effect {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
    pub unsafe fn condition(&mut self)
     -> *mut [ff_condition_effect; 2usize] {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
    pub unsafe fn rumble(&mut self) -> *mut ff_rumble_effect {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
}
impl ::std::default::Default for ff_effect_data {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct input_absinfo {
    pub value: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub fuzz: i32,
    pub flat: i32,
    pub resolution: i32,
}
impl ::std::default::Default for input_absinfo {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct input_keymap_entry {
    pub flags: u8,
    pub len: u8,
    pub index: u16,
    pub keycode: u32,
    pub scancode: [u8; 32usize],
}
impl ::std::default::Default for input_keymap_entry {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ff_replay {
    pub length: u16,
    pub delay: u16,
}
impl ::std::default::Default for ff_replay {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ff_trigger {
    pub button: u16,
    pub interval: u16,
}
impl ::std::default::Default for ff_trigger {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ff_envelope {
    pub attack_length: u16,
    pub attack_level: u16,
    pub fade_length: u16,
    pub fade_level: u16,
}
impl ::std::default::Default for ff_envelope {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ff_constant_effect {
    pub level: i16,
    pub envelope: ff_envelope,
}
impl ::std::default::Default for ff_constant_effect {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ff_ramp_effect {
    pub start_level: i16,
    pub end_level: i16,
    pub envelope: ff_envelope,
}
impl ::std::default::Default for ff_ramp_effect {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ff_condition_effect {
    pub right_saturation: u16,
    pub left_saturation: u16,
    pub right_coeff: i16,
    pub left_coeff: i16,
    pub deadband: u16,
    pub center: i16,
}
impl ::std::default::Default for ff_condition_effect {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[allow(raw_pointer_derive)]
pub struct ff_periodic_effect {
    pub waveform: u16,
    pub period: u16,
    pub magnitude: i16,
    pub offset: i16,
    pub phase: u16,
    pub envelope: ff_envelope,
    pub custom_len: u32,
    pub custom_data: *mut i16,
}
impl ::std::default::Default for ff_periodic_effect {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ff_rumble_effect {
    pub strong_magnitude: u16,
    pub weak_magnitude: u16,
}
impl ::std::default::Default for ff_rumble_effect {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

ioctl!(read buf eviocgname with b'E', 0x06; u8);
ioctl!(read buf eviocgphys with b'E', 0x07; u8);
ioctl!(read buf eviocguniq with b'E', 0x08; u8);
ioctl!(read buf eviocgprop with b'E', 0x09; u8);
ioctl!(read buf eviocgmtslots with b'E', 0x0a; u8);
ioctl!(read buf eviocgkey with b'E', 0x18; u8);
ioctl!(read buf eviocgled with b'E', 0x19; u8);
ioctl!(read buf eviocgsnd with b'E', 0x1a; u8);
ioctl!(read buf eviocgsw with b'E', 0x1b; u8);

ioctl!(write eviocsff with b'E', 0x80; ff_effect);
ioctl!(write eviocgrab with b'E', 0x90; ::libc::c_int);
ioctl!(write eviocrevoke with b'E', 0x91; ::libc::c_int);
ioctl!(write eviocsclockid with b'E', 0xa0; ::libc::c_int);

pub unsafe fn eviocgbit(fd: ::libc::c_int, ev: u32, len: ::libc::c_int, buf: *mut u8) -> ::libc::c_int {
    super::ioctl(fd, ior!(b'E', 0x20 + ev, len) as ::libc::c_ulong, buf)
}

pub unsafe fn eviocgabs(fd: ::libc::c_int, abs: u32, buf: *mut input_absinfo) -> ::libc::c_int {
    super::ioctl(fd, ior!(b'E', 0x40 + abs, ::std::mem::size_of::<input_absinfo>()) as ::libc::c_ulong, buf)
}

#[cfg(target_arch = "x86_64")]
include!("linux-generated-x86_64.rs");
