use std::ptr;
use std::c_str::{CString, ToCStr};
use libc::{c_char};
use syscall::{syscall, SysPivotRoot};
use {SysResult, SysError};

mod ffi {
    use libc::{c_char, c_int};

    extern {
        // change working directory
        // doc: http://man7.org/linux/man-pages/man2/chdir.2.html
        pub fn chdir(path: *const c_char) -> c_int;

        // execute program
        // doc: http://man7.org/linux/man-pages/man2/execve.2.html
        pub fn execve(filename: *const c_char, argv: *const *const c_char, envp: *const *const c_char) -> c_int;
    }
}

pub fn chdir<S: ToCStr>(path: S) -> SysResult<()> {
    let path = path.to_c_str();
    let res = unsafe { ffi::chdir(path.as_ptr()) };

    if res != 0 {
        return Err(SysError::last());
    }

    return Ok(())
}

pub fn execve<S: ToCStr, S1: ToCStr, I1: Iterator<S1>, S2: ToCStr, I2: Iterator<S2>>(
        filename: S, args: I1, env: I2) -> SysResult<()> {

    let args: Vec<CString> = args.map(|s| s.to_c_str()).collect();
    let env: Vec<CString> = env.map(|s| s.to_c_str()).collect();

    let mut args_p: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();
    args_p.push(ptr::null());

    let mut env_p: Vec<*const c_char> = env.iter().map(|s| s.as_ptr()).collect();
    env_p.push(ptr::null());

    let res = unsafe {
        ffi::execve(filename.to_c_str().as_ptr(), args_p.as_ptr(), env_p.as_ptr())
    };

    if res != 0 {
        return Err(SysError::last());
    }

    // Should never reach here
    Ok(())
}

pub fn pivot_root<S1: ToCStr, S2: ToCStr>(new_root: S1, put_old: S2) -> SysResult<()> {
    let new_root = new_root.to_c_str();
    let put_old = put_old.to_c_str();

    let res = unsafe {
        syscall(SysPivotRoot, new_root.as_ptr(), put_old.as_ptr())
    };

    if res == 0 {
        return Err(SysError::last());
    }

    Ok(())
}
