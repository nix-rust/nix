//! Safe wrappers around functions found in libc "unistd.h" header

use {Errno, Error, Result, NixPath};
use fcntl::{AtFlags, fcntl, OFlag, O_CLOEXEC, FD_CLOEXEC};
use fcntl::FcntlArg::F_SETFD;
use libc::{self, c_char, c_void, c_int, c_uint, size_t, pid_t, off_t, uid_t, gid_t, mode_t};
use std::mem;
use std::ffi::{CString, CStr, OsString, OsStr};
use std::os::unix::ffi::{OsStringExt, OsStrExt};
use std::os::unix::io::RawFd;
use std::path::PathBuf;
use void::Void;
use sys::stat::Mode;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::linux::*;

/// Represents the successful result of calling `fork`
///
/// When `fork` is called, the process continues execution in the parent process
/// and in the new child.  This return type can be examined to determine whether
/// you are now executing in the parent process or in the child.
#[derive(Clone, Copy)]
pub enum ForkResult {
    Parent { child: pid_t },
    Child,
}

impl ForkResult {

    /// Return `true` if this is the child process of the `fork()`
    #[inline]
    pub fn is_child(&self) -> bool {
        match *self {
            ForkResult::Child => true,
            _ => false
        }
    }

    /// Returns `true` if this is the parent process of the `fork()`
    #[inline]
    pub fn is_parent(&self) -> bool {
        !self.is_child()
    }
}

/// Create a new child process duplicating the parent process ([see
/// fork(2)](http://man7.org/linux/man-pages/man2/fork.2.html)).
///
/// After calling the fork system call (successfully) two processes will
/// be created that are identical with the exception of their pid and the
/// return value of this function.  As an example:
///
/// ```no_run
/// use nix::unistd::{fork, ForkResult};
///
/// match fork() {
///    Ok(ForkResult::Parent { child, .. }) => {
///        println!("Continuing execution in parent process, new child has pid: {}", child);
///    }
///    Ok(ForkResult::Child) => println!("I'm a new child process"),
///    Err(_) => println!("Fork failed"),
/// }
/// ```
///
/// This will print something like the following (order indeterministic).  The
/// thing to note is that you end up with two processes continuing execution
/// immediately after the fork call but with different match arms.
///
/// ```text
/// Continuing execution in parent process, new child has pid: 1234
/// I'm a new child process
/// ```
#[inline]
pub fn fork() -> Result<ForkResult> {
    use self::ForkResult::*;
    let res = unsafe { libc::fork() };

    Errno::result(res).map(|res| match res {
        0 => Child,
        res => Parent { child: res },
    })
}

/// Get the pid of this process (see
/// [getpid(2)](http://man7.org/linux/man-pages/man2/getpid.2.html)).
///
/// Since you are running code, there is always a pid to return, so there
/// is no error case that needs to be handled.
#[inline]
pub fn getpid() -> pid_t {
    unsafe { libc::getpid() }
}

/// Get the pid of this processes' parent (see
/// [getpid(2)](http://man7.org/linux/man-pages/man2/getpid.2.html)).
///
/// There is always a parent pid to return, so there is no error case that needs
/// to be handled.
#[inline]
pub fn getppid() -> pid_t {
    unsafe { libc::getppid() } // no error handling, according to man page: "These functions are always successful."
}

/// Set a process group ID (see
/// [setpgid(2)](http://man7.org/linux/man-pages/man2/setpgid.2.html)).
///
/// Set the process group id (PGID) of a particular process.  If a pid of zero
/// is specified, then the pid of the calling process is used.  Process groups
/// may be used to group together a set of processes in order for the OS to
/// apply some operations across the group.
///
/// `setsid()` may be used to create a new process group.
#[inline]
pub fn setpgid(pid: pid_t, pgid: pid_t) -> Result<()> {
    let res = unsafe { libc::setpgid(pid, pgid) };
    Errno::result(res).map(drop)
}
#[inline]
pub fn getpgid(pid: Option<pid_t>) -> Result<pid_t> {
    let res = unsafe { libc::getpgid(pid.unwrap_or(0 as pid_t)) };
    Errno::result(res)
}

/// Create new session and set process group id (see
/// [setsid(2)](http://man7.org/linux/man-pages/man2/setsid.2.html)).
#[inline]
pub fn setsid() -> Result<pid_t> {
    Errno::result(unsafe { libc::setsid() })
}


