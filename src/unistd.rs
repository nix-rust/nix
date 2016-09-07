//! Standard symbolic constants and types
//!
use {Errno, Error, Result, NixPath};
use fcntl::{fcntl, OFlag, O_NONBLOCK, O_CLOEXEC, FD_CLOEXEC};
use fcntl::FcntlArg::{F_SETFD, F_SETFL};
use libc::{self, c_char, c_void, c_int, c_uint, size_t, pid_t, off_t, uid_t, gid_t, mode_t};
use std::mem;
use std::ffi::{CString, CStr, OsString};
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::os::unix::io::RawFd;
use void::Void;
use sys::stat::Mode;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::linux::*;

#[derive(Clone, Copy)]
pub enum ForkResult {
    Parent {
        child: pid_t
    },
    Child
}

impl ForkResult {
    #[inline]
    pub fn is_child(&self) -> bool {
        match *self {
            ForkResult::Child => true,
            _ => false
        }
    }

    #[inline]
    pub fn is_parent(&self) -> bool {
        !self.is_child()
    }
}

#[inline]
pub fn fork() -> Result<ForkResult> {
    use self::ForkResult::*;
    let res = unsafe { libc::fork() };

    Errno::result(res).map(|res| match res {
        0 => Child,
        res => Parent { child: res }
    })
}

#[inline]
pub fn getpid() -> pid_t {
    unsafe { libc::getpid() } // no error handling, according to man page: "These functions are always successful."
}
#[inline]
pub fn getppid() -> pid_t {
    unsafe { libc::getppid() } // no error handling, according to man page: "These functions are always successful."
}
#[inline]
pub fn setpgid(pid: pid_t, pgid: pid_t) -> Result<()> {
    let res = unsafe { libc::setpgid(pid, pgid) };
    Errno::result(res).map(drop)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[inline]
pub fn gettid() -> pid_t {
    unsafe { libc::syscall(libc::SYS_gettid) as pid_t }    // no error handling, according to man page: "These functions are always successful."
}

#[inline]
pub fn dup(oldfd: RawFd) -> Result<RawFd> {
    let res = unsafe { libc::dup(oldfd) };

    Errno::result(res)
}

#[inline]
pub fn dup2(oldfd: RawFd, newfd: RawFd) -> Result<RawFd> {
    let res = unsafe { libc::dup2(oldfd, newfd) };

    Errno::result(res)
}

pub fn dup3(oldfd: RawFd, newfd: RawFd, flags: OFlag) -> Result<RawFd> {
    dup3_polyfill(oldfd, newfd, flags)
}

#[inline]
fn dup3_polyfill(oldfd: RawFd, newfd: RawFd, flags: OFlag) -> Result<RawFd> {
    if oldfd == newfd {
        return Err(Error::Sys(Errno::EINVAL));
    }

    let fd = try!(dup2(oldfd, newfd));

    if flags.contains(O_CLOEXEC) {
        if let Err(e) = fcntl(fd, F_SETFD(FD_CLOEXEC)) {
            let _ = close(fd);
            return Err(e);
        }
    }

    Ok(fd)
}

#[inline]
pub fn chdir<P: ?Sized + NixPath>(path: &P) -> Result<()> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe { libc::chdir(cstr.as_ptr()) }
    }));

    Errno::result(res).map(drop)
}

/// Creates new directory `path` with access rights `mode`.
///
/// # Errors
///
/// There are several situations where mkdir might fail:
///
/// - current user has insufficient rights in the parent directory
/// - the path already exists
/// - the path name is too long (longer than `PATH_MAX`, usually 4096 on linux, 1024 on OS X)
///
/// For a full list consult 
/// [man mkdir(2)](http://man7.org/linux/man-pages/man2/mkdir.2.html#ERRORS)
///
/// # Example
///
/// ```rust
/// extern crate tempdir;
/// extern crate nix;
///
/// use nix::unistd;
/// use nix::sys::stat;
/// use tempdir::TempDir;
///
/// fn main() {
///     let mut tmp_dir = TempDir::new("test_mkdir").unwrap().into_path();
///     tmp_dir.push("new_dir");
///
///     // create new directory and give read, write and execute rights to the owner
///     match unistd::mkdir(&tmp_dir, stat::S_IRWXU) {
///        Ok(_) => println!("created {:?}", tmp_dir),
///        Err(err) => println!("Error creating directory: {}", err),
///     }
/// }
/// ```
#[inline]
pub fn mkdir<P: ?Sized + NixPath>(path: &P, mode: Mode) -> Result<()> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe { libc::mkdir(cstr.as_ptr(), mode.bits() as mode_t) }
    }));

    Errno::result(res).map(drop)
}

