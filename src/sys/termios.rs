use errno::{SysResult, from_ffi};
use fcntl::Fd;
use libc::c_int;

pub use self::ffi::consts::*;
pub use self::ffi::consts::SetArg::*;
pub use self::ffi::consts::FlushArg::*;
pub use self::ffi::consts::FlowArg::*;

mod ffi {
    use libc::c_int;

    pub use self::consts::*;

    // `Termios` contains bitflags which are not considered
    // `foreign-function-safe` by the compiler.
    #[allow(improper_ctypes)]
    extern {
        pub fn cfgetispeed(termios: *const Termios) -> speed_t;
        pub fn cfgetospeed(termios: *const Termios) -> speed_t;
        pub fn cfsetispeed(termios: *mut Termios, speed: speed_t) -> c_int;
        pub fn cfsetospeed(termios: *mut Termios, speed: speed_t) -> c_int;
        pub fn tcgetattr(fd: c_int, termios: *mut Termios) -> c_int;
        pub fn tcsetattr(fd: c_int,
                         optional_actions: c_int,
                         termios: *const Termios) -> c_int;
        pub fn tcdrain(fd: c_int) -> c_int;
        pub fn tcflow(fd: c_int, action: c_int) -> c_int;
        pub fn tcflush(fd: c_int, action: c_int) -> c_int;
        pub fn tcsendbreak(fd: c_int, duration: c_int) -> c_int;
    }

    #[cfg(target_os = "macos")]
    pub mod consts {
        use libc::{c_int, c_ulong, c_uchar};

        pub type tcflag_t = c_ulong;
        pub type cc_t = c_uchar;
        pub type speed_t = c_ulong;

        #[repr(C)]
        #[derive(Copy)]
        pub struct Termios {
            pub c_iflag: InputFlags,
            pub c_oflag: OutputFlags,
            pub c_cflag: ControlFlags,
            pub c_lflag: LocalFlags,
            pub c_cc: [cc_t; NCCS],
            pub c_ispeed: speed_t,
            pub c_ospeed: speed_t
        }

        pub const VEOF: usize     = 0;
        pub const VEOL: usize     = 1;
        pub const VEOL2: usize    = 2;
        pub const VERASE: usize   = 3;
        pub const VWERASE: usize  = 4;
        pub const VKILL: usize    = 5;
        pub const VREPRINT: usize = 6;
        pub const VINTR: usize    = 8;
        pub const VQUIT: usize    = 9;
        pub const VSUSP: usize    = 10;
        pub const VDSUSP: usize   = 11;
        pub const VSTART: usize   = 12;
        pub const VSTOP: usize    = 13;
        pub const VLNEXT: usize   = 14;
        pub const VDISCARD: usize = 15;
        pub const VMIN: usize     = 16;
        pub const VTIME: usize    = 17;
        pub const VSTATUS: usize  = 18;
        pub const NCCS: usize     = 20;

        bitflags! {
            flags InputFlags: tcflag_t {
                const IGNBRK  = 0x00000001,
                const BRKINT  = 0x00000002,
                const IGNPAR  = 0x00000004,
                const PARMRK  = 0x00000008,
                const INPCK   = 0x00000010,
                const ISTRIP  = 0x00000020,
                const INLCR   = 0x00000040,
                const IGNCR   = 0x00000080,
                const ICRNL   = 0x00000100,
                const IXON    = 0x00000200,
                const IXOFF   = 0x00000400,
                const IXANY   = 0x00000800,
                const IMAXBEL = 0x00002000,
                const IUTF8   = 0x00004000,
            }
        }

        bitflags! {
            flags OutputFlags: tcflag_t {
                const OPOST  = 0x00000001,
                const ONLCR  = 0x00000002,
                const OXTABS = 0x00000004,
                const ONOEOT = 0x00000008,
            }
        }

        bitflags! {
            flags ControlFlags: tcflag_t {
                const CIGNORE    = 0x00000001,
                const CSIZE      = 0x00000300,
                const CS5        = 0x00000000,
                const CS6        = 0x00000100,
                const CS7        = 0x00000200,
                const CS8        = 0x00000300,
                const CSTOPB     = 0x00000400,
                const CREAD      = 0x00000800,
                const PARENB     = 0x00001000,
                const PARODD     = 0x00002000,
                const HUPCL      = 0x00004000,
                const CLOCAL     = 0x00008000,
                const CCTS_OFLOW = 0x00010000,
                const CRTSCTS    = 0x00030000,
                const CRTS_IFLOW = 0x00020000,
                const CDTR_IFLOW = 0x00040000,
                const CDSR_OFLOW = 0x00080000,
                const CCAR_OFLOW = 0x00100000,
                const MDMBUF     = 0x00100000,
            }
        }

