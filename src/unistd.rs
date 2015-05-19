//! Standard symbolic constants and types
//!
use {Error, Result, NixPath, AsExtStr, from_ffi};
use errno::Errno;
use fcntl::{fcntl, Fd, OFlag, O_NONBLOCK, O_CLOEXEC, FD_CLOEXEC};
use fcntl::FcntlArg::{F_SETFD, F_SETFL};
use libc::{c_char, c_void, c_int, size_t, pid_t, off_t};
use std::{mem, ptr};
use std::ffi::CString;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::linux::*;

mod ffi {
    use libc::{c_char, c_int, size_t};
    pub use libc::{close, read, write, pipe, ftruncate, unlink};
    pub use libc::funcs::posix88::unistd::{fork, getpid, getppid};

    extern {
        // duplicate a file descriptor
        // doc: http://man7.org/linux/man-pages/man2/dup.2.html
        pub fn dup(oldfd: c_int) -> c_int;
        pub fn dup2(oldfd: c_int, newfd: c_int) -> c_int;

        // change working directory
        // doc: http://man7.org/linux/man-pages/man2/chdir.2.html
        pub fn chdir(path: *const c_char) -> c_int;

        // execute program
        // doc: http://man7.org/linux/man-pages/man2/execve.2.html
        pub fn execve(filename: *const c_char, argv: *const *const c_char, envp: *const *const c_char) -> c_int;
        // doc: http://man7.org/linux/man-pages/man3/exec.3.html
        #[cfg(any(target_os = "linux", target_os = "android"))]
        pub fn execvpe(filename: *const c_char, argv: *const *const c_char, envp: *const *const c_char) -> c_int;

        // run the current process in the background
        // doc: http://man7.org/linux/man-pages/man3/daemon.3.html
        pub fn daemon(nochdir: c_int, noclose: c_int) -> c_int;

        // sets the hostname to the value given
        // doc: http://man7.org/linux/man-pages/man2/gethostname.2.html
        pub fn gethostname(name: *mut c_char, len: size_t) -> c_int;

        // gets the hostname
        // doc: http://man7.org/linux/man-pages/man2/gethostname.2.html
        pub fn sethostname(name: *const c_char, len: size_t) -> c_int;

        // change root directory
        // doc: http://man7.org/linux/man-pages/man2/gethostname.2.html
        pub fn chroot(path: *const c_char) -> c_int;
    }
}

#[derive(Clone, Copy)]
pub enum Fork {
    Parent(pid_t),
    Child
}

impl Fork {
    pub fn is_child(&self) -> bool {
        match *self {
            Fork::Child => true,
            _ => false
        }
    }

    pub fn is_parent(&self) -> bool {
        match *self {
            Fork::Parent(_) => true,
            _ => false
        }
    }
}

pub fn fork() -> Result<Fork> {
    use self::Fork::*;

    let res = unsafe { ffi::fork() };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    } else if res == 0 {
        Ok(Child)
    } else {
        Ok(Parent(res))
    }
}

#[inline]
pub fn getpid() -> pid_t {
    unsafe { ffi::getpid() } // no error handling, according to man page: "These functions are always successful."
}
#[inline]
pub fn getppid() -> pid_t {
    unsafe { ffi::getppid() } // no error handling, according to man page: "These functions are always successful."
}