/// Returns the current directory as a PathBuf
///
/// Err is returned if the current user doesn't have the permission to read or search a component
/// of the current path.
///
/// # Example
///
/// ```rust
/// extern crate nix;
///
/// use nix::unistd;
///
/// fn main() {
///     // assume that we are allowed to get current directory
///     let dir = unistd::getcwd().unwrap();
///     println!("The current directory is {:?}", dir);
/// }
/// ```
#[inline]
pub fn getcwd() -> Result<PathBuf> {
    let mut buf = Vec::with_capacity(512);
    loop {
        unsafe {
            let ptr = buf.as_mut_ptr() as *mut libc::c_char;

            // The buffer must be large enough to store the absolute pathname plus
            // a terminating null byte, or else null is returned.
            // To safely handle this we start with a reasonable size (512 bytes)
            // and double the buffer size upon every error
            if !libc::getcwd(ptr, buf.capacity()).is_null() {
                let len = CStr::from_ptr(buf.as_ptr() as *const libc::c_char).to_bytes().len();
                buf.set_len(len);
                buf.shrink_to_fit();
                return Ok(PathBuf::from(OsString::from_vec(buf)));
            } else {
                let error = Errno::last();
                // ERANGE means buffer was too small to store directory name
                if error != Errno::ERANGE {
                    return Err(Error::Sys(error));
                }
            }

            // Trigger the internal buffer resizing logic of `Vec` by requiring
            // more space than the current capacity.
            let cap = buf.capacity();
            buf.set_len(cap);
            buf.reserve(1);
        }
    }
}

#[inline]
pub fn chown<P: ?Sized + NixPath>(path: &P, owner: Option<uid_t>, group: Option<gid_t>) -> Result<()> {
    let res = try!(path.with_nix_path(|cstr| {
        // According to the POSIX specification, -1 is used to indicate that
        // owner and group, respectively, are not to be changed. Since uid_t and
        // gid_t are unsigned types, we use wrapping_sub to get '-1'.
        unsafe { libc::chown(cstr.as_ptr(),
                             owner.unwrap_or((0 as uid_t).wrapping_sub(1)),
                             group.unwrap_or((0 as gid_t).wrapping_sub(1))) }
    }));

    Errno::result(res).map(drop)
}

fn to_exec_array(args: &[CString]) -> Vec<*const c_char> {
    use std::ptr;
    use libc::c_char;

    let mut args_p: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();
    args_p.push(ptr::null());
    args_p
}

#[inline]
pub fn execv(path: &CString, argv: &[CString]) -> Result<Void> {
    let args_p = to_exec_array(argv);

    unsafe {
        libc::execv(path.as_ptr(), args_p.as_ptr())
    };

    Err(Error::Sys(Errno::last()))
}

#[inline]
pub fn execve(path: &CString, args: &[CString], env: &[CString]) -> Result<Void> {
    let args_p = to_exec_array(args);
    let env_p = to_exec_array(env);

    unsafe {
        libc::execve(path.as_ptr(), args_p.as_ptr(), env_p.as_ptr())
    };

    Err(Error::Sys(Errno::last()))
}

#[inline]
pub fn execvp(filename: &CString, args: &[CString]) -> Result<Void> {
    let args_p = to_exec_array(args);

    unsafe {
        libc::execvp(filename.as_ptr(), args_p.as_ptr())
    };

    Err(Error::Sys(Errno::last()))
}

pub fn daemon(nochdir: bool, noclose: bool) -> Result<()> {
    let res = unsafe { libc::daemon(nochdir as c_int, noclose as c_int) };
    Errno::result(res).map(drop)
}

