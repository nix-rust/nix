pub use libc::dev_t;
pub use libc::stat as FileStat;

use {Errno, Result, NixPath};
use fcntl::AtFlags;
use libc::{self, mode_t};
use std::mem;
use std::os::unix::io::RawFd;

libc_bitflags!(
    pub struct SFlag: mode_t {
        S_IFIFO;
        S_IFCHR;
        S_IFDIR;
        S_IFBLK;
        S_IFREG;
        S_IFLNK;
        S_IFSOCK;
        S_IFMT;
    }
);

libc_bitflags! {
    pub struct Mode: mode_t {
        S_IRWXU;
        S_IRUSR;
        S_IWUSR;
        S_IXUSR;
        S_IRWXG;
        S_IRGRP;
        S_IWGRP;
        S_IXGRP;
        S_IRWXO;
        S_IROTH;
        S_IWOTH;
        S_IXOTH;
        S_ISUID as mode_t;
        S_ISGID as mode_t;
        S_ISVTX as mode_t;
    }
}

pub fn mknod<P: ?Sized + NixPath>(path: &P, kind: SFlag, perm: Mode, dev: dev_t) -> Result<()> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            libc::mknod(cstr.as_ptr(), kind.bits | perm.bits() as mode_t, dev)
        }
    }));

    Errno::result(res).map(drop)
}

#[cfg(target_os = "linux")]
pub fn major(dev: dev_t) -> u64 {
    ((dev >> 32) & 0xfffff000) |
    ((dev >>  8) & 0x00000fff)
}

#[cfg(target_os = "linux")]
pub fn minor(dev: dev_t) -> u64 {
    ((dev >> 12) & 0xffffff00) |
    ((dev      ) & 0x000000ff)
}

#[cfg(target_os = "linux")]
pub fn makedev(major: u64, minor: u64) -> dev_t {
    ((major & 0xfffff000) << 32) |
    ((major & 0x00000fff) <<  8) |
    ((minor & 0xffffff00) << 12) |
    ((minor & 0x000000ff)      )
}

pub fn umask(mode: Mode) -> Mode {
    let prev = unsafe { libc::umask(mode.bits() as mode_t) };
    Mode::from_bits(prev).expect("[BUG] umask returned invalid Mode")
}

pub fn stat<P: ?Sized + NixPath>(path: &P) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            libc::stat(cstr.as_ptr(), &mut dst as *mut FileStat)
        }
    }));

    try!(Errno::result(res));

    Ok(dst)
}

pub fn lstat<P: ?Sized + NixPath>(path: &P) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            libc::lstat(cstr.as_ptr(), &mut dst as *mut FileStat)
        }
    }));

    try!(Errno::result(res));

    Ok(dst)
}

pub fn fstat(fd: RawFd) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = unsafe { libc::fstat(fd, &mut dst as *mut FileStat) };

    try!(Errno::result(res));

    Ok(dst)
}

pub fn fstatat<P: ?Sized + NixPath>(dirfd: RawFd, pathname: &P, f: AtFlags) -> Result<FileStat> {
    let mut dst = unsafe { mem::uninitialized() };
    let res = try!(pathname.with_nix_path(|cstr| {
        unsafe { libc::fstatat(dirfd, cstr.as_ptr(), &mut dst as *mut FileStat, f.bits() as libc::c_int) }
    }));

    try!(Errno::result(res));

    Ok(dst)
}

/// Change permissions of a file 
/// ([see chmod(2)](http://man7.org/linux/man-pages/man2/chmod.2.html)).
pub fn chmod<P: ?Sized + NixPath>(pathname: &P, mode: Mode) -> Result<()> {
    let res = try!(pathname.with_nix_path(|cstr| {
        unsafe { libc::chmod(cstr.as_ptr(), mode.bits()) }
    }));

    Errno::result(res).map(drop)
}

/// Change permissions of a file 
/// ([see fchmod(2)](http://man7.org/linux/man-pages/man2/fchmod.2.html)).
pub fn fchmod(fd: RawFd, mode: Mode) -> Result<()> {
    let res = unsafe { libc::fchmod(fd, mode.bits()) };

    Errno::result(res).map(drop)
}

/// Change permissions of a file 
/// ([see fchmodat(2)](http://man7.org/linux/man-pages/man2/fchmodat.2.html)).
pub fn fchmodat<P: ?Sized + NixPath>(dirfd: RawFd, pathname: &P, mode: Mode, flags: AtFlags) -> Result<()> {
    let res = try!(pathname.with_nix_path(|cstr| {
        unsafe {
            libc::fchmodat(dirfd,
                           cstr.as_ptr(),
                           mode.bits(),
                           flags.bits())
        }
    }));

    Errno::result(res).map(drop)
}