#[inline]
pub fn dup(oldfd: Fd) -> Result<Fd> {
    let res = unsafe { ffi::dup(oldfd) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(res)
}

#[inline]
pub fn dup2(oldfd: Fd, newfd: Fd) -> Result<Fd> {
    let res = unsafe { ffi::dup2(oldfd, newfd) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(res)
}

pub fn dup3(oldfd: Fd, newfd: Fd, flags: OFlag) -> Result<Fd> {
    dup3_polyfill(oldfd, newfd, flags)
}

#[inline]
fn dup3_polyfill(oldfd: Fd, newfd: Fd, flags: OFlag) -> Result<Fd> {
    use errno::EINVAL;

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
    let res = try!(path.with_nix_path(|osstr| {
        unsafe { ffi::chdir(osstr.as_ext_str()) }
    }));

    if res != 0 {
        return Err(Error::Sys(Errno::last()));
    }

    return Ok(())
}

#[inline]
pub fn execve(filename: &CString, args: &[CString], env: &[CString]) -> Result<()> {
    let mut args_p: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();
    args_p.push(ptr::null());

    let mut env_p: Vec<*const c_char> = env.iter().map(|s| s.as_ptr()).collect();
    env_p.push(ptr::null());

    let res = unsafe {
        ffi::execve(filename.as_ptr(), args_p.as_ptr(), env_p.as_ptr())
    };

    if res != 0 {
        return Err(Error::Sys(Errno::last()));
    }

    unreachable!()
}

pub fn daemon(nochdir: bool, noclose: bool) -> Result<()> {
    let res = unsafe { ffi::daemon(nochdir as c_int, noclose as c_int) };
    from_ffi(res)
}

pub fn sethostname(name: &[u8]) -> Result<()> {
    let ptr = name.as_ptr() as *const c_char;
    let len = name.len() as size_t;

    let res = unsafe { ffi::sethostname(ptr, len) };
    from_ffi(res)
}

pub fn gethostname(name: &mut [u8]) -> Result<()> {
    let ptr = name.as_mut_ptr() as *mut c_char;
    let len = name.len() as size_t;

    let res = unsafe { ffi::gethostname(ptr, len) };
    from_ffi(res)
}

pub fn close(fd: Fd) -> Result<()> {
    let res = unsafe { ffi::close(fd) };
    from_ffi(res)
}

pub fn read(fd: Fd, buf: &mut [u8]) -> Result<usize> {
    let res = unsafe { ffi::read(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    return Ok(res as usize)
}

pub fn write(fd: Fd, buf: &[u8]) -> Result<usize> {
    let res = unsafe { ffi::write(fd, buf.as_ptr() as *const c_void, buf.len() as size_t) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    return Ok(res as usize)
}

pub fn pipe() -> Result<(Fd, Fd)> {
    unsafe {
        let mut res;
        let mut fds: [c_int; 2] = mem::uninitialized();

        res = ffi::pipe(fds.as_mut_ptr());

        if res < 0 {
            return Err(Error::Sys(Errno::last()));
        }

        Ok((fds[0], fds[1]))
    }
}

pub fn pipe2(flags: OFlag) -> Result<(Fd, Fd)> {
    unsafe {
        let mut res;
        let mut fds: [c_int; 2] = mem::uninitialized();

        res = ffi::pipe(fds.as_mut_ptr());

        if res < 0 {
            return Err(Error::Sys(Errno::last()));
        }

        try!(pipe2_setflags(fds[0], fds[1], flags));

        Ok((fds[0], fds[1]))
    }
}

fn pipe2_setflags(fd1: Fd, fd2: Fd, flags: OFlag) -> Result<()> {
    let mut res = Ok(());

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
            return Err(e);
        }
    }
}

pub fn ftruncate(fd: Fd, len: off_t) -> Result<()> {
    if unsafe { ffi::ftruncate(fd, len) } < 0 {
        Err(Error::Sys(Errno::last()))
    } else {
        Ok(())
    }
}

pub fn isatty(fd: Fd) -> Result<bool> {
    use libc;

    if unsafe { libc::isatty(fd) } == 1 {
        Ok(true)
    } else {
        match Errno::last() {
            // ENOTTY means `fd` is a valid file descriptor, but not a TTY, so
            // we return `Ok(false)`
            Errno::ENOTTY => Ok(false),
            err => Err(Error::Sys(err))
        }
    }
}

pub fn unlink<P: ?Sized + NixPath>(path: &P) -> Result<()> {
    let res = try!(path.with_nix_path(|osstr| {
    unsafe {
        ffi::unlink(osstr.as_ext_str())
    }
    }));
    from_ffi(res)
}

#[inline]
pub fn chroot<P: ?Sized + NixPath>(path: &P) -> Result<()> {
    let res = try!(path.with_nix_path(|osstr| {
        unsafe { ffi::chroot(osstr.as_ext_str()) }
    }));

    if res != 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux {
    use std::ptr;
    use sys::syscall::{syscall, SYSPIVOTROOT};
    use errno::Errno;
    use {Error, Result, NixPath};
    use std::ffi::CString;
    use libc::c_char;

    pub fn pivot_root<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(
            new_root: &P1, put_old: &P2) -> Result<()> {
        let res = try!(try!(new_root.with_nix_path(|new_root| {
            put_old.with_nix_path(|put_old| {
                unsafe {
                    syscall(SYSPIVOTROOT, new_root, put_old)
                }
            })
        })));

        if res != 0 {
            return Err(Error::Sys(Errno::last()));
        }

        Ok(())
    }

    #[inline]
    pub fn execvpe(filename: &CString, args: &[CString], env: &[CString]) -> Result<()> {
        let mut args_p: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();
        args_p.push(ptr::null());

        let mut env_p: Vec<*const c_char> = env.iter().map(|s| s.as_ptr()).collect();
        env_p.push(ptr::null());

        let res = unsafe {
            super::ffi::execvpe(filename.as_ptr(), args_p.as_ptr(), env_p.as_ptr())
        };

        if res != 0 {
            return Err(Error::Sys(Errno::last()));
        }

        unreachable!()
    }
}
