use std::{mem, ops};
use std::os::unix::io::RawFd;
use libc::{self, c_int, c_void, c_ulong, pid_t};
use {Errno, Result};

// For some functions taking with a parameter of type CloneFlags,
// only a subset of these flags have an effect.
bitflags!{
    flags CloneFlags: c_int {
        const CLONE_VM             = libc::CLONE_VM,
        const CLONE_FS             = libc::CLONE_FS,
        const CLONE_FILES          = libc::CLONE_FILES,
        const CLONE_SIGHAND        = libc::CLONE_SIGHAND,
        const CLONE_PTRACE         = libc::CLONE_PTRACE,
        const CLONE_VFORK          = libc::CLONE_VFORK,
        const CLONE_PARENT         = libc::CLONE_PARENT,
        const CLONE_THREAD         = libc::CLONE_THREAD,
        const CLONE_NEWNS          = libc::CLONE_NEWNS,
        const CLONE_SYSVSEM        = libc::CLONE_SYSVSEM,
        const CLONE_SETTLS         = libc::CLONE_SETTLS,
        const CLONE_PARENT_SETTID  = libc::CLONE_PARENT_SETTID,
        const CLONE_CHILD_CLEARTID = libc::CLONE_CHILD_CLEARTID,
        const CLONE_DETACHED       = libc::CLONE_DETACHED,
        const CLONE_UNTRACED       = libc::CLONE_UNTRACED,
        const CLONE_CHILD_SETTID   = libc::CLONE_CHILD_SETTID,
        // TODO: Once, we use a version containing
        // https://github.com/rust-lang-nursery/libc/pull/147
        // get rid of the casts.
        const CLONE_NEWUTS         = libc::CLONE_NEWUTS as c_int,
        const CLONE_NEWIPC         = libc::CLONE_NEWIPC as c_int,
        const CLONE_NEWUSER        = libc::CLONE_NEWUSER as c_int,
        const CLONE_NEWPID         = libc::CLONE_NEWPID as c_int,
        const CLONE_NEWNET         = libc::CLONE_NEWNET as c_int,
        const CLONE_IO             = libc::CLONE_IO as c_int,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CloneFlagsArg {
    flags: CloneFlags,
    signal: Option<c_int>,
}

impl CloneFlagsArg {
    fn bits(self) -> c_int {
        self.flags.bits() | self.signal.unwrap_or(0)
    }
}

impl ops::BitOr<c_int> for CloneFlags {
    type Output = CloneFlagsArg;
    fn bitor(self, rhs: c_int) -> CloneFlagsArg {
        CloneFlagsArg {
            flags: self,
            signal: Some(rhs),
        }
    }
}

impl From<CloneFlags> for CloneFlagsArg {
    fn from(flags: CloneFlags) -> CloneFlagsArg {
        CloneFlagsArg {
            flags: flags,
            signal: None,
        }
    }
}



// Support a maximum CPU set of 1024 nodes
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
mod cpuset_attribs {
    use super::CpuMask;
    pub const CPU_SETSIZE:           usize = 1024;
    pub const CPU_MASK_BITS:         usize = 64;

    #[inline]
    pub fn set_cpu_mask_flag(cur: CpuMask, bit: usize) -> CpuMask {
        cur | (1u64 << bit)
    }

    #[inline]
    pub fn clear_cpu_mask_flag(cur: CpuMask, bit: usize) -> CpuMask {
        cur & !(1u64 << bit)
    }
}

#[cfg(all(target_arch = "x86", target_os = "linux"))]
mod cpuset_attribs {
    use super::CpuMask;
    pub const CPU_SETSIZE:           usize = 1024;
    pub const CPU_MASK_BITS:         usize = 32;

    #[inline]
    pub fn set_cpu_mask_flag(cur: CpuMask, bit: usize) -> CpuMask {
        cur | (1u32 << bit)
    }

    #[inline]
    pub fn clear_cpu_mask_flag(cur: CpuMask, bit: usize) -> CpuMask {
        cur & !(1u32 << bit)
    }
}

#[cfg(all(target_arch = "aarch64", any(target_os = "linux", target_os = "android")))]
mod cpuset_attribs {
    use super::CpuMask;
    pub const CPU_SETSIZE:           usize = 1024;
    pub const CPU_MASK_BITS:         usize = 64;

    #[inline]
    pub fn set_cpu_mask_flag(cur: CpuMask, bit: usize) -> CpuMask {
        cur | (1u64 << bit)
    }

    #[inline]
    pub fn clear_cpu_mask_flag(cur: CpuMask, bit: usize) -> CpuMask {
        cur & !(1u64 << bit)
    }
}

#[cfg(all(any(target_arch = "arm", target_arch = "mips"), target_os = "android"))]
mod cpuset_attribs {
    use super::CpuMask;
    // bionic only supports up to 32 independent CPUs, instead of 1024.
    pub const CPU_SETSIZE:          usize = 32;
    pub const CPU_MASK_BITS:        usize = 32;

    #[inline]
    pub fn set_cpu_mask_flag(cur: CpuMask, bit: usize) -> CpuMask {
        cur | (1u32 << bit)
    }

    #[inline]
    pub fn clear_cpu_mask_flag(cur: CpuMask, bit: usize) -> CpuMask {
        cur & !(1u32 << bit)
    }
}

#[cfg(all(any(target_arch = "arm", target_arch = "mips"), target_os = "linux"))]
mod cpuset_attribs {
    use super::CpuMask;
    pub const CPU_SETSIZE:          usize = 1024;
    pub const CPU_MASK_BITS:        usize = 32;

    #[inline]
    pub fn set_cpu_mask_flag(cur: CpuMask, bit: usize) -> CpuMask {
        cur | (1u32 << bit)
    }

    #[inline]
    pub fn clear_cpu_mask_flag(cur: CpuMask, bit: usize) -> CpuMask {
        cur & !(1u32 << bit)
    }
}

pub type CloneCb<'a> = Box<FnMut() -> isize + 'a>;

// A single CPU mask word
pub type CpuMask = c_ulong;

// Structure representing the CPU set to apply
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CpuSet {
    cpu_mask: [CpuMask; cpuset_attribs::CPU_SETSIZE/cpuset_attribs::CPU_MASK_BITS]
}

impl CpuSet {
    pub fn new() -> CpuSet {
        CpuSet {
            cpu_mask: unsafe { mem::zeroed() }
        }
    }

    pub fn set(&mut self, field: usize) {
        let word = field / cpuset_attribs::CPU_MASK_BITS;
        let bit = field % cpuset_attribs::CPU_MASK_BITS;

        self.cpu_mask[word] = cpuset_attribs::set_cpu_mask_flag(self.cpu_mask[word], bit);
    }

    pub fn unset(&mut self, field: usize) {
        let word = field / cpuset_attribs::CPU_MASK_BITS;
        let bit = field % cpuset_attribs::CPU_MASK_BITS;

        self.cpu_mask[word] = cpuset_attribs::clear_cpu_mask_flag(self.cpu_mask[word], bit);
    }
}

mod ffi {
    use libc::{c_void, c_int, pid_t, size_t};
    use super::CpuSet;

    pub type CloneCb = extern "C" fn (data: *const super::CloneCb) -> c_int;

    // We cannot give a proper #[repr(C)] to super::CloneCb
    #[allow(improper_ctypes)]
    extern {
        // create a child process
        // doc: http://man7.org/linux/man-pages/man2/clone.2.html
        pub fn clone(
            cb: *const CloneCb,
            child_stack: *mut c_void,
            flags: c_int,
            arg: *mut super::CloneCb,
            ...) -> c_int;

        // disassociate parts of the process execution context
        // doc: http://man7.org/linux/man-pages/man2/unshare.2.html
        pub fn unshare(flags: c_int) -> c_int;

        // reassociate thread with a namespace
        // doc: http://man7.org/linux/man-pages/man2/setns.2.html
        pub fn setns(fd: c_int, nstype: c_int) -> c_int;

        // Set the current CPU set that a task is allowed to run on
        pub fn sched_setaffinity(__pid: pid_t, __cpusetsize: size_t, __cpuset: *const CpuSet) -> c_int;
    }
}

pub fn sched_setaffinity(pid: isize, cpuset: &CpuSet) -> Result<()> {
    use libc::{pid_t, size_t};

    let res = unsafe {
        ffi::sched_setaffinity(pid as pid_t, mem::size_of::<CpuSet>() as size_t, mem::transmute(cpuset))
    };

    Errno::result(res).map(drop)
}

/// The clone(2) system call.
///
/// # Examples
///
/// ```no_run
/// use std::thread;
/// use std::time::Duration;
///
/// use nix::sched::clone;
/// use nix::sched::CLONE_NEWUTS;
/// use nix::sys::signal::SIGCHLD;
///
/// let mut stack = Box::new([0u8; 1024 * 1024]);
/// let pid = clone(Box::new(|| {
///     thread::sleep(Duration::from_secs(5));
///     0
/// }), &mut stack[..], CLONE_NEWUTS | SIGCHLD).unwrap();
/// ```
pub fn clone<F: Into<CloneFlagsArg>>(mut cb: CloneCb, stack: &mut [u8], flags: F) -> Result<pid_t> {
    extern "C" fn callback(data: *mut CloneCb) -> c_int {
        let cb: &mut CloneCb = unsafe { &mut *data };
        (*cb)() as c_int
    }

    let flags: CloneFlagsArg = flags.into();

    let res = unsafe {
        let ptr = stack.as_mut_ptr().offset(stack.len() as isize);
        ffi::clone(mem::transmute(callback), ptr as *mut c_void, flags.bits(), &mut cb)
    };

    Errno::result(res)
}

pub fn unshare(flags: CloneFlags) -> Result<()> {
    let res = unsafe { ffi::unshare(flags.bits()) };

    Errno::result(res).map(drop)
}

pub fn setns(fd: RawFd, nstype: CloneFlags) -> Result<()> {
    let res = unsafe { ffi::setns(fd, nstype.bits()) };

    Errno::result(res).map(drop)
}
