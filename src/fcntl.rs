use {Error, Result, NixPath};
use errno::Errno;
use libc::{mode_t, c_int};
use sys::stat::Mode;
use std::os::unix::io::RawFd;

pub use self::consts::*;
pub use self::ffi::flock;

#[allow(dead_code)]
mod ffi {
    pub use libc::{open, fcntl};
    pub use self::os::*;
    pub use libc::funcs::bsd44::flock as libc_flock;
    pub use libc::consts::os::bsd44::{LOCK_SH, LOCK_EX, LOCK_NB, LOCK_UN};

    #[cfg(any(target_os = "linux", target_os = "android"))]
    mod os {
        use libc::{c_int, c_short, off_t, pid_t};

        #[repr(C)]
        #[derive(Clone, Copy, Default, Debug)]
        pub struct flock {
            pub l_type: c_short,
            pub l_whence: c_short,
            pub l_start: off_t,
            pub l_len: off_t,
            pub l_pid: pid_t,

            // not actually here, but brings in line with freebsd
            pub l_sysid: c_int,
        }

        pub const F_DUPFD:         c_int = 0;
        pub const F_DUPFD_CLOEXEC: c_int = 1030;
        pub const F_GETFD:         c_int = 1;
        pub const F_SETFD:         c_int = 2;
        pub const F_GETFL:         c_int = 3;
        pub const F_SETFL:         c_int = 4;
        pub const F_SETLK:         c_int = 6;
        pub const F_SETLKW:        c_int = 7;
        pub const F_GETLK:         c_int = 5;

        pub const F_ADD_SEALS:     c_int = 1033;
        pub const F_GET_SEALS:     c_int = 1034;

        pub const F_SEAL_SEAL:     c_int = 1;
        pub const F_SEAL_SHRINK:   c_int = 2;
        pub const F_SEAL_GROW:     c_int = 4;
        pub const F_SEAL_WRITE:    c_int = 8;
    }

    #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "ios", target_os = "openbsd"))]
    mod os {
        use libc::{c_int, c_short, off_t, pid_t};

        #[repr(C)]
        #[derive(Clone, Copy, Default, Debug)]
        pub struct flock {
            pub l_start: off_t,
            pub l_len: off_t,
            pub l_pid: pid_t,
            pub l_type: c_short,
            pub l_whence: c_short,

            // not actually here, but brings in line with freebsd
            pub l_sysid: c_int,
        }

        pub const F_DUPFD:         c_int = 0;
        pub const F_DUPFD_CLOEXEC: c_int = 67;
        pub const F_GETFD:         c_int = 1;
        pub const F_SETFD:         c_int = 2;
        pub const F_GETFL:         c_int = 3;
        pub const F_SETFL:         c_int = 4;
        pub const F_SETLK:         c_int = 8;
        pub const F_SETLKW:        c_int = 9;
        pub const F_GETLK:         c_int = 7;
    }
}

pub fn open<P: ?Sized + NixPath>(path: &P, oflag: OFlag, mode: Mode) -> Result<RawFd> {
    let fd = try!(path.with_nix_path(|cstr| {
        unsafe { ffi::open(cstr.as_ptr(), oflag.bits(), mode.bits() as mode_t) }
    }));

    if fd < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(fd)
}