        bitflags! {
            flags LocalFlags: tcflag_t {
                const ECHOKE     = 0x00000001,
                const ECHOE      = 0x00000002,
                const ECHOK      = 0x00000004,
                const ECHO       = 0x00000008,
                const ECHONL     = 0x00000010,
                const ECHOPRT    = 0x00000020,
                const ECHOCTL    = 0x00000040,
                const ISIG       = 0x00000080,
                const ICANON     = 0x00000100,
                const ALTWERASE  = 0x00000200,
                const IEXTEN     = 0x00000400,
                const EXTPROC    = 0x00000800,
                const TOSTOP     = 0x00400000,
                const FLUSHO     = 0x00800000,
                const NOKERNINFO = 0x02000000,
                const PENDIN     = 0x20000000,
                const NOFLSH     = 0x80000000,
            }
        }

        pub const NL0: c_int  = 0x00000000;
        pub const NL1: c_int  = 0x00000100;
        pub const NL2: c_int  = 0x00000200;
        pub const NL3: c_int  = 0x00000300;
        pub const TAB0: c_int = 0x00000000;
        pub const TAB1: c_int = 0x00000400;
        pub const TAB2: c_int = 0x00000800;
        pub const TAB3: c_int = 0x00000004;
        pub const CR0: c_int  = 0x00000000;
        pub const CR1: c_int  = 0x00001000;
        pub const CR2: c_int  = 0x00002000;
        pub const CR3: c_int  = 0x00003000;
        pub const FF0: c_int  = 0x00000000;
        pub const FF1: c_int  = 0x00004000;
        pub const BS0: c_int  = 0x00000000;
        pub const BS1: c_int  = 0x00008000;
        pub const VT0: c_int  = 0x00000000;
        pub const VT1: c_int  = 0x00010000;

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Copy)]
        #[repr(C)]
        pub enum SetArg {
            TCSANOW   = 0,
            TCSADRAIN = 1,
            TCSAFLUSH = 2,
            TCSASOFT  = 16,
        }

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Copy)]
        #[repr(C)]
        pub enum FlushArg {
            TCIFLUSH  = 1,
            TCOFLUSH  = 2,
            TCIOFLUSH = 3,
        }

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Copy)]
        #[repr(C)]
        pub enum FlowArg {
            TCOOFF = 1,
            TCOON  = 2,
            TCIOFF = 3,
            TCION  = 4,
        }
    }

    #[cfg(target_os = "linux")]
    pub mod consts {
        use libc::{c_int, c_uint, c_uchar};

        pub type tcflag_t = c_uint;
        pub type cc_t = c_uchar;
        pub type speed_t = c_uint;

        #[repr(C)]
        #[derive(Copy)]
        pub struct Termios {
            pub c_iflag: InputFlags,
            pub c_oflag: OutputFlags,
            pub c_cflag: ControlFlags,
            pub c_lflag: LocalFlags,
            pub c_line: cc_t,
            pub c_cc: [cc_t; NCCS],
            pub c_ispeed: speed_t,
            pub c_ospeed: speed_t
        }

        pub const VEOF: usize     = 4;
        pub const VEOL: usize     = 11;
        pub const VEOL2: usize    = 16;
        pub const VERASE: usize   = 2;
        pub const VWERASE: usize  = 14;
        pub const VKILL: usize    = 3;
        pub const VREPRINT: usize = 12;
        pub const VINTR: usize    = 0;
        pub const VQUIT: usize    = 1;
        pub const VSUSP: usize    = 10;
        pub const VSTART: usize   = 8;
        pub const VSTOP: usize    = 9;
        pub const VLNEXT: usize   = 15;
        pub const VDISCARD: usize = 13;
        pub const VMIN: usize     = 6;
        pub const VTIME: usize    = 5;
        pub const NCCS: usize     = 32;

        bitflags! {
            flags InputFlags: tcflag_t {
                const IGNBRK  = 0x00000001,
                const BRKINT  = 0x00000002,
                const IGNPAR  = 0x00000004,
                const PARMRK  = 0x00000008,
                const INPCK   = 0x00000010,
                const ISTRIP  = 0x00000020,
                const INLCR   = 0x00000040,
                const IGNCR   = 0x00000080,
                const ICRNL   = 0x00000100,
                const IXON    = 0x00000400,
                const IXOFF   = 0x00001000,
                const IXANY   = 0x00000800,
                const IMAXBEL = 0x00002000,
                const IUTF8   = 0x00004000,
            }
        }

        bitflags! {
            flags OutputFlags: tcflag_t {
                const OPOST  = 0x00000001,
                const ONLCR  = 0x00000004,
            }
        }

        bitflags! {
            flags ControlFlags: tcflag_t {
                const CSIZE      = 0x00000030,
                const CS5        = 0x00000000,
                const CS6        = 0x00000010,
                const CS7        = 0x00000020,
                const CS8        = 0x00000030,
                const CSTOPB     = 0x00000040,
                const CREAD      = 0x00000080,
                const PARENB     = 0x00000100,
                const PARODD     = 0x00000200,
                const HUPCL      = 0x00000400,
                const CLOCAL     = 0x00000800,
                const CRTSCTS    = 0x80000000,
            }
        }

        bitflags! {
            flags LocalFlags: tcflag_t {
                const ECHOKE     = 0x00000800,
                const ECHOE      = 0x00000010,
                const ECHOK      = 0x00000020,
                const ECHO       = 0x00000008,
                const ECHONL     = 0x00000040,
                const ECHOPRT    = 0x00000400,
                const ECHOCTL    = 0x00000200,
                const ISIG       = 0x00000001,
                const ICANON     = 0x00000002,
                const IEXTEN     = 0x00008000,
                const EXTPROC    = 0x00010000,
                const TOSTOP     = 0x00000100,
                const FLUSHO     = 0x00001000,
                const PENDIN     = 0x00004000,
                const NOFLSH     = 0x00000080,
            }
        }

        pub const NL0: c_int  = 0x00000000;
        pub const NL1: c_int  = 0x00000100;
        pub const TAB0: c_int = 0x00000000;
        pub const TAB1: c_int = 0x00000800;
        pub const TAB2: c_int = 0x00001000;
        pub const TAB3: c_int = 0x00001800;
        pub const CR0: c_int  = 0x00000000;
        pub const CR1: c_int  = 0x00000200;
        pub const CR2: c_int  = 0x00000400;
        pub const CR3: c_int  = 0x00000600;
        pub const FF0: c_int  = 0x00000000;
        pub const FF1: c_int  = 0x00008000;
        pub const BS0: c_int  = 0x00000000;
        pub const BS1: c_int  = 0x00002000;
        pub const VT0: c_int  = 0x00000000;
        pub const VT1: c_int  = 0x00004000;

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Copy)]
        #[repr(C)]
        pub enum SetArg {
            TCSANOW   = 0,
            TCSADRAIN = 1,
            TCSAFLUSH = 2,
        }

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Copy)]
        #[repr(C)]
        pub enum FlushArg {
            TCIFLUSH  = 0,
            TCOFLUSH  = 1,
            TCIOFLUSH = 2,
        }

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Copy)]
        #[repr(C)]
        pub enum FlowArg {
            TCOOFF = 0,
            TCOON  = 1,
            TCIOFF = 2,
            TCION  = 3,
        }
    }
}

