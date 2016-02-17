use {Errno, Result, NixPath};
use libc::{c_int, c_uint};
use sys::stat::Mode;
use std::os::unix::io::RawFd;

#[cfg(any(target_os = "linux", target_os = "android"))]
use sys::uio::IoVec;  // For vmsplice
#[cfg(any(target_os = "linux", target_os = "android"))]
use libc;

pub use self::consts::*;
pub use self::ffi::flock;

#[allow(dead_code)]
mod ffi {
    pub use libc::{open, fcntl};
    pub use self::os::*;
    pub use libc::flock as libc_flock;
    pub use libc::{LOCK_SH, LOCK_EX, LOCK_NB, LOCK_UN};

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

    #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
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
        #[cfg(not(any(target_os = "dragonfly", target_os = "netbsd")))]
        pub const F_DUPFD_CLOEXEC: c_int = 67;
        #[cfg(target_os = "dragonfly")]
        pub const F_DUPFD_CLOEXEC: c_int = 17;
        #[cfg(target_os = "netbsd")]
        pub const F_DUPFD_CLOEXEC: c_int = 12;
        pub const F_GETFD:         c_int = 1;
        pub const F_SETFD:         c_int = 2;
        pub const F_GETFL:         c_int = 3;
        pub const F_SETFL:         c_int = 4;
        #[cfg(target_os = "netbsd")]
        pub const F_GETOWN:        c_int = 5;
        #[cfg(target_os = "netbsd")]
        pub const F_SETOWN:        c_int = 6;
        pub const F_GETLK:         c_int = 7;
        pub const F_SETLK:         c_int = 8;
        pub const F_SETLKW:        c_int = 9;

        #[cfg(target_os = "netbsd")]
        pub const F_CLOSEM:        c_int = 10;
        #[cfg(target_os = "netbsd")]
        pub const F_MAXFD:         c_int = 11;
        #[cfg(target_os = "netbsd")]
        pub const F_GETNOSIGPIPE:  c_int = 13;
        #[cfg(target_os = "netbsd")]
        pub const F_SETNOSIGPIPE:  c_int = 14;
    }
}

pub fn open<P: ?Sized + NixPath>(path: &P, oflag: OFlag, mode: Mode) -> Result<RawFd> {
    let fd = try!(path.with_nix_path(|cstr| {
        unsafe { ffi::open(cstr.as_ptr(), oflag.bits(), mode.bits() as c_uint) }
    }));

    Errno::result(fd)
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

    Errno::result(res)
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

    Errno::result(res).map(drop)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn splice(fd_in: RawFd, off_in: Option<&mut libc::loff_t>,
          fd_out: RawFd, off_out: Option<&mut libc::loff_t>,
          len: usize, flags: SpliceFFlags) -> Result<usize> {
    use std::ptr;
    let off_in = off_in.map(|offset| offset as *mut _).unwrap_or(ptr::null_mut());
    let off_out = off_out.map(|offset| offset as *mut _).unwrap_or(ptr::null_mut());

    let ret = unsafe { libc::splice(fd_in, off_in, fd_out, off_out, len, flags.bits()) };
    Errno::result(ret).map(|r| r as usize)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn tee(fd_in: RawFd, fd_out: RawFd, len: usize, flags: SpliceFFlags) -> Result<usize> {
    let ret = unsafe { libc::tee(fd_in, fd_out, len, flags.bits()) };
    Errno::result(ret).map(|r| r as usize)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn vmsplice(fd: RawFd, iov: &[IoVec<&[u8]>], flags: SpliceFFlags) -> Result<usize> {
    let ret = unsafe {
        libc::vmsplice(fd, iov.as_ptr() as *const libc::iovec, iov.len(), flags.bits())
    };
    Errno::result(ret).map(|r| r as usize)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod consts {
    use libc::{self, c_int, c_uint};

    bitflags! {
        flags SpliceFFlags: c_uint {
            const SPLICE_F_MOVE = libc::SPLICE_F_MOVE,
            const SPLICE_F_NONBLOCK = libc::SPLICE_F_NONBLOCK,
            const SPLICE_F_MORE = libc::SPLICE_F_MORE,
            const SPLICE_F_GIFT = libc::SPLICE_F_GIFT,
        }
    }

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

#[cfg(target_os = "netbsd")]
mod consts {
    use libc::c_int;

    bitflags!(
        flags OFlag: c_int {
            const O_ACCMODE   = 0x0000003,
            const O_RDONLY    = 0x0000000,
            const O_WRONLY    = 0x0000001,
            const O_RDWR      = 0x0000002,
            const O_NONBLOCK  = 0x0000004,
            const O_APPEND    = 0x0000008,
            const O_SHLOCK    = 0x0000010,
            const O_EXLOCK    = 0x0000020,
            const O_ASYNC     = 0x0000040,
            const O_SYNC      = 0x0000080,
            const O_NOFOLLOW  = 0x0000100,
            const O_CREAT     = 0x0000200,
            const O_TRUNC     = 0x0000400,
            const O_EXCL      = 0x0000800,
            const O_NOCTTY    = 0x0008000,
            const O_DSYNC     = 0x0010000,
            const O_RSYNC     = 0x0020000,
            const O_ALT_IO    = 0x0040000,
            const O_DIRECT    = 0x0080000,
            const O_NOSIGPIPE = 0x0100000,
            const O_DIRECTORY = 0x0200000,
            const O_CLOEXEC   = 0x0400000,
            const O_SEARCH    = 0x0800000,
            const O_FSYNC     = O_SYNC.bits,
            const O_NDELAY    = O_NONBLOCK.bits,
        }
    );

    bitflags!(
        flags FdFlag: c_int {
            const FD_CLOEXEC = 1
        }
    );
}

#[cfg(target_os = "dragonfly")]
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
            const O_DIRECTORY = 0x8000000, // different from FreeBSD!
            const O_NOFOLLOW  = 0x0000100,
            const O_CLOEXEC   = 0x0020000, // different from FreeBSD!
            const O_SYNC      = 0x0000080,
            const O_NDELAY    = O_NONBLOCK.bits,
            const O_FSYNC     = O_SYNC.bits,
            const O_SHLOCK    = 0x0000010, // different from FreeBSD!
            const O_EXLOCK    = 0x0000020,
            const O_DIRECT    = 0x0010000,
        }
    );

    bitflags!(
        flags FdFlag: c_int {
            const FD_CLOEXEC = 1
        }
    );
}
