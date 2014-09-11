use std::ptr;
use std::c_str::{CString, ToCStr};
use libc::{c_char, c_void, c_int, size_t};
use fcntl::{Fd, OFlag};
use errno::{SysResult, SysError, from_ffi};

#[cfg(target_os = "linux")]
pub use self::linux::*;

mod ffi {
    use libc::{c_char, c_int, size_t};
    pub use libc::{close, read, write};

    extern {
        // duplicate a file descriptor
        // doc: http://man7.org/linux/man-pages/man2/dup.2.html
        pub fn dup(oldfd: c_int) -> c_int;
        pub fn dup2(oldfd: c_int, newfd: c_int) -> c_int;
        pub fn dup3(oldfd: c_int, newfd: c_int, flags: c_int) -> c_int;

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

#[inline]
pub fn dup3(oldfd: Fd, newfd: Fd, flags: OFlag) -> SysResult<Fd> {
    let res = unsafe { ffi::dup3(oldfd, newfd, flags.bits()) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
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
pub fn execve(filename: CString, args: &[CString], env: &[CString]) -> SysResult<()> {
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

    // Should never reach here
    Ok(())
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

#[cfg(target_os = "linux")]
mod linux {
    use std::path::Path;
    use syscall::{syscall, SysPivotRoot};
    use errno::{SysResult, SysError};

    pub fn pivot_root(new_root: &Path, put_old: &Path) -> SysResult<()> {
        let new_root = new_root.to_c_str();
        let put_old = put_old.to_c_str();

        let res = unsafe {
            syscall(SysPivotRoot, new_root.as_ptr(), put_old.as_ptr())
        };

        if res != 0 {
            return Err(SysError::last());
        }

        Ok(())
    }
}
