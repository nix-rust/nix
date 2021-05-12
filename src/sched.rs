use crate::{Errno, Result};

#[cfg(any(target_os = "android", target_os = "linux"))]
pub use self::sched_linux_like::*;

#[cfg(any(target_os = "android", target_os = "linux"))]
mod sched_linux_like {
    use crate::errno::Errno;
    use libc::{self, c_int, c_void};
    use std::mem;
    use std::option::Option;
    use std::os::unix::io::RawFd;
    use crate::unistd::Pid;
    use crate::{Error, Result};

    // For some functions taking with a parameter of type CloneFlags,
    // only a subset of these flags have an effect.
    libc_bitflags! {
        pub struct CloneFlags: c_int {
            CLONE_VM;
            CLONE_FS;
            CLONE_FILES;
            CLONE_SIGHAND;
            CLONE_PTRACE;
            CLONE_VFORK;
            CLONE_PARENT;
            CLONE_THREAD;
            CLONE_NEWNS;
            CLONE_SYSVSEM;
            CLONE_SETTLS;
            CLONE_PARENT_SETTID;
            CLONE_CHILD_CLEARTID;
            CLONE_DETACHED;
            CLONE_UNTRACED;
            CLONE_CHILD_SETTID;
            CLONE_NEWCGROUP;
            CLONE_NEWUTS;
            CLONE_NEWIPC;
            CLONE_NEWUSER;
            CLONE_NEWPID;
            CLONE_NEWNET;
            CLONE_IO;
        }
    }

    pub type CloneCb<'a> = Box<dyn FnMut() -> isize + 'a>;

    /// CpuSet represent a bit-mask of CPUs.
    /// CpuSets are used by sched_setaffinity and
    /// sched_getaffinity for example.
    ///
    /// This is a wrapper around `libc::cpu_set_t`.
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub struct CpuSet {
        cpu_set: libc::cpu_set_t,
    }

    impl CpuSet {
        /// Create a new and empty CpuSet.
        pub fn new() -> CpuSet {
            CpuSet {
                cpu_set: unsafe { mem::zeroed() },
            }
        }

        /// Test to see if a CPU is in the CpuSet.
        /// `field` is the CPU id to test
        pub fn is_set(&self, field: usize) -> Result<bool> {
            if field >= CpuSet::count() {
                Err(Error::from(Errno::EINVAL))
            } else {
                Ok(unsafe { libc::CPU_ISSET(field, &self.cpu_set) })
            }
        }

        /// Add a CPU to CpuSet.
        /// `field` is the CPU id to add
        pub fn set(&mut self, field: usize) -> Result<()> {
            if field >= CpuSet::count() {
                Err(Error::from(Errno::EINVAL))
            } else {
                unsafe { libc::CPU_SET(field, &mut self.cpu_set); }
                Ok(())
            }
        }

        /// Remove a CPU from CpuSet.
        /// `field` is the CPU id to remove
        pub fn unset(&mut self, field: usize) -> Result<()> {
            if field >= CpuSet::count() {
                Err(Error::from(Errno::EINVAL))
            } else {
                unsafe { libc::CPU_CLR(field, &mut self.cpu_set);}
                Ok(())
            }
        }

        /// Return the maximum number of CPU in CpuSet
        pub const fn count() -> usize {
            8 * mem::size_of::<libc::cpu_set_t>()
        }
    }

    impl Default for CpuSet {
        fn default() -> Self {
            Self::new()
        }
    }

    /// `sched_setaffinity` set a thread's CPU affinity mask
    /// ([`sched_setaffinity(2)`](https://man7.org/linux/man-pages/man2/sched_setaffinity.2.html))
    ///
    /// `pid` is the thread ID to update.
    /// If pid is zero, then the calling thread is updated.
    ///
    /// The `cpuset` argument specifies the set of CPUs on which the thread
    /// will be eligible to run.
    ///
    /// # Example
    ///
    /// Binding the current thread to CPU 0 can be done as follows:
    ///
    /// ```rust,no_run
    /// use nix::sched::{CpuSet, sched_setaffinity};
    /// use nix::unistd::Pid;
    ///
    /// let mut cpu_set = CpuSet::new();
    /// cpu_set.set(0);
    /// sched_setaffinity(Pid::from_raw(0), &cpu_set);
    /// ```
    pub fn sched_setaffinity(pid: Pid, cpuset: &CpuSet) -> Result<()> {
        let res = unsafe {
            libc::sched_setaffinity(
                pid.into(),
                mem::size_of::<CpuSet>() as libc::size_t,
                &cpuset.cpu_set,
            )
        };

        Errno::result(res).map(drop)
    }