pub fn sethostname(name: &[u8]) -> Result<()> {
    // Handle some differences in type of the len arg across platforms.
    cfg_if! {
        if #[cfg(any(target_os = "dragonfly",
                     target_os = "freebsd",
                     target_os = "ios",
                     target_os = "macos", ))] {
            type sethostname_len_t = c_int;
        } else {
            type sethostname_len_t = size_t;
        }
    }
    let ptr = name.as_ptr() as *const c_char;
    let len = name.len() as sethostname_len_t;

    let res = unsafe { libc::sethostname(ptr, len) };
    Errno::result(res).map(drop)
}

pub fn gethostname(name: &mut [u8]) -> Result<()> {
    let ptr = name.as_mut_ptr() as *mut c_char;
    let len = name.len() as size_t;

    let res = unsafe { libc::gethostname(ptr, len) };
    Errno::result(res).map(drop)
}

pub fn close(fd: RawFd) -> Result<()> {
    let res = unsafe { libc::close(fd) };
    Errno::result(res).map(drop)
}

pub fn read(fd: RawFd, buf: &mut [u8]) -> Result<usize> {
    let res = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t) };

    Errno::result(res).map(|r| r as usize)
}

pub fn write(fd: RawFd, buf: &[u8]) -> Result<usize> {
    let res = unsafe { libc::write(fd, buf.as_ptr() as *const c_void, buf.len() as size_t) };

    Errno::result(res).map(|r| r as usize)
}

pub enum Whence {
    SeekSet,
    SeekCur,
    SeekEnd,
    SeekData,
    SeekHole
}

impl Whence {
    fn to_libc_type(&self) -> c_int {
        match self {
            &Whence::SeekSet => libc::SEEK_SET,
            &Whence::SeekCur => libc::SEEK_CUR,
            &Whence::SeekEnd => libc::SEEK_END,
            &Whence::SeekData => 3,
            &Whence::SeekHole => 4
        }
    }

}

pub fn lseek(fd: RawFd, offset: libc::off_t, whence: Whence) -> Result<libc::off_t> {
    let res = unsafe { libc::lseek(fd, offset, whence.to_libc_type()) };

    Errno::result(res).map(|r| r as libc::off_t)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn lseek64(fd: RawFd, offset: libc::off64_t, whence: Whence) -> Result<libc::off64_t> {
    let res = unsafe { libc::lseek64(fd, offset, whence.to_libc_type()) };

    Errno::result(res).map(|r| r as libc::off64_t)
}

pub fn pipe() -> Result<(RawFd, RawFd)> {
    unsafe {
        let mut fds: [c_int; 2] = mem::uninitialized();

        let res = libc::pipe(fds.as_mut_ptr());

        try!(Errno::result(res));

        Ok((fds[0], fds[1]))
    }
}

pub fn pipe2(flags: OFlag) -> Result<(RawFd, RawFd)> {
    unsafe {
        let mut fds: [c_int; 2] = mem::uninitialized();

        let res = libc::pipe(fds.as_mut_ptr());

        try!(Errno::result(res));

        try!(pipe2_setflags(fds[0], fds[1], flags));

        Ok((fds[0], fds[1]))
    }
}

fn pipe2_setflags(fd1: RawFd, fd2: RawFd, flags: OFlag) -> Result<()> {
    let mut res = Ok(0);

    if flags.contains(O_CLOEXEC) {
        res = res
            .and_then(|_| fcntl(fd1, F_SETFD(FD_CLOEXEC)))
            .and_then(|_| fcntl(fd2, F_SETFD(FD_CLOEXEC)));
    }

    if flags.contains(O_NONBLOCK) {
        res = res
            .and_then(|_| fcntl(fd1, F_SETFL(O_NONBLOCK)))
            .and_then(|_| fcntl(fd2, F_SETFL(O_NONBLOCK)));
    }

    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            let _ = close(fd1);
            let _ = close(fd2);
            Err(e)
        }
    }
}

pub fn ftruncate(fd: RawFd, len: off_t) -> Result<()> {
    Errno::result(unsafe { libc::ftruncate(fd, len) }).map(drop)
}

