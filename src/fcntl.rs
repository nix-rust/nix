use {Errno, Result, NixPath};
use libc::{self, c_int, c_uint};
use sys::stat::Mode;
use std::os::unix::io::RawFd;

#[cfg(any(target_os = "linux", target_os = "android"))]
use sys::uio::IoVec;  // For vmsplice

pub use self::consts::*;

// TODO: The remainder of the ffi module should be removed afer work on
// https://github.com/rust-lang/libc/issues/235 is resolved.
#[allow(dead_code)]
mod ffi {
    use libc::c_int;

    pub const F_ADD_SEALS: c_int = 1033;
    pub const F_GET_SEALS: c_int = 1034;
}

pub fn open<P: ?Sized + NixPath>(path: &P, oflag: OFlag, mode: Mode) -> Result<RawFd> {
    let fd = try!(path.with_nix_path(|cstr| {
        unsafe { libc::open(cstr.as_ptr(), oflag.bits(), mode.bits() as c_uint) }
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
    F_SETLK(&'a libc::flock),
    F_SETLKW(&'a libc::flock),
    F_GETLK(&'a mut libc::flock),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_OFD_SETLK(&'a libc::flock),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_OFD_SETLKW(&'a libc::flock),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_OFD_GETLK(&'a mut libc::flock),
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
            F_DUPFD(rawfd) => libc::fcntl(fd, libc::F_DUPFD, rawfd),
            F_DUPFD_CLOEXEC(rawfd) => libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, rawfd),
            F_GETFD => libc::fcntl(fd, libc::F_GETFD),
            F_SETFD(flag) => libc::fcntl(fd, libc::F_SETFD, flag.bits()),
            F_GETFL => libc::fcntl(fd, libc::F_GETFL),
            F_SETFL(flag) => libc::fcntl(fd, libc::F_SETFL, flag.bits()),
            F_SETLK(flock) => libc::fcntl(fd, libc::F_SETLK, flock),
            F_SETLKW(flock) => libc::fcntl(fd, libc::F_SETLKW, flock),
            F_GETLK(flock) => libc::fcntl(fd, libc::F_GETLK, flock),
            #[cfg(target_os = "linux")]
            F_ADD_SEALS(flag) => libc::fcntl(fd, ffi::F_ADD_SEALS, flag.bits()),
            #[cfg(target_os = "linux")]
            F_GET_SEALS => libc::fcntl(fd, ffi::F_GET_SEALS),
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
            LockShared => libc::flock(fd, libc::LOCK_SH),
            LockExclusive => libc::flock(fd, libc::LOCK_EX),
            Unlock => libc::flock(fd, libc::LOCK_UN),
            LockSharedNonblock => libc::flock(fd, libc::LOCK_SH | libc::LOCK_NB),
            LockExclusiveNonblock => libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB),
            UnlockNonblock => libc::flock(fd, libc::LOCK_UN | libc::LOCK_NB),
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