pub enum FcntlArg<'a> {
    F_DUPFD(RawFd),
    F_DUPFD_CLOEXEC(RawFd),
    F_GETFD,
    F_SETFD(FdFlag), // FD_FLAGS
    F_GETFL,
    F_SETFL(OFlag), // O_NONBLOCK
    F_SETLK(&'a flock),
    F_SETLKW(&'a flock),
    F_GETLK(&'a mut flock),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_OFD_SETLK(&'a flock),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_OFD_SETLKW(&'a flock),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_OFD_GETLK(&'a mut flock),
    #[cfg(target_os = "linux")]
    F_ADD_SEALS(SealFlag),
    #[cfg(target_os = "linux")]
    F_GET_SEALS,

    // TODO: Rest of flags
}

// TODO: Figure out how to handle value fcntl returns
pub fn fcntl(fd: RawFd, arg: FcntlArg) -> Result<c_int> {
    use self::FcntlArg::*;

    let res = unsafe {
        match arg {
            F_DUPFD(rawfd) => ffi::fcntl(fd, ffi::F_DUPFD, rawfd),
            F_DUPFD_CLOEXEC(rawfd) => ffi::fcntl(fd, ffi::F_DUPFD_CLOEXEC, rawfd),
            F_GETFD => ffi::fcntl(fd, ffi::F_GETFD),
            F_SETFD(flag) => ffi::fcntl(fd, ffi::F_SETFD, flag.bits()),
            F_GETFL => ffi::fcntl(fd, ffi::F_GETFL),
            F_SETFL(flag) => ffi::fcntl(fd, ffi::F_SETFL, flag.bits()),
            F_SETLK(flock) => ffi::fcntl(fd, ffi::F_SETLK, flock),
            F_SETLKW(flock) => ffi::fcntl(fd, ffi::F_SETLKW, flock),
            F_GETLK(flock) => ffi::fcntl(fd, ffi::F_GETLK, flock),
            #[cfg(target_os = "linux")]
            F_ADD_SEALS(flag) => ffi::fcntl(fd, ffi::F_ADD_SEALS, flag.bits()),
            #[cfg(target_os = "linux")]
            F_GET_SEALS => ffi::fcntl(fd, ffi::F_GET_SEALS),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            _ => unimplemented!()
        }
    };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(res)
}

pub enum FlockArg {
    LockShared,
    LockExclusive,
    Unlock,
    LockSharedNonblock,
    LockExclusiveNonblock,
    UnlockNonblock,
}

pub fn flock(fd: RawFd, arg: FlockArg) -> Result<()> {
    use self::FlockArg::*;

    let res = unsafe {
        match arg {
            LockShared => ffi::libc_flock(fd, ffi::LOCK_SH),
            LockExclusive => ffi::libc_flock(fd, ffi::LOCK_EX),
            Unlock => ffi::libc_flock(fd, ffi::LOCK_UN),
            LockSharedNonblock => ffi::libc_flock(fd, ffi::LOCK_SH | ffi::LOCK_NB),
            LockExclusiveNonblock => ffi::libc_flock(fd, ffi::LOCK_EX | ffi::LOCK_NB),
            UnlockNonblock => ffi::libc_flock(fd, ffi::LOCK_UN | ffi::LOCK_NB),
        }
    };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod consts {
    use libc::c_int;

    bitflags!(
        flags OFlag: c_int {
            const O_ACCMODE   = 0o00000003,
            const O_RDONLY    = 0o00000000,
            const O_WRONLY    = 0o00000001,
            const O_RDWR      = 0o00000002,
            const O_CREAT     = 0o00000100,
            const O_EXCL      = 0o00000200,
            const O_NOCTTY    = 0o00000400,
            const O_TRUNC     = 0o00001000,
            const O_APPEND    = 0o00002000,
            const O_NONBLOCK  = 0o00004000,
            const O_DSYNC     = 0o00010000,
            const O_DIRECT    = 0o00040000,
            const O_LARGEFILE = 0o00100000,
            const O_DIRECTORY = 0o00200000,
            const O_NOFOLLOW  = 0o00400000,
            const O_NOATIME   = 0o01000000,
            const O_CLOEXEC   = 0o02000000,
            const O_SYNC      = 0o04000000,
            const O_PATH      = 0o10000000,
            const O_TMPFILE   = 0o20000000,
            const O_NDELAY    = O_NONBLOCK.bits
        }
    );

    bitflags!(
        flags FdFlag: c_int {
            const FD_CLOEXEC = 1
        }
    );

    bitflags!(
        flags SealFlag: c_int {
            const F_SEAL_SEAL = 1,
            const F_SEAL_SHRINK = 2,
            const F_SEAL_GROW = 4,
            const F_SEAL_WRITE = 8,
        }
    );

}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod consts {
    use libc::c_int;

    bitflags!(
        flags OFlag: c_int {
            const O_ACCMODE   = 0x0000003,
            const O_RDONLY    = 0x0000000,
            const O_WRONLY    = 0x0000001,
            const O_RDWR      = 0x0000002,
            const O_CREAT     = 0x0000200,
            const O_EXCL      = 0x0000800,
            const O_NOCTTY    = 0x0020000,
            const O_TRUNC     = 0x0000400,
            const O_APPEND    = 0x0000008,
            const O_NONBLOCK  = 0x0000004,
            const O_DSYNC     = 0x0400000,
            const O_DIRECTORY = 0x0100000,
            const O_NOFOLLOW  = 0x0000100,
            const O_CLOEXEC   = 0x1000000,
            const O_SYNC      = 0x0000080,
            const O_NDELAY    = O_NONBLOCK.bits,
            const O_FSYNC     = O_SYNC.bits
        }
    );

    bitflags!(
        flags FdFlag: c_int {
            const FD_CLOEXEC = 1
        }
    );
}

#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
mod consts {
    use libc::c_int;

    bitflags!(
        flags OFlag: c_int {
            const O_ACCMODE   = 0x0000003,
            const O_RDONLY    = 0x0000000,
            const O_WRONLY    = 0x0000001,
            const O_RDWR      = 0x0000002,
            const O_CREAT     = 0x0000200,
            const O_EXCL      = 0x0000800,
            const O_NOCTTY    = 0x0008000,
            const O_TRUNC     = 0x0000400,
            const O_APPEND    = 0x0000008,
            const O_NONBLOCK  = 0x0000004,
            const O_DIRECTORY = 0x0020000,
            const O_NOFOLLOW  = 0x0000100,
            const O_CLOEXEC   = 0x0100000,
            const O_SYNC      = 0x0000080,
            const O_NDELAY    = O_NONBLOCK.bits,
            const O_FSYNC     = O_SYNC.bits,
            const O_SHLOCK    = 0x0000080,
            const O_EXLOCK    = 0x0000020,
            const O_DIRECT    = 0x0010000,
            const O_EXEC      = 0x0040000,
            const O_TTY_INIT  = 0x0080000
        }
    );

    bitflags!(
        flags FdFlag: c_int {
            const FD_CLOEXEC = 1
        }
    );
}