    /// `sched_getaffinity` get a thread's CPU affinity mask
    /// ([`sched_getaffinity(2)`](https://man7.org/linux/man-pages/man2/sched_getaffinity.2.html))
    ///
    /// `pid` is the thread ID to check.
    /// If pid is zero, then the calling thread is checked.
    ///
    /// Returned `cpuset` is the set of CPUs on which the thread
    /// is eligible to run.
    ///
    /// # Example
    ///
    /// Checking if the current thread can run on CPU 0 can be done as follows:
    ///
    /// ```rust,no_run
    /// use nix::sched::sched_getaffinity;
    /// use nix::unistd::Pid;
    ///
    /// let cpu_set = sched_getaffinity(Pid::from_raw(0)).unwrap();
    /// if cpu_set.is_set(0).unwrap() {
    ///     println!("Current thread can run on CPU 0");
    /// }
    /// ```
    pub fn sched_getaffinity(pid: Pid) -> Result<CpuSet> {
        let mut cpuset = CpuSet::new();
        let res = unsafe {
            libc::sched_getaffinity(
                pid.into(),
                mem::size_of::<CpuSet>() as libc::size_t,
                &mut cpuset.cpu_set,
            )
        };

        Errno::result(res).and(Ok(cpuset))
    }

    /// `clone` create a child process
    /// ([`clone(2)`](https://man7.org/linux/man-pages/man2/clone.2.html))
    ///
    /// `stack` is a reference to an array which will hold the stack of the new
    /// process.  Unlike when calling `clone(2)` from C, the provided stack
    /// address need not be the highest address of the region.  Nix will take
    /// care of that requirement.  The user only needs to provide a reference to
    /// a normally allocated buffer.
    pub fn clone(
        mut cb: CloneCb,
        stack: &mut [u8],
        flags: CloneFlags,
        signal: Option<c_int>,
    ) -> Result<Pid> {
        extern "C" fn callback(data: *mut CloneCb) -> c_int {
            let cb: &mut CloneCb = unsafe { &mut *data };
            (*cb)() as c_int
        }

        let res = unsafe {
            let combined = flags.bits() | signal.unwrap_or(0);
            let ptr = stack.as_mut_ptr().add(stack.len());
            let ptr_aligned = ptr.sub(ptr as usize % 16);
            libc::clone(
                mem::transmute(
                    callback as extern "C" fn(*mut Box<dyn FnMut() -> isize>) -> i32,
                ),
                ptr_aligned as *mut c_void,
                combined,
                &mut cb as *mut _ as *mut c_void,
            )
        };

        Errno::result(res).map(Pid::from_raw)
    }

    pub fn unshare(flags: CloneFlags) -> Result<()> {
        let res = unsafe { libc::unshare(flags.bits()) };

        Errno::result(res).map(drop)
    }

    pub fn setns(fd: RawFd, nstype: CloneFlags) -> Result<()> {
        let res = unsafe { libc::setns(fd, nstype.bits()) };

        Errno::result(res).map(drop)
    }
}

#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use self::sched_scheduler::*;

#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
mod sched_scheduler {
    use crate::unistd::Pid;
    use crate::{Errno, Result};
    use libc::{self, c_int};
    use std::convert::TryFrom;
    use std::mem;

    libc_enum! {
        #[repr(i32)]
        #[non_exhaustive]
        pub enum SchedType {
            #[cfg(target_os = "android")]
            SCHED_NORMAL,
            #[cfg(any(target_os = "dragonfly", target_os = "freebsd", target_os = "fuchsia", target_os = "illumos", target_os = "linux", target_os = "netbsd"))]
            SCHED_OTHER,
            #[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd", target_os = "fuchsia", target_os = "illumos", target_os = "linux", target_os = "netbsd"))]
            SCHED_FIFO,
            #[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd", target_os = "fuchsia", target_os = "illumos", target_os = "linux", target_os = "netbsd"))]
            SCHED_RR,
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            SCHED_BATCH,
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            SCHED_IDLE,
            #[cfg(target_os = "android")]
            SCHED_DEADLINE,
            #[cfg(target_os = "illumos")]
            SCHED_SYS,
            #[cfg(target_os = "illumos")]
            SCHED_IA,
            #[cfg(target_os = "illumos")]
            SCHED_FSS,
            #[cfg(target_os = "illumos")]
            SCHED_FX,
        }

        impl TryFrom<i32>
    }

