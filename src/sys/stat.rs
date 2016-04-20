pub use libc::dev_t;
pub use libc::stat as FileStat;

use {Errno, Result, NixPath};
use libc::{self, mode_t};
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
        const S_IFIFO = libc::S_IFIFO,
        const S_IFCHR = libc::S_IFCHR,
        const S_IFDIR = libc::S_IFDIR,
        const S_IFBLK = libc::S_IFBLK,
        const S_IFREG = libc::S_IFREG,
        const S_IFLNK = libc::S_IFLNK,
        const S_IFSOCK = libc::S_IFSOCK,
        const S_IFMT = libc::S_IFMT,
    }
);

bitflags! {
    flags Mode: mode_t {
        const S_IRWXU = libc::S_IRWXU,
        const S_IRUSR = libc::S_IRUSR,
        const S_IWUSR = libc::S_IWUSR,
        const S_IXUSR = libc::S_IXUSR,

        const S_IRWXG = libc::S_IRWXG,
        const S_IRGRP = libc::S_IRGRP,
        const S_IWGRP = libc::S_IWGRP,
        const S_IXGRP = libc::S_IXGRP,

        const S_IRWXO = libc::S_IRWXO,
        const S_IROTH = libc::S_IROTH,
        const S_IWOTH = libc::S_IWOTH,
        const S_IXOTH = libc::S_IXOTH,

        const S_ISUID = libc::S_ISUID as mode_t,
        const S_ISGID = libc::S_ISGID as mode_t,
        const S_ISVTX = libc::S_ISVTX as mode_t,
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
