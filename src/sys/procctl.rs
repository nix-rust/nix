//! procctl provides programmatic control over processes.
//!
//! Here we act on the current process, so we save a getpid syscall here.
//!
//! For more documentation, please read [procctl(2)](https://man.freebsd.org/cgi/man.cgi?query=procctl&sektion=2)
use crate::errno::Errno;
use crate::sys::signal::Signal;
use crate::Result;

use libc::c_int;
use std::convert::TryFrom;

/// Enable/disable tracing on the current process, allowing debugging and core dump generation.
pub fn set_dumpable(attribute: bool) -> Result<()> {
    let mut dumpable = match attribute {
        true => libc::PROC_TRACE_CTL_ENABLE,
        false => libc::PROC_TRACE_CTL_DISABLE
    };

    let res = unsafe { libc::procctl(libc::P_PID, 0, libc::PROC_TRACE_CTL, &mut dumpable as *mut c_int as _) };
    Errno::result(res).map(drop)
}

/// Get the tracing status of the current process.
pub fn get_dumpable() -> Result<bool> {
    let mut dumpable: c_int = 0;

    let res = unsafe { libc::procctl(libc::P_PID, 0, libc::PROC_TRACE_STATUS, &mut dumpable as *mut c_int as _) };
    match Errno::result(res) {
        Ok(_) => Ok(matches!(dumpable, libc::PROC_TRACE_CTL_ENABLE)),
        Err(e) => Err(e),
    }
}

/// Set the delivery of the `signal` when the parent of the calling process exits.
pub fn set_pdeathsig<T: Into<Option<Signal>>>(signal: T) -> Result<()> {
    let mut sig = match signal.into() {
        Some(s) => s as c_int,
        None => 0,
    };

    let res = unsafe { libc::procctl(libc::P_PID, 0, libc::PROC_PDEATHSIG_CTL, &mut sig as *mut c_int as _) };
    Errno::result(res).map(drop)
}

/// Get the current signal id that will be delivered to the parent process when it's exiting.
pub fn get_pdeathsig() -> Result<Option<Signal>> {
    let mut sig: c_int = 0;

    let res = unsafe { libc::procctl(libc::P_PID, 0, libc::PROC_PDEATHSIG_STATUS, &mut sig as *mut c_int as _) };
    match Errno::result(res) {
        Ok(_) => Ok(match sig {
            0 => None,
            _ => Some(Signal::try_from(sig)?),
        }),
        Err(e) => Err(e),
    }
}