    libc_bitflags! {
        pub struct SchedFlags: c_int {
            #[cfg(target_os = "linux")]
            SCHED_RESET_ON_FORK;
        }
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub struct SchedPolicy {
        pub sched_type: SchedType,
        #[cfg(target_os = "linux")]
        pub sched_flags: SchedFlags,
    }

    impl SchedPolicy {
        pub fn new(sched_type: SchedType) -> Self {
            SchedPolicy::_with_flags(sched_type, SchedFlags::empty())
        }

        pub fn with_flags(sched_type: SchedType, sched_flags: SchedFlags) -> Self {
            SchedPolicy::_with_flags(sched_type, sched_flags)
        }

        #[cfg(target_os = "linux")]
        pub const fn bits(&self) -> i32 {
            self.sched_type as i32 | self.sched_flags.bits()
        }

        #[cfg(not(target_os = "linux"))]
        pub const fn bits(&self) -> i32 {
            self.sched_type as i32
        }

        #[cfg(target_os = "linux")]
        pub fn from_bits(bits: i32) -> Option<Self> {
            let type_bits = bits & !SchedFlags::all().bits();
            let sched_type = SchedType::try_from(type_bits).ok()?;
            let flag_bits = bits & SchedFlags::all().bits();
            let sched_flags = SchedFlags::from_bits(flag_bits)?;
            Some(SchedPolicy::with_flags(sched_type, sched_flags))
        }

        #[cfg(not(target_os = "linux"))]
        pub fn from_bits(bits: i32) -> Option<Self> {
            let type_bits = bits & !SchedFlags::all().bits();
            let sched_type = SchedType::try_from(type_bits).ok()?;
            Some(SchedPolicy::new(sched_type))
        }

        #[cfg(target_os = "linux")]
        #[inline]
        fn _with_flags(sched_type: SchedType, sched_flags: SchedFlags) -> Self {
            SchedPolicy {
                sched_type,
                sched_flags,
            }
        }

        #[cfg(not(target_os = "linux"))]
        #[inline]
        fn _with_flags(sched_type: SchedType, _sched_flags: SchedFlags) -> Self {
            SchedPolicy {
                sched_type,
            }
        }
    }

    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub struct SchedParam(libc::sched_param);

    impl SchedParam {
        pub fn new(priority: i32) -> Self {
            let sched_param = unsafe {
                // Illumos's sched_param has a private padding field.
                let mut param = mem::MaybeUninit::<libc::sched_param>::zeroed();
                let p = param.as_mut_ptr();
                (*p).sched_priority = priority;
                param.assume_init()
            };
            SchedParam(sched_param)
        }

        pub const fn priority(&self) -> i32 {
            self.0.sched_priority
        }
    }

    impl Default for SchedParam {
        fn default() -> Self {
            SchedParam::new(0)
        }
    }

    /// Get minimum priority value for policy
    ///
    /// See also [`sched_get_priority_min(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/sched_get_priority_min.html)
    pub fn sched_get_priority_min(policy: SchedPolicy) -> Result<i32> {
        let res = unsafe { libc::sched_get_priority_min(policy.bits()) };

        Errno::result(res)
    }

    /// Get maximum priority value for policy
    ///
    /// See also [`sched_get_priority_max(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/sched_get_priority_max.html)
    pub fn sched_get_priority_max(policy: SchedPolicy) -> Result<i32> {
        let res = unsafe { libc::sched_get_priority_max(policy.bits()) };

        Errno::result(res)
    }

    /// Set thread's scheduling parameters
    ///
    /// `pid` is the thread ID to update.
    /// If `pid` is None or zero, then the parameters for the calling thread are set.
    ///
    /// See also [`sched_setparam(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/sched_setparam.html)
    pub fn sched_setparam(pid: Option<Pid>, sched_param: SchedParam) -> Result<()> {
        let res =
            unsafe { libc::sched_setparam(pid.unwrap_or_else(|| Pid::from_raw(0)).into(), &sched_param.0) };

        Errno::result(res).map(drop)
    }

    /// Get thread's scheduling parameters
    ///
    /// `pid` is the thread ID to check.
    /// If `pid` is None or zero, then the parameters for the calling thread are retrieved.
    ///
    /// See also [`sched_getparam(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/sched_getparam.html)
    pub fn sched_getparam(pid: Option<Pid>) -> Result<SchedParam> {
        let mut sched_param = mem::MaybeUninit::uninit();
        let res = unsafe {
            libc::sched_getparam(
                pid.unwrap_or_else(|| Pid::from_raw(0)).into(),
                sched_param.as_mut_ptr(),
            )
        };

        Errno::result(res).map(|_| unsafe { SchedParam(sched_param.assume_init()) })
    }

    /// Set thread's scheduling policy and parameters
    ///
    /// `pid` is the thread ID to update.
    /// If `pid` is None or zero, then the policy and parameters for the calling thread are set.
    ///
    /// See also [`sched_setscheduler(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/sched_setscheduler.html)
    pub fn sched_setscheduler(
        pid: Option<Pid>,
        policy: SchedPolicy,
        sched_param: SchedParam,
    ) -> Result<()> {
        let res = unsafe {
            libc::sched_setscheduler(
                pid.unwrap_or_else(|| Pid::from_raw(0)).into(),
                policy.bits(),
                &sched_param.0,
            )
        };

        Errno::result(res).map(drop)
    }

    /// Get thread's scheduling policy and parameters
    ///
    /// `pid` is the thread ID to check.
    /// If `pid` is None or zero, then the policy and parameters for the calling thread are retrieved.
    ///
    /// See also [`sched_getscheduler(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/sched_getscheduler.html)
    pub fn sched_getscheduler(pid: Option<Pid>) -> Result<SchedPolicy> {
        let res = unsafe { libc::sched_getscheduler(pid.unwrap_or_else(|| Pid::from_raw(0)).into()) };

        Errno::result(res).and(SchedPolicy::from_bits(res).ok_or(Errno::EINVAL))
    }
}

/// Explicitly yield the processor to other threads.
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/sched_yield.html)
pub fn sched_yield() -> Result<()> {
    let res = unsafe { libc::sched_yield() };

    Errno::result(res).map(drop)
}