/// Get the terminal foreground process group (see
/// [tcgetpgrp(3)](http://man7.org/linux/man-pages/man3/tcgetpgrp.3.html)).
///
/// Get the group process id (GPID) of the foreground process group on the
/// terminal associated to file descriptor (FD).
#[inline]
pub fn tcgetpgrp(fd: c_int) -> Result<pid_t> {
    let res = unsafe { libc::tcgetpgrp(fd) };
    Errno::result(res)
}
/// Set the terminal foreground process group (see
/// [tcgetpgrp(3)](http://man7.org/linux/man-pages/man3/tcgetpgrp.3.html)).
///
/// Get the group process id (PGID) to the foreground process group on the
/// terminal associated to file descriptor (FD).
#[inline]
pub fn tcsetpgrp(fd: c_int, pgrp: pid_t) -> Result<()> {
    let res = unsafe { libc::tcsetpgrp(fd, pgrp) };
    Errno::result(res).map(drop)
}


/// Get the group id of the calling process (see
///[getpgrp(3)](http://man7.org/linux/man-pages/man3/getpgrp.3p.html)).
///
/// Get the process group id (PGID) of the calling process.
/// According to the man page it is always successful.
#[inline]
pub fn getpgrp() -> pid_t {
    unsafe { libc::getpgrp() }
}

/// Get the caller's thread ID (see
/// [gettid(2)](http://man7.org/linux/man-pages/man2/gettid.2.html).
///
/// This function is only available on Linux based systems.  In a single
/// threaded process, the main thread will have the same ID as the process.  In
/// a multithreaded process, each thread will have a unique thread id but the
/// same process ID.
///
/// No error handling is required as a thread id should always exist for any
/// process, even if threads are not being used.
#[cfg(any(target_os = "linux", target_os = "android"))]
#[inline]
pub fn gettid() -> pid_t {
    unsafe { libc::syscall(libc::SYS_gettid) as pid_t }
}

/// Create a copy of the specified file descriptor (see
/// [dup(2)](http://man7.org/linux/man-pages/man2/dup.2.html)).
///
/// The new file descriptor will be have a new index but refer to the same
/// resource as the old file descriptor and the old and new file descriptors may
/// be used interchangeably.  The new and old file descriptor share the same
/// underlying resource, offset, and file status flags.  The actual index used
/// for the file descriptor will be the lowest fd index that is available.
///
/// The two file descriptors do not share file descriptor flags (e.g. `FD_CLOEXEC`).
#[inline]
pub fn dup(oldfd: RawFd) -> Result<RawFd> {
    let res = unsafe { libc::dup(oldfd) };

    Errno::result(res)
}

/// Create a copy of the specified file descriptor using the specified fd (see
/// [dup(2)](http://man7.org/linux/man-pages/man2/dup.2.html)).
///
/// This function behaves similar to `dup()` except that it will try to use the
/// specified fd instead of allocating a new one.  See the man pages for more
/// detail on the exact behavior of this function.
#[inline]
pub fn dup2(oldfd: RawFd, newfd: RawFd) -> Result<RawFd> {
    let res = unsafe { libc::dup2(oldfd, newfd) };

    Errno::result(res)
}

/// Create a new copy of the specified file descriptor using the specified fd
/// and flags (see [dup(2)](http://man7.org/linux/man-pages/man2/dup.2.html)).
///
/// This function behaves similar to `dup2()` but allows for flags to be
/// specified.
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

/// Change the current working directory of the calling process (see
/// [chdir(2)](http://man7.org/linux/man-pages/man2/chdir.2.html)).
///
/// This function may fail in a number of different scenarios.  See the man
/// pages for additional details on possible failure cases.
#[inline]
pub fn chdir<P: ?Sized + NixPath>(path: &P) -> Result<()> {
    let res = try!(path.with_nix_path(|cstr| {
        unsafe { libc::chdir(cstr.as_ptr()) }
    }));

    Errno::result(res).map(drop)
}