pub fn isatty(fd: RawFd) -> Result<bool> {
    use libc;

    unsafe {
        // ENOTTY means `fd` is a valid file descriptor, but not a TTY, so
        // we return `Ok(false)`
        if libc::isatty(fd) == 1 {
            Ok(true)
        } else {
            match Errno::last() {
                Errno::ENOTTY => Ok(false),
                err => Err(Error::Sys(err)),
            }
        }
    }
}

pub fn unlink<P: ?Sized + NixPath>(path: &P) -> Result<()> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe {
            libc::unlink(cstr.as_ptr())
        }
    }));
    Errno::result(res).map(drop)
}

#[inline]
pub fn chroot<P: ?Sized + NixPath>(path: &P) -> Result<()> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe { libc::chroot(cstr.as_ptr()) }
    }));

    Errno::result(res).map(drop)
}

#[inline]
pub fn fsync(fd: RawFd) -> Result<()> {
    let res = unsafe { libc::fsync(fd) };

    Errno::result(res).map(drop)
}

// `fdatasync(2) is in POSIX, but in libc it is only defined in `libc::notbsd`.
// TODO: exclude only Apple systems after https://github.com/rust-lang/libc/pull/211
#[cfg(any(target_os = "linux",
          target_os = "android",
          target_os = "emscripten"))]
#[inline]
pub fn fdatasync(fd: RawFd) -> Result<()> {
    let res = unsafe { libc::fdatasync(fd) };

    Errno::result(res).map(drop)
}

// POSIX requires that getuid, geteuid, getgid, getegid are always successful,
// so no need to check return value or errno. See:
//   - http://pubs.opengroup.org/onlinepubs/9699919799/functions/getuid.html
//   - http://pubs.opengroup.org/onlinepubs/9699919799/functions/geteuid.html
//   - http://pubs.opengroup.org/onlinepubs/9699919799/functions/getgid.html
//   - http://pubs.opengroup.org/onlinepubs/9699919799/functions/geteuid.html
#[inline]
pub fn getuid() -> uid_t {
    unsafe { libc::getuid() }
}

#[inline]
pub fn geteuid() -> uid_t {
    unsafe { libc::geteuid() }
}

#[inline]
pub fn getgid() -> gid_t {
    unsafe { libc::getgid() }
}

#[inline]
pub fn getegid() -> gid_t {
    unsafe { libc::getegid() }
}

#[inline]
pub fn setuid(uid: uid_t) -> Result<()> {
    let res = unsafe { libc::setuid(uid) };

    Errno::result(res).map(drop)
}

#[inline]
pub fn setgid(gid: gid_t) -> Result<()> {
    let res = unsafe { libc::setgid(gid) };

    Errno::result(res).map(drop)
}

#[inline]
pub fn pause() -> Result<()> {
    let res = unsafe { libc::pause() };

    Errno::result(res).map(drop)
}

#[inline]
// Per POSIX, does not fail:
//   http://pubs.opengroup.org/onlinepubs/009695399/functions/sleep.html#tag_03_705_05
pub fn sleep(seconds: libc::c_uint) -> c_uint {
    unsafe { libc::sleep(seconds) }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux {
    use sys::syscall::{syscall, SYSPIVOTROOT};
    use {Errno, Result, NixPath};

    #[cfg(feature = "execvpe")]
    use std::ffi::CString;

    pub fn pivot_root<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(
            new_root: &P1, put_old: &P2) -> Result<()> {
        let res = try!(try!(new_root.with_nix_path(|new_root| {
            put_old.with_nix_path(|put_old| {
                unsafe {
                    syscall(SYSPIVOTROOT, new_root.as_ptr(), put_old.as_ptr())
                }
            })
        })));

        Errno::result(res).map(drop)
    }

    #[inline]
    #[cfg(feature = "execvpe")]
    pub fn execvpe(filename: &CString, args: &[CString], env: &[CString]) -> Result<()> {
        use std::ptr;
        use libc::c_char;

        let mut args_p: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();
        args_p.push(ptr::null());

        let mut env_p: Vec<*const c_char> = env.iter().map(|s| s.as_ptr()).collect();
        env_p.push(ptr::null());

        unsafe {
            super::ffi::execvpe(filename.as_ptr(), args_p.as_ptr(), env_p.as_ptr())
        };

        Err(Error::Sys(Errno::last()))
    }
}
