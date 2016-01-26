pub use libc::dev_t;
pub use libc::stat as FileStat;

use {Errno, Result, NixPath};
use libc::mode_t;
use std::mem;
use std::os::unix::io::RawFd;

mod ffi {
    use libc::{c_char, c_int, mode_t, dev_t};
    pub use libc::{stat, fstat, lstat};

    extern {
        pub fn mknod(pathname: *const c_char, mode: mode_t, dev: dev_t) -> c_int;
        pub fn umask(mask: mode_t) -> mode_t;
    }
}

bitflags!(
    flags SFlag: mode_t {
        const S_IFREG  = 0o100000,
        const S_IFCHR  = 0o020000,
        const S_IFBLK  = 0o060000,
        const S_IFIFO  = 0o010000,
        const S_IFSOCK = 0o140000
    }
);

bitflags! {
    flags Mode: mode_t {
        const S_IRWXU = 0o0700,
        const S_IRUSR = 0o0400,
        const S_IWUSR = 0o0200,
        const S_IXUSR = 0o0100,

        const S_IRWXG = 0o0070,
        const S_IRGRP = 0o0040,
        const S_IWGRP = 0o0020,
        const S_IXGRP = 0o0010,

        const S_IRWXO = 0o0007,
        const S_IROTH = 0o0004,
        const S_IWOTH = 0o0002,
        const S_IXOTH = 0o0001,

        const S_ISUID = 0o4000,
        const S_ISGID = 0o2000,
        const S_ISVTX = 0o1000,
    }
}

pub fn mknod<P: ?Sized + NixPath>(path: &P, kind: SFlag, perm: Mode, dev: dev_t) -> Result<()> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            ffi::mknod(cstr.as_ptr(), kind.bits | perm.bits() as mode_t, dev)
        }
    }));

    Errno::result(res).map(drop)
}

#[cfg(target_os = "linux")]
const MINORBITS: usize = 20;

#[cfg(target_os = "linux")]
pub fn mkdev(major: u64, minor: u64) -> dev_t {
    (major << MINORBITS) | minor
}

pub fn umask(mode: Mode) -> Mode {
    let prev = unsafe { ffi::umask(mode.bits() as mode_t) };
    Mode::from_bits(prev).expect("[BUG] umask returned invalid Mode")
}

pub fn stat<P: ?Sized + NixPath>(path: &P) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            ffi::stat(cstr.as_ptr(), &mut dst as *mut FileStat)
        }
    }));

    try!(Errno::result(res));

    Ok(dst)
}

pub fn lstat<P: ?Sized + NixPath>(path: &P) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            ffi::lstat(cstr.as_ptr(), &mut dst as *mut FileStat)
        }
    }));

    try!(Errno::result(res));

    Ok(dst)
}

pub fn fstat(fd: RawFd) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = unsafe { ffi::fstat(fd, &mut dst as *mut FileStat) };

    try!(Errno::result(res));

    Ok(dst)
}
