use libc;
use fcntl::Fd;
use {NixResult, from_ffi};

pub use self::ffi::Winsize;
pub use self::IoctlArg::*;

mod ffi {
    use libc::c_ushort;

    #[derive(Copy, Debug)]
    pub struct Winsize {
        pub ws_row: c_ushort,
        pub ws_col: c_ushort,
        pub ws_xpixel: c_ushort,
        pub ws_ypixel: c_ushort,
    }

    #[cfg(target_os = "macos")]
    pub mod os {
        use libc::c_ulong;
        pub const TIOCGWINSZ: c_ulong = 0x40087468;
    }

    #[cfg(target_os = "linux")]
    pub mod os {
        use libc::c_int;
        pub const TIOCGWINSZ: c_int = 0x5413;
    }
}

pub enum IoctlArg<'a> {
    TIOCGWINSZ(&'a mut Winsize)
}

pub fn ioctl(fd: Fd, arg: IoctlArg) -> NixResult<()> {
    match arg {
        TIOCGWINSZ(&mut ref winsize) => {
            from_ffi(unsafe {
                libc::funcs::bsd44::ioctl(fd, ffi::os::TIOCGWINSZ, winsize)
            })
        }
    }
}
