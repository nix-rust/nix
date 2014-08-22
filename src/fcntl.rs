use std::path::Path;
use std::io::FilePermission;
use libc::{c_int, mode_t};
use errno::{SysResult, SysError};

pub use self::consts::*;
pub use self::ffi::flock;

pub type Fd = c_int;

mod ffi {
    pub use libc::{open, fcntl};
    pub use self::os::*;

    #[cfg(target_os = "linux")]
    mod os {
        use libc::{c_int, c_short, off_t, pid_t};

        pub struct flock {
            pub l_type: c_short,
            pub l_whence: c_short,
            pub l_start: off_t,
            pub l_len: off_t,
            pub l_pid: pid_t,

            // not actually here, but brings in line with freebsd
            pub l_sysid: c_int,
        }

        pub static F_DUPFD:         c_int = 0;
        pub static F_DUPFD_CLOEXEC: c_int = 1030;
        pub static F_GETFD:         c_int = 1;
        pub static F_SETFD:         c_int = 2;
        pub static F_GETFL:         c_int = 3;
        pub static F_SETFL:         c_int = 4;
        pub static F_SETLK:         c_int = 6;
        pub static F_SETLKW:        c_int = 7;
        pub static F_GETLK:         c_int = 5;
    }

    #[cfg(target_os = "macos")]
    #[cfg(target_os = "ios")]
    mod os {
        use libc::{c_int, c_short, off_t, pid_t};

        pub struct flock {
            pub l_start: off_t,
            pub l_len: off_t,
            pub l_pid: pid_t,
            pub l_type: c_short,
            pub l_whence: c_short,

            // not actually here, but brings in line with freebsd
            pub l_sysid: c_int,
        }

        pub static F_DUPFD:         c_int = 0;
        pub static F_DUPFD_CLOEXEC: c_int = 67;
        pub static F_GETFD:         c_int = 1;
        pub static F_SETFD:         c_int = 2;
        pub static F_GETFL:         c_int = 3;
        pub static F_SETFL:         c_int = 4;
        pub static F_SETLK:         c_int = 8;
        pub static F_SETLKW:        c_int = 9;
        pub static F_GETLK:         c_int = 7;
    }
}

pub fn open(path: &Path, oflag: OFlag, mode: FilePermission) -> SysResult<Fd> {
    let fd = unsafe { ffi::open(path.to_c_str().as_ptr(), oflag.bits(), mode.bits() as mode_t) };

    if fd < 0 {
        return Err(SysError::last());
    }

    Ok(fd)
}

pub enum FcntlArg<'a> {
    F_DUPFD(Fd),
    F_DUPFD_CLOEXEC(Fd),
    F_GETFD,
    F_SETFD(FdFlag), // FD_FLAGS
    F_GETFL,
    F_SETFL(OFlag), // O_NONBLOCK
    F_SETLK(&'a flock),
    F_SETLKW(&'a flock),
    F_GETLK(&'a mut flock),
    #[cfg(target_os = "linux")]
    F_OFD_SETLK(&'a flock),
    #[cfg(target_os = "linux")]
    F_OFD_SETLKW(&'a flock),
    #[cfg(target_os = "linux")]
    F_OFD_GETLK(&'a mut flock)

    // TODO: Rest of flags
}

// TODO: Figure out how to handle value fcntl returns
pub fn fcntl(fd: Fd, arg: FcntlArg) -> SysResult<()> {
    let res = unsafe {
        match arg {
            F_SETFD(flag) => ffi::fcntl(fd, ffi::F_SETFD, flag.bits()),
            F_SETFL(flag) => ffi::fcntl(fd, ffi::F_SETFL, flag.bits()),
            _ => unimplemented!()
        }
    };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(())
}

#[cfg(target_os = "linux")]
mod consts {
    use libc::c_int;

    bitflags!(
        flags OFlag: c_int {
            static O_ACCMODE   = 0o00000003,
            static O_RDONLY    = 0o00000000,
            static O_WRONLY    = 0o00000001,
            static O_RDWR      = 0o00000002,
            static O_CREAT     = 0o00000100,
            static O_EXCL      = 0o00000200,
            static O_NOCTTY    = 0o00000400,
            static O_TRUNC     = 0o00001000,
            static O_APPEND    = 0o00002000,
            static O_NONBLOCK  = 0o00004000,
            static O_DSYNC     = 0o00010000,
            static O_DIRECT    = 0o00040000,
            static O_LARGEFILE = 0o00100000,
            static O_DIRECTORY = 0o00200000,
            static O_NOFOLLOW  = 0o00400000,
            static O_NOATIME   = 0o01000000,
            static O_CLOEXEC   = 0o02000000,
            static O_SYNC      = 0o04000000,
            static O_PATH      = 0o10000000,
            static O_TMPFILE   = 0o20000000,
            static O_NDELAY    = O_NONBLOCK.bits
        }
    )

    bitflags!(
        flags FdFlag: c_int {
            static FD_CLOEXEC = 1
        }
    )
}

#[cfg(target_os = "macos")]
#[cfg(target_os = "ios")]
mod consts {
    use libc::c_int;

    bitflags!(
        flags OFlag: c_int {
            static O_ACCMODE   = 0x0000003,
            static O_RDONLY    = 0x0000000,
            static O_WRONLY    = 0x0000001,
            static O_RDWR      = 0x0000002,
            static O_CREAT     = 0x0000200,
            static O_EXCL      = 0x0000800,
            static O_NOCTTY    = 0x0020000,
            static O_TRUNC     = 0x0000400,
            static O_APPEND    = 0x0000008,
            static O_NONBLOCK  = 0x0000004,
            static O_DSYNC     = 0x0400000,
            static O_DIRECTORY = 0x0100000,
            static O_NOFOLLOW  = 0x0000100,
            static O_CLOEXEC   = 0x1000000,
            static O_SYNC      = 0x0000080,
            static O_NDELAY    = O_NONBLOCK.bits,
            static O_FSYNC     = O_SYNC.bits
        }
    )

    bitflags!(
        flags FdFlag: c_int {
            static FD_CLOEXEC = 1
        }
    )
}
