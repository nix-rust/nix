use std::{mem, ptr};
use std::c_str::{CString, ToCStr};
use libc::{c_char, c_void, c_int, size_t, pid_t};
use fcntl::{fcntl, Fd, OFlag, O_NONBLOCK, O_CLOEXEC, FD_CLOEXEC, F_SETFD, F_SETFL};
use errno::{SysResult, SysError, from_ffi};

#[cfg(target_os = "linux")]
pub use self::linux::*;

mod ffi {
    use libc::{c_char, c_int, size_t};
    pub use libc::{close, read, write, pipe};
    pub use libc::funcs::posix88::unistd::fork;

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

        // run the current process in the background
        // doc: http://man7.org/linux/man-pages/man3/daemon.3.html
        pub fn daemon(nochdir: c_int, noclose: c_int) -> c_int;

        // sets the hostname to the value given
        // doc: http://man7.org/linux/man-pages/man2/gethostname.2.html
        pub fn gethostname(name: *mut c_char, len: size_t) -> c_int;

        // gets the hostname
        // doc: http://man7.org/linux/man-pages/man2/gethostname.2.html
        pub fn sethostname(name: *const c_char, len: size_t) -> c_int;
    }
}

pub enum Fork {
    Parent(pid_t),
    Child
}

impl Fork {
    pub fn is_child(&self) -> bool {
        match *self {
            Child => true,
            _ => false
        }
    }

    pub fn is_parent(&self) -> bool {
        match *self {
            Parent(_) => true,
            _ => false
        }
    }
}

pub fn fork() -> SysResult<Fork> {
    let res = unsafe { ffi::fork() };

    if res < 0 {
        return Err(SysError::last());
    } else if res == 0 {
        Ok(Child)
    } else {
        Ok(Parent(res))
    }
}

