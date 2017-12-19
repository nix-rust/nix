use libc::{self, c_ulong, c_int};

use {Errno, Result};

libc_enum!{
    /// PrctlOption enum defining the action to be taken.
    #[repr(i32)]
    pub enum PrctlOption {
        PR_CAP_AMBIENT,
        PR_CAPBSET_READ,
        PR_CAPBSET_DROP,
        PR_SET_CHILD_SUBREAPER,
        PR_GET_CHILD_SUBREAPER,
        PR_SET_DUMPABLE,
        PR_SET_ENDIAN,
        PR_GET_ENDIAN,
        PR_SET_FP_MODE,
        PR_GET_FP_MODE,
        PR_SET_FPEMU,
        PR_GET_FPEMU,
        PR_SET_FPEXC,
        PR_GET_FPEXC,
        PR_SET_KEEPCAPS,
        PR_GET_KEEPCAPS,
        PR_MCE_KILL,
        PR_MCE_KILL_GET,
        PR_SET_MM,
        PR_MPX_ENABLE_MANAGEMENT,
        PR_MPX_DISABLE_MANAGEMENT,
        PR_SET_NAME,
        PR_GET_NAME,
        PR_SET_NO_NEW_PRIVS,
        PR_GET_NO_NEW_PRIVS,
        PR_SET_PDEATHSIG,
        PR_GET_PDEATHSIG,
        PR_SET_PTRACER,
        PR_SET_SECCOMP,
        PR_GET_SECCOMP,
        PR_SET_SECUREBITS,
        PR_GET_SECUREBITS,
        PR_SET_THP_DISABLE,
        PR_TASK_PERF_EVENTS_DISABLE,
        PR_TASK_PERF_EVENTS_ENABLE,
        PR_GET_THP_DISABLE,
        PR_GET_TID_ADDRESS,
        PR_SET_TIMERSLACK,
        PR_GET_TIMERSLACK,
        PR_SET_TIMING,
        PR_GET_TIMING,
        PR_SET_TSC,
        PR_GET_TSC,
        PR_SET_UNALIGN,
        PR_GET_UNALIGN
    }
}

/// Apply an operation on a process
/// [prctl(2)](http://man7.org/linux/man-pages/man2/prctl.2.html)
///
/// prctl is called with a first argument describing what to do,
/// further arguments with a significance depending on the first one.
pub fn prctl(option: PrctlOption, arg2: c_ulong, arg3: c_ulong, arg4: c_ulong, arg5: c_ulong) -> Result<()> {
    let res = unsafe { libc::prctl(option as c_int, arg2, arg3, arg4, arg5) };

    Errno::result(res).map(drop)
}