/// Change the current working directory of the process to the one
/// given as an open file descriptor (see
/// [fchdir(2)](http://man7.org/linux/man-pages/man2/fchdir.2.html)).
///
/// This function may fail in a number of different scenarios.  See the man
/// pages for additional details on possible failure cases.
#[inline]
pub fn fchdir(dirfd: RawFd) -> Result<()> {
    let res = unsafe { libc::fchdir(dirfd) };

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
///     let tmp_dir1 = TempDir::new("test_mkdir").unwrap();
///     let tmp_dir2 = tmp_dir1.path().join("new_dir");
///
///     // create new directory and give read, write and execute rights to the owner
///     match unistd::mkdir(&tmp_dir2, stat::S_IRWXU) {
///        Ok(_) => println!("created {:?}", tmp_dir2),
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

/// Change the ownership of the file at `path` to be owned by the specified
/// `owner` (user) and `group` (see
/// [chown(2)](http://man7.org/linux/man-pages/man2/lchown.2.html)).
///
/// The owner/group for the provided path name will not be modified if `None` is
/// provided for that argument.  Ownership change will be attempted for the path
/// only if `Some` owner/group is provided.
///
/// This call may fail under a number of different situations.  See [the man
/// pages](http://man7.org/linux/man-pages/man2/lchown.2.html#ERRORS) for
/// additional details.
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

/// Replace the current process image with a new one (see
/// [exec(3)](http://man7.org/linux/man-pages/man3/exec.3.html)).
///
/// See the `::nix::unistd::execve` system call for additional details.  `execv`
/// performs the same action but does not allow for customization of the
/// environment for the new process.
#[inline]
pub fn execv(path: &CString, argv: &[CString]) -> Result<Void> {
    let args_p = to_exec_array(argv);

    unsafe {
        libc::execv(path.as_ptr(), args_p.as_ptr())
    };

    Err(Error::Sys(Errno::last()))
}


/// Replace the current process image with a new one (see
/// [execve(2)](http://man7.org/linux/man-pages/man2/execve.2.html)).
///
/// The execve system call allows for another process to be "called" which will
/// replace the current process image.  That is, this process becomes the new
/// command that is run. On success, this function will not return. Instead,
/// the new program will run until it exits.
///
/// If an error occurs, this function will return with an indication of the
/// cause of failure.  See
/// [execve(2)#errors](http://man7.org/linux/man-pages/man2/execve.2.html#ERRORS)
/// for a list of potential problems that maight cause execv to fail.
///
/// `::nix::unistd::execv` and `::nix::unistd::execve` take as arguments a slice
/// of `::std::ffi::CString`s for `args` and `env` (for `execve`). Each element
/// in the `args` list is an argument to the new process. Each element in the
/// `env` list should be a string in the form "key=value".
#[inline]
pub fn execve(path: &CString, args: &[CString], env: &[CString]) -> Result<Void> {
    let args_p = to_exec_array(args);
    let env_p = to_exec_array(env);

    unsafe {
        libc::execve(path.as_ptr(), args_p.as_ptr(), env_p.as_ptr())
    };

    Err(Error::Sys(Errno::last()))
}

/// Replace the current process image with a new one and replicate shell `PATH`
/// searching behavior (see
/// [exec(3)](http://man7.org/linux/man-pages/man3/exec.3.html)).
///
/// See `::nix::unistd::execve` for additoinal details.  `execvp` behaves the
/// same as execv except that it will examine the `PATH` environment variables
/// for file names not specified with a leading slash.  For example, `execv`
/// would not work if "bash" was specified for the path argument, but `execvp`
/// would assuming that a bash executable was on the system `PATH`.
#[inline]
pub fn execvp(filename: &CString, args: &[CString]) -> Result<Void> {
    let args_p = to_exec_array(args);

    unsafe {
        libc::execvp(filename.as_ptr(), args_p.as_ptr())
    };

    Err(Error::Sys(Errno::last()))
}

/// Daemonize this process by detaching from the controlling terminal (see
/// [daemon(3)](http://man7.org/linux/man-pages/man3/daemon.3.html)).
///
/// When a process is launched it is typically associated with a parent and it,
/// in turn, by its controlling terminal/process.  In order for a process to run
/// in the "background" it must daemonize itself by detaching itself.  Under
/// posix, this is done by doing the following:
///
/// 1. Parent process (this one) forks
/// 2. Parent process exits
/// 3. Child process continues to run.
///
/// `nochdir`:
///
/// * `nochdir = true`: The current working directory after daemonizing will
///    be the current working directory.
/// *  `nochdir = false`: The current working directory after daemonizing will
///    be the root direcory, `/`.
///
/// `noclose`:
///
/// * `noclose = true`: The process' current stdin, stdout, and stderr file
///   descriptors will remain identical after daemonizing.
/// * `noclose = false`: The process' stdin, stdout, and stderr will point to
///   `/dev/null` after daemonizing.
///
/// The underlying implementation (in libc) calls both
/// [fork(2)](http://man7.org/linux/man-pages/man2/fork.2.html) and
/// [setsid(2)](http://man7.org/linux/man-pages/man2/setsid.2.html) and, as
/// such, error that could be returned by either of those functions could also
/// show up as errors here.
pub fn daemon(nochdir: bool, noclose: bool) -> Result<()> {
    let res = unsafe { libc::daemon(nochdir as c_int, noclose as c_int) };
    Errno::result(res).map(drop)
}

/// Set the system host name (see
/// [gethostname(2)](http://man7.org/linux/man-pages/man2/gethostname.2.html)).
///
/// Given a name, attempt to update the system host name to the given string.
/// On some systems, the host name is limited to as few as 64 bytes.  An error
/// will be return if the name is not valid or the current process does not have
/// permissions to update the host name.
pub fn sethostname<S: AsRef<OsStr>>(name: S) -> Result<()> {
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
    let ptr = name.as_ref().as_bytes().as_ptr() as *const c_char;
    let len = name.as_ref().len() as sethostname_len_t;

    let res = unsafe { libc::sethostname(ptr, len) };
    Errno::result(res).map(drop)
}

/// Get the host name and store it in the provided buffer, returning a pointer
/// the CStr in that buffer on success (see
/// [gethostname(2)](http://man7.org/linux/man-pages/man2/gethostname.2.html)).
///
/// This function call attempts to get the host name for the running system and
/// store it in a provided buffer.  The buffer will be populated with bytes up
/// to the length of the provided slice including a NUL terminating byte.  If
/// the hostname is longer than the length provided, no error will be provided.
/// The posix specification does not specify whether implementations will
/// null-terminate in this case, but the nix implementation will ensure that the
/// buffer is null terminated in this case.
///
/// ```no_run
/// use nix::unistd;
///
/// let mut buf = [0u8; 64];
/// let hostname_cstr = unistd::gethostname(&mut buf).expect("Failed getting hostname");
/// let hostname = hostname_cstr.to_str().expect("Hostname wasn't valid UTF-8");
/// println!("Hostname: {}", hostname);
/// ```
pub fn gethostname<'a>(buffer: &'a mut [u8]) -> Result<&'a CStr> {
    let ptr = buffer.as_mut_ptr() as *mut c_char;
    let len = buffer.len() as size_t;

    let res = unsafe { libc::gethostname(ptr, len) };
    Errno::result(res).map(|_| {
        buffer[len - 1] = 0; // ensure always null-terminated
        unsafe { CStr::from_ptr(buffer.as_ptr() as *const c_char) }
    })
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

