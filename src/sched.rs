use std::mem;
use std::os::unix::io::RawFd;
use std::option::Option;
use libc::{self, c_int, c_void, pid_t};
use {Errno, Error, Result};

// For some functions taking with a parameter of type CloneFlags,
// only a subset of these flags have an effect.
libc_bitflags!{
    flags CloneFlags: libc::c_int {
        CLONE_VM,
        CLONE_FS,
        CLONE_FILES,
        CLONE_SIGHAND,
        CLONE_PTRACE,
        CLONE_VFORK,
        CLONE_PARENT,
        CLONE_THREAD,
        CLONE_NEWNS,
        CLONE_SYSVSEM,
        CLONE_SETTLS,
        CLONE_PARENT_SETTID,
        CLONE_CHILD_CLEARTID,
        CLONE_DETACHED,
        CLONE_UNTRACED,
        CLONE_CHILD_SETTID,
        CLONE_NEWUTS,
        CLONE_NEWIPC,
        CLONE_NEWUSER,
        CLONE_NEWPID,
        CLONE_NEWNET,
        CLONE_IO,
    }
}

pub type CloneCb<'a> = Box<FnMut() -> isize + 'a>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CpuSet {
    cpu_set: libc::cpu_set_t,
}

impl CpuSet {
    pub fn new() -> CpuSet {
        CpuSet { cpu_set: unsafe { mem::zeroed() } }
    }

    pub fn is_set(&self, field: usize) -> Result<bool> {
        if field >= 8 * mem::size_of::<libc::cpu_set_t>() {
            Err(Error::Sys(Errno::EINVAL))
        } else {
            Ok(unsafe { libc::CPU_ISSET(field, &self.cpu_set) })
        }
    }

    pub fn set(&mut self, field: usize) -> Result<()> {
        if field >= 8 * mem::size_of::<libc::cpu_set_t>() {
            Err(Error::Sys(Errno::EINVAL))
        } else {
            Ok(unsafe { libc::CPU_SET(field, &mut self.cpu_set) })
        }
    }

    pub fn unset(&mut self, field: usize) -> Result<()> {
        if field >= 8 * mem::size_of::<libc::cpu_set_t>() {
            Err(Error::Sys(Errno::EINVAL))
        } else {
            Ok(unsafe { libc::CPU_CLR(field, &mut self.cpu_set) })
        }
    }
}

mod ffi {
    use libc::{c_void, c_int};

    pub type CloneCb = extern "C" fn(data: *const super::CloneCb) -> c_int;

    // We cannot give a proper #[repr(C)] to super::CloneCb
    #[allow(improper_ctypes)]
    extern "C" {
        // create a child process
        // doc: http://man7.org/linux/man-pages/man2/clone.2.html
        pub fn clone(cb: *const CloneCb,
                     child_stack: *mut c_void,
                     flags: c_int,
                     arg: *mut super::CloneCb,
                     ...)
                     -> c_int;
    }
}

pub fn sched_setaffinity(pid: isize, cpuset: &CpuSet) -> Result<()> {
    let res = unsafe {
        libc::sched_setaffinity(pid as libc::pid_t,
                                mem::size_of::<CpuSet>() as libc::size_t,
                                mem::transmute(cpuset))
    };

    Errno::result(res).map(drop)
}

pub fn clone(mut cb: CloneCb,
             stack: &mut [u8],
             flags: CloneFlags,
             signal: Option<c_int>)
             -> Result<pid_t> {
    extern "C" fn callback(data: *mut CloneCb) -> c_int {
        let cb: &mut CloneCb = unsafe { &mut *data };
        (*cb)() as c_int
    }

    let res = unsafe {
        let combined = flags.bits() | signal.unwrap_or(0);
        let ptr = stack.as_mut_ptr().offset(stack.len() as isize);
        ffi::clone(mem::transmute(callback as extern "C" fn(*mut Box<::std::ops::FnMut() -> isize>) -> i32),
                   ptr as *mut c_void,
                   combined,
                   &mut cb)
    };

    Errno::result(res)
}

pub fn unshare(flags: CloneFlags) -> Result<()> {
    let res = unsafe { libc::unshare(flags.bits()) };

    Errno::result(res).map(drop)
}

pub fn setns(fd: RawFd, nstype: CloneFlags) -> Result<()> {
    let res = unsafe { libc::setns(fd, nstype.bits()) };

    Errno::result(res).map(drop)
}
