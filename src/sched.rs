#![cfg(target_os = "linux")]

use std::mem;
use libc::{c_int, c_uint, c_void};
use super::{SysResult, SysError};

pub type CloneFlags = c_uint;

pub static CLONE_VM:             CloneFlags = 0x00000100;
pub static CLONE_FS:             CloneFlags = 0x00000200;
pub static CLONE_FILES:          CloneFlags = 0x00000400;
pub static CLONE_SIGHAND:        CloneFlags = 0x00000800;
pub static CLONE_PTRACE:         CloneFlags = 0x00002000;
pub static CLONE_VFORK:          CloneFlags = 0x00004000;
pub static CLONE_PARENT:         CloneFlags = 0x00008000;
pub static CLONE_THREAD:         CloneFlags = 0x00010000;
pub static CLONE_NEWNS:          CloneFlags = 0x00020000;
pub static CLONE_SYSVSEM:        CloneFlags = 0x00040000;
pub static CLONE_SETTLS:         CloneFlags = 0x00080000;
pub static CLONE_PARENT_SETTID:  CloneFlags = 0x00100000;
pub static CLONE_CHILD_CLEARTID: CloneFlags = 0x00200000;
pub static CLONE_DETACHED:       CloneFlags = 0x00400000;
pub static CLONE_UNTRACED:       CloneFlags = 0x00800000;
pub static CLONE_CHILD_SETTID:   CloneFlags = 0x01000000;
pub static CLONE_NEWUTS:         CloneFlags = 0x04000000;
pub static CLONE_NEWIPC:         CloneFlags = 0x08000000;
pub static CLONE_NEWUSER:        CloneFlags = 0x10000000;
pub static CLONE_NEWPID:         CloneFlags = 0x20000000;
pub static CLONE_NEWNET:         CloneFlags = 0x40000000;
pub static CLONE_IO:             CloneFlags = 0x80000000;

pub type CloneCb<'a> = ||:'a -> int;

mod ffi {
    use libc::{c_void, c_int};

    type CloneCb = extern "C" fn (data: *const super::CloneCb) -> c_int;

    extern {
        // create a child process
        // doc: http://man7.org/linux/man-pages/man2/clone.2.html
        pub fn clone(
            cb: *const CloneCb,
            child_stack: *mut c_void,
            flags: super::CloneFlags,
            arg: *mut super::CloneCb,
            ...) -> c_int;

        // disassociate parts of the process execution context
        // doc: http://man7.org/linux/man-pages/man2/unshare.2.html
        pub fn unshare(flags: super::CloneFlags) -> c_int;
    }
}

pub fn clone(mut cb: CloneCb, stack: &mut [u8], flags: CloneFlags) -> SysResult<()> {
    extern "C" fn callback(data: *mut CloneCb) -> c_int {
        let cb: &mut CloneCb = unsafe { &mut *data };
        (*cb)() as c_int
    }

    let res = unsafe {
        let ptr = stack.as_mut_ptr().offset(stack.len() as int);
        ffi::clone(mem::transmute(callback), ptr as *mut c_void, flags, &mut cb)
    };

    if res != 0 {
        return Err(SysError::last());
    }

    Ok(())
}

pub fn unshare(flags: CloneFlags) -> SysResult<()> {
    let res = unsafe { ffi::unshare(flags) };

    if res != 0 {
        return Err(SysError::last());
    }

    Ok(())
}
