use {Error, Errno, Result, NixPath};
use libc::{self, c_int, c_uint, c_char, size_t, ssize_t};
use sys::stat::Mode;
use std::os::unix::io::RawFd;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

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

libc_bitflags!{
    pub struct AtFlags: c_int {
        AT_SYMLINK_NOFOLLOW;
        #[cfg(any(target_os = "linux", target_os = "android"))]
        AT_NO_AUTOMOUNT;
        #[cfg(any(target_os = "linux", target_os = "android"))]
        AT_EMPTY_PATH;
    }
}

pub fn open<P: ?Sized + NixPath>(path: &P, oflag: OFlag, mode: Mode) -> Result<RawFd> {
    let fd = try!(path.with_nix_path(|cstr| {
        unsafe { libc::open(cstr.as_ptr(), oflag.bits(), mode.bits() as c_uint) }
    }));

    Errno::result(fd)
}

pub fn openat<P: ?Sized + NixPath>(dirfd: RawFd, path: &P, oflag: OFlag, mode: Mode) -> Result<RawFd> {
    let fd = try!(path.with_nix_path(|cstr| {
        unsafe { libc::openat(dirfd, cstr.as_ptr(), oflag.bits(), mode.bits() as c_uint) }
    }));
    Errno::result(fd)
}

fn wrap_readlink_result<'a>(buffer: &'a mut[u8], res: ssize_t)
  -> Result<&'a OsStr> {
    match Errno::result(res) {
        Err(err) => Err(err),
        Ok(len) => {
            if (len as usize) >= buffer.len() {
                Err(Error::Sys(Errno::ENAMETOOLONG))
            } else {
                Ok(OsStr::from_bytes(&buffer[..(len as usize)]))
            }
        }
    }
}

pub fn readlink<'a, P: ?Sized + NixPath>(path: &P, buffer: &'a mut [u8]) -> Result<&'a OsStr> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe { libc::readlink(cstr.as_ptr(), buffer.as_mut_ptr() as *mut c_char, buffer.len() as size_t) }
    }));

    wrap_readlink_result(buffer, res)
}


pub fn readlinkat<'a, P: ?Sized + NixPath>(dirfd: RawFd, path: &P, buffer: &'a mut [u8]) -> Result<&'a OsStr> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe { libc::readlinkat(dirfd, cstr.as_ptr(), buffer.as_mut_ptr() as *mut c_char, buffer.len() as size_t) }
    }));

    wrap_readlink_result(buffer, res)
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
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    F_FULLFSYNC,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_GETPIPE_SZ,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_SETPIPE_SZ(libc::c_int),

    // TODO: Rest of flags
}
pub use self::FcntlArg::*;

// TODO: Figure out how to handle value fcntl returns
pub fn fcntl(fd: RawFd, arg: FcntlArg) -> Result<c_int> {
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
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            F_FULLFSYNC => libc::fcntl(fd, libc::F_FULLFSYNC),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            F_GETPIPE_SZ => libc::fcntl(fd, libc::F_GETPIPE_SZ),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            F_SETPIPE_SZ(size) => libc::fcntl(fd, libc::F_SETPIPE_SZ, size),
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

    libc_bitflags! {
        pub struct SpliceFFlags: c_uint {
            SPLICE_F_MOVE;
            SPLICE_F_NONBLOCK;
            SPLICE_F_MORE;
            SPLICE_F_GIFT;
        }
    }

    bitflags!(
        pub struct OFlag: c_int {
            const O_ACCMODE   = libc::O_ACCMODE;
            const O_RDONLY    = libc::O_RDONLY;
            const O_WRONLY    = libc::O_WRONLY;
            const O_RDWR      = libc::O_RDWR;
            const O_CREAT     = libc::O_CREAT;
            const O_EXCL      = libc::O_EXCL;
            const O_NOCTTY    = libc::O_NOCTTY;
            const O_TRUNC     = libc::O_TRUNC;
            const O_APPEND    = libc::O_APPEND;
            const O_NONBLOCK  = libc::O_NONBLOCK;
            const O_DSYNC     = libc::O_DSYNC;
            const O_DIRECT    = libc::O_DIRECT;
            const O_LARGEFILE = 0o00100000;
            const O_DIRECTORY = libc::O_DIRECTORY;
            const O_NOFOLLOW  = libc::O_NOFOLLOW;
            const O_NOATIME   = 0o01000000;
            const O_CLOEXEC   = libc::O_CLOEXEC;
            const O_SYNC      = libc::O_SYNC;
            const O_PATH      = 0o10000000;
            const O_TMPFILE   = libc::O_TMPFILE;
            const O_NDELAY    = libc::O_NDELAY;
        }
    );

    libc_bitflags!(
        pub struct FdFlag: c_int {
            FD_CLOEXEC;
        }
    );

    bitflags!(
        pub struct SealFlag: c_int {
            const F_SEAL_SEAL = 1;
            const F_SEAL_SHRINK = 2;
            const F_SEAL_GROW = 4;
            const F_SEAL_WRITE = 8;
        }
    );

}

#[cfg(any(target_os = "netbsd", target_os = "dragonfly", target_os = "openbsd",
          target_os = "freebsd", target_os = "macos", target_os = "ios"))]
mod consts {
    use libc::{self,c_int};

    libc_bitflags!(
        pub struct OFlag: c_int {
            O_ACCMODE;
            O_RDONLY;
            O_WRONLY;
            O_RDWR;
            O_NONBLOCK;
            O_APPEND;
            O_SHLOCK;
            O_EXLOCK;
            O_ASYNC;
            O_SYNC;
            O_NOFOLLOW;
            O_CREAT;
            O_TRUNC;
            O_EXCL;
            O_NOCTTY;
            O_DIRECTORY;
            O_CLOEXEC;
            O_FSYNC;
            O_NDELAY;
            #[cfg(any(target_os = "netbsd", target_os = "openbsd", target_os = "macos",
                      target_os = "ios"))]
            O_DSYNC;
            #[cfg(any(target_os = "netbsd", target_os = "dragonfly", target_os = "freebsd"))]
            O_DIRECT;
            #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
            O_RSYNC;
            #[cfg(target_os = "freebsd")]
            O_EXEC;
            #[cfg(target_os = "freebsd")]
            O_TTY_INIT;
            #[cfg(target_os = "netbsd")]
            O_ALT_IO;
            #[cfg(target_os = "netbsd")]
            O_NOSIGPIPE;
            #[cfg(target_os = "netbsd")]
            O_SEARCH;
        }
    );

    libc_bitflags!(
        pub struct FdFlag: c_int {
            FD_CLOEXEC;
        }
    );
}