pub fn cfgetispeed(termios: &Termios) -> speed_t {
    unsafe {
        ffi::cfgetispeed(termios)
    }
}

pub fn cfgetospeed(termios: &Termios) -> speed_t {
    unsafe {
        ffi::cfgetospeed(termios)
    }
}

pub fn cfsetispeed(termios: &mut Termios, speed: speed_t) -> SysResult<()> {
    from_ffi(unsafe {
        ffi::cfsetispeed(termios, speed)
    })
}

pub fn cfsetospeed(termios: &mut Termios, speed: speed_t) -> SysResult<()> {
    from_ffi(unsafe {
        ffi::cfsetospeed(termios, speed)
    })
}

pub fn tcgetattr(fd: Fd, termios: &mut Termios) -> SysResult<()> {
    from_ffi(unsafe {
        ffi::tcgetattr(fd, termios)
    })
}

pub fn tcsetattr(fd: Fd,
                 actions: SetArg,
                 termios: &Termios) -> SysResult<()> {
    from_ffi(unsafe {
        ffi::tcsetattr(fd, actions as c_int, termios)
    })
}

pub fn tcdrain(fd: Fd) -> SysResult<()> {
    from_ffi(unsafe {
        ffi::tcdrain(fd)
    })
}

pub fn tcflow(fd: Fd, action: FlowArg) -> SysResult<()> {
    from_ffi(unsafe {
        ffi::tcflow(fd, action as c_int)
    })
}

pub fn tcflush(fd: Fd, action: FlushArg) -> SysResult<()> {
    from_ffi(unsafe {
        ffi::tcflush(fd, action as c_int)
    })
}

pub fn tcsendbreak(fd: Fd, action: c_int) -> SysResult<()> {
    from_ffi(unsafe {
        ffi::tcsendbreak(fd, action)
    })
}