// libc only defines `pipe2` in `libc::notbsd`.
#[cfg(any(target_os = "linux",
          target_os = "android",
          target_os = "emscripten"))]
pub fn pipe2(flags: OFlag) -> Result<(RawFd, RawFd)> {
    let mut fds: [c_int; 2] = unsafe { mem::uninitialized() };

    let res = unsafe { libc::pipe2(fds.as_mut_ptr(), flags.bits()) };

    try!(Errno::result(res));

    Ok((fds[0], fds[1]))
}

#[cfg(not(any(target_os = "linux",
              target_os = "android",
              target_os = "emscripten")))]
pub fn pipe2(flags: OFlag) -> Result<(RawFd, RawFd)> {
    let mut fds: [c_int; 2] = unsafe { mem::uninitialized() };

    let res = unsafe { libc::pipe(fds.as_mut_ptr()) };

    try!(Errno::result(res));

    try!(pipe2_setflags(fds[0], fds[1], flags));

    Ok((fds[0], fds[1]))
}

#[cfg(not(any(target_os = "linux",
              target_os = "android",
              target_os = "emscripten")))]
fn pipe2_setflags(fd1: RawFd, fd2: RawFd, flags: OFlag) -> Result<()> {
    use fcntl::O_NONBLOCK;
    use fcntl::FcntlArg::F_SETFL;

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

/// Creates a regular file which persists even after process termination
///
/// * `template`: a path whose 6 rightmost characters must be X, e.g. /tmp/tmpfile_XXXXXX
/// * returns: tuple of file descriptor and filename
///
/// Err is returned either if no temporary filename could be created or the template doesn't
/// end with XXXXXX
///
/// # Example
///
/// ```rust
/// use nix::unistd;
///
/// let _ = match unistd::mkstemp("/tmp/tempfile_XXXXXX") {
///     Ok((fd, path)) => {
///         unistd::unlink(path.as_path()).unwrap(); // flag file to be deleted at app termination
///         fd
///     }
///     Err(e) => panic!("mkstemp failed: {}", e)
/// };
/// // do something with fd
/// ```
#[inline]
pub fn mkstemp<P: ?Sized + NixPath>(template: &P) -> Result<(RawFd, PathBuf)> {
    let mut path = try!(template.with_nix_path(|path| {path.to_bytes_with_nul().to_owned()}));
    let p = path.as_mut_ptr() as *mut _;
    let fd = unsafe { libc::mkstemp(p) };
    let last = path.pop(); // drop the trailing nul
    debug_assert!(last == Some(b'\0'));
    let pathname = OsString::from_vec(path);
    try!(Errno::result(fd));
    Ok((fd, PathBuf::from(pathname)))
}

libc_bitflags!{
    /// Flags that determine what permissions to check for in [`access`](fn.access.html)
    /// and [`faccessat`](fn.faccessat.html) functions.
    pub flags AccessFlags: c_int {
        /// Read permission.
        R_OK,
        /// Write permission.
        W_OK,
        /// Execute permission.
        X_OK,
    }
}

/// Whether [`access`](fn.access.html) and [`faccessat`](fn.faccessat.html) functions
/// should check whether a file exists or whether the current process has the requested
/// permissions to access it.
pub enum AccessMode {
    /// Check whether the file exists.
    F_OK,
    /// Check whether the file exists and the current process has the requested
    /// permissions to access it.
    Flags(AccessFlags),
}

impl AccessMode {
    /// Convert the `enum AccessMode` + `flags AccessFlags` combo to the plain
    /// `mode: c_int` syscalls expect.
    fn to_libc_type(&self) -> c_int {
        match self {
            &AccessMode::F_OK => libc::F_OK,
            &AccessMode::Flags(flags) => flags.bits(),
        }
    }
}

/// Check whether a file exists or whether the current process can access a file ([see access(2)]
/// (http://man7.org/linux/man-pages/man2/access.2.html)).
///
/// # Example
/// ```
/// use nix::unistd::*;
///
/// let devnull_exists = access("/dev/null", AccessMode::F_OK);
/// assert!(devnull_exists.is_ok());
/// ```
pub fn access<P: ?Sized + NixPath>(path: &P, mode: AccessMode) -> Result<()> {
    let res = try!(path.with_nix_path(|path|
        unsafe {
            libc::access(path.as_ptr(), mode.to_libc_type())
        }
    ));
    Errno::result(res).map(drop)
}

/// Check whether a file exists or whether the current process can access a file,
/// relative to a directory file descriptor ([see faccessat(2)]
/// (http://man7.org/linux/man-pages/man2/faccessat.2.html)).
pub fn faccessat<P: ?Sized + NixPath>(dirfd: RawFd,
                                      path: &P,
                                      mode: AccessMode,
                                      flags: AtFlags)
                                      -> Result<()> {
    let res = try!(path.with_nix_path(|path|
        unsafe {
            libc::faccessat(dirfd,
                            path.as_ptr(),
                            mode.to_libc_type(),
                            flags.bits())
        }
    ));
    Errno::result(res).map(drop)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux {
    use libc::{self, uid_t, gid_t};
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

    /// Sets the real, effective, and saved uid.
    /// ([see setresuid(2)](http://man7.org/linux/man-pages/man2/setresuid.2.html))
    ///
    /// * `ruid`: real user id
    /// * `euid`: effective user id
    /// * `suid`: saved user id
    /// * returns: Ok or libc error code.
    ///
    /// Err is returned if the user doesn't have permission to set this UID.
    #[inline]
    pub fn setresuid(ruid: uid_t, euid: uid_t, suid: uid_t) -> Result<()> {
        let res = unsafe { libc::setresuid(ruid, euid, suid) };

        Errno::result(res).map(drop)
    }

    /// Sets the real, effective, and saved gid.
    /// ([see setresuid(2)](http://man7.org/linux/man-pages/man2/setresuid.2.html))
    ///
    /// * `rgid`: real user id
    /// * `egid`: effective user id
    /// * `sgid`: saved user id
    /// * returns: Ok or libc error code.
    ///
    /// Err is returned if the user doesn't have permission to set this GID.
    #[inline]
    pub fn setresgid(rgid: gid_t, egid: gid_t, sgid: gid_t) -> Result<()> {
        let res = unsafe { libc::setresgid(rgid, egid, sgid) };

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