#[inline]
pub fn dup(oldfd: Fd) -> SysResult<Fd> {
    let res = unsafe { ffi::dup(oldfd) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
}

#[inline]
pub fn dup2(oldfd: Fd, newfd: Fd) -> SysResult<Fd> {
    let res = unsafe { ffi::dup2(oldfd, newfd) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
pub fn dup3(oldfd: Fd, newfd: Fd, flags: OFlag) -> SysResult<Fd> {
    type F = unsafe extern "C" fn(c_int, c_int, c_int) -> c_int;

    extern {
        #[linkage = "extern_weak"]
        static dup3: *const ();
    }

    if !dup3.is_null() {
        let res = unsafe {
            mem::transmute::<*const (), F>(dup3)(
                oldfd, newfd, flags.bits())
        };

        if res < 0 {
            return Err(SysError::last());
        }

        Ok(res)
    } else {
        dup3_polyfill(oldfd, newfd, flags)
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn dup3(oldfd: Fd, newfd: Fd, flags: OFlag) -> SysResult<Fd> {
    dup3_polyfill(oldfd, newfd, flags)
}

#[inline]
fn dup3_polyfill(oldfd: Fd, newfd: Fd, flags: OFlag) -> SysResult<Fd> {
    use errno::EINVAL;

    if oldfd == newfd {
        return Err(SysError { kind: EINVAL });
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
pub fn chdir<S: ToCStr>(path: S) -> SysResult<()> {
    let path = path.to_c_str();
    let res = unsafe { ffi::chdir(path.as_ptr()) };

    if res != 0 {
        return Err(SysError::last());
    }

    return Ok(())
}

#[inline]
pub fn execve(filename: &CString, args: &[CString], env: &[CString]) -> SysResult<()> {
    let mut args_p: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();
    args_p.push(ptr::null());

    let mut env_p: Vec<*const c_char> = env.iter().map(|s| s.as_ptr()).collect();
    env_p.push(ptr::null());

    let res = unsafe {
        ffi::execve(filename.as_ptr(), args_p.as_ptr(), env_p.as_ptr())
    };

    if res != 0 {
        return Err(SysError::last());
    }

    unreachable!()
}

pub fn daemon(nochdir: bool, noclose: bool) -> SysResult<()> {
    let res = unsafe { ffi::daemon(nochdir as c_int, noclose as c_int) };
    from_ffi(res)
}

pub fn sethostname(name: &[u8]) -> SysResult<()> {
    let ptr = name.as_ptr() as *const c_char;
    let len = name.len() as u64;

    let res = unsafe { ffi::sethostname(ptr, len) };
    from_ffi(res)
}

pub fn gethostname(name: &mut [u8]) -> SysResult<()> {
    let ptr = name.as_mut_ptr() as *mut c_char;
    let len = name.len() as u64;

    let res = unsafe { ffi::gethostname(ptr, len) };
    from_ffi(res)
}

pub fn close(fd: Fd) -> SysResult<()> {
    let res = unsafe { ffi::close(fd) };
    from_ffi(res)
}

pub fn read(fd: Fd, buf: &mut [u8]) -> SysResult<uint> {
    let res = unsafe { ffi::read(fd, buf.as_mut_ptr() as *mut c_void, buf.len() as size_t) };

    if res < 0 {
        return Err(SysError::last());
    }

    return Ok(res as uint)
}

pub fn write(fd: Fd, buf: &[u8]) -> SysResult<uint> {
    let res = unsafe { ffi::write(fd, buf.as_ptr() as *const c_void, buf.len() as size_t) };

    if res < 0 {
        return Err(SysError::last());
    }

    return Ok(res as uint)
}

pub fn pipe() -> SysResult<(Fd, Fd)> {
    unsafe {
        let mut res;
        let mut fds: [c_int, ..2] = mem::uninitialized();

        res = ffi::pipe(fds.as_mut_ptr());

        if res < 0 {
            return Err(SysError::last());
        }

        Ok((fds[0], fds[1]))
    }
}

#[cfg(target_os = "linux")]
pub fn pipe2(flags: OFlag) -> SysResult<(Fd, Fd)> {
    type F = unsafe extern "C" fn(fds: *mut c_int, flags: c_int) -> c_int;

    extern {
        #[linkage = "extern_weak"]
        static pipe2: *const ();
    }

    let feat_atomic = !pipe2.is_null();

    unsafe {
        let mut res;
        let mut fds: [c_int, ..2] = mem::uninitialized();

        if feat_atomic {
            res = mem::transmute::<*const (), F>(pipe2)(
                fds.as_mut_ptr(), flags.bits());
        } else {
            res = ffi::pipe(fds.as_mut_ptr());
        }

        if res < 0 {
            return Err(SysError::last());
        }

        if !feat_atomic {
            try!(pipe2_setflags(fds[0], fds[1], flags));
        }

        Ok((fds[0], fds[1]))
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub fn pipe2(flags: OFlag) -> SysResult<(Fd, Fd)> {
    unsafe {
        let mut res;
        let mut fds: [c_int, ..2] = mem::uninitialized();

        res = ffi::pipe(fds.as_mut_ptr());

        if res < 0 {
            return Err(SysError::last());
        }

        try!(pipe2_setflags(fds[0], fds[1], flags));

        Ok((fds[0], fds[1]))
    }
}

fn pipe2_setflags(fd1: Fd, fd2: Fd, flags: OFlag) -> SysResult<()> {
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

#[cfg(target_os = "linux")]
mod linux {
    use std::path::Path;
    use syscall::{syscall, SYSPIVOTROOT};
    use errno::{SysResult, SysError};

    pub fn pivot_root(new_root: &Path, put_old: &Path) -> SysResult<()> {
        let new_root = new_root.to_c_str();
        let put_old = put_old.to_c_str();

        let res = unsafe {
            syscall(SYSPIVOTROOT, new_root.as_ptr(), put_old.as_ptr())
        };

        if res != 0 {
            return Err(SysError::last());
        }

        Ok(())
    }
}
