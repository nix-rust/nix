//! For detailed description of the ptrace requests, consult `man ptrace`.

use std::{mem, ptr};
use {Errno, Error, Result};
use libc::{c_void, c_long, siginfo_t};
use ::unistd::Pid;

pub mod ptrace {
    use libc::c_int;

    pub type PtraceRequest = c_int;

    pub const PTRACE_TRACEME:     PtraceRequest = 0;
    pub const PTRACE_PEEKTEXT:    PtraceRequest = 1;
    pub const PTRACE_PEEKDATA:    PtraceRequest = 2;
    pub const PTRACE_PEEKUSER:    PtraceRequest = 3;
    pub const PTRACE_POKETEXT:    PtraceRequest = 4;
    pub const PTRACE_POKEDATA:    PtraceRequest = 5;
    pub const PTRACE_POKEUSER:    PtraceRequest = 6;
    pub const PTRACE_CONT:        PtraceRequest = 7;
    pub const PTRACE_KILL:        PtraceRequest = 8;
    pub const PTRACE_SINGLESTEP:  PtraceRequest = 9;
    pub const PTRACE_GETREGS:     PtraceRequest = 12;
    pub const PTRACE_SETREGS:     PtraceRequest = 13;
    pub const PTRACE_GETFPREGS:   PtraceRequest = 14;
    pub const PTRACE_SETFPREGS:   PtraceRequest = 15;
    pub const PTRACE_ATTACH:      PtraceRequest = 16;
    pub const PTRACE_DETACH:      PtraceRequest = 17;
    pub const PTRACE_GETFPXREGS:  PtraceRequest = 18;
    pub const PTRACE_SETFPXREGS:  PtraceRequest = 19;
    pub const PTRACE_SYSCALL:     PtraceRequest = 24;
    pub const PTRACE_SETOPTIONS:  PtraceRequest = 0x4200;
    pub const PTRACE_GETEVENTMSG: PtraceRequest = 0x4201;
    pub const PTRACE_GETSIGINFO:  PtraceRequest = 0x4202;
    pub const PTRACE_SETSIGINFO:  PtraceRequest = 0x4203;
    pub const PTRACE_GETREGSET:   PtraceRequest = 0x4204;
    pub const PTRACE_SETREGSET:   PtraceRequest = 0x4205;
    pub const PTRACE_SEIZE:       PtraceRequest = 0x4206;
    pub const PTRACE_INTERRUPT:   PtraceRequest = 0x4207;
    pub const PTRACE_LISTEN:      PtraceRequest = 0x4208;
    pub const PTRACE_PEEKSIGINFO: PtraceRequest = 0x4209;

    pub type PtraceEvent = c_int;

    pub const PTRACE_EVENT_FORK:       PtraceEvent = 1;
    pub const PTRACE_EVENT_VFORK:      PtraceEvent = 2;
    pub const PTRACE_EVENT_CLONE:      PtraceEvent = 3;
    pub const PTRACE_EVENT_EXEC:       PtraceEvent = 4;
    pub const PTRACE_EVENT_VFORK_DONE: PtraceEvent = 5;
    pub const PTRACE_EVENT_EXIT:       PtraceEvent = 6;
    pub const PTRACE_EVENT_SECCOMP:    PtraceEvent = 6;
    pub const PTRACE_EVENT_STOP:       PtraceEvent = 128;

    pub type PtraceOptions = c_int;
    pub const PTRACE_O_TRACESYSGOOD: PtraceOptions   = 1;
    pub const PTRACE_O_TRACEFORK: PtraceOptions      = (1 << PTRACE_EVENT_FORK);
    pub const PTRACE_O_TRACEVFORK: PtraceOptions     = (1 << PTRACE_EVENT_VFORK);
    pub const PTRACE_O_TRACECLONE: PtraceOptions     = (1 << PTRACE_EVENT_CLONE);
    pub const PTRACE_O_TRACEEXEC: PtraceOptions      = (1 << PTRACE_EVENT_EXEC);
    pub const PTRACE_O_TRACEVFORKDONE: PtraceOptions = (1 << PTRACE_EVENT_VFORK_DONE);
    pub const PTRACE_O_TRACEEXIT: PtraceOptions      = (1 << PTRACE_EVENT_EXIT);
    pub const PTRACE_O_TRACESECCOMP: PtraceOptions   = (1 << PTRACE_EVENT_SECCOMP);
}

mod ffi {
    use libc::{pid_t, c_int, c_long, c_void};

    extern {
        pub fn ptrace(request: c_int, pid: pid_t, addr: * const c_void, data: * const c_void) -> c_long;
    }
}

/// Performs a ptrace request. If the request in question is provided by a specialised function
/// this function will return an unsupported operation error.
pub unsafe fn ptrace(request: ptrace::PtraceRequest, pid: Pid, addr: *mut c_void, data: *mut c_void) -> Result<c_long> {
    use self::ptrace::*;

    match request {
        PTRACE_PEEKTEXT | PTRACE_PEEKDATA | PTRACE_PEEKUSER => ptrace_peek(request, pid, addr, data),
        PTRACE_GETSIGINFO | PTRACE_GETEVENTMSG | PTRACE_SETSIGINFO | PTRACE_SETOPTIONS => Err(Error::UnsupportedOperation),
        _ => ptrace_other(request, pid, addr, data)
    }
}

fn ptrace_peek(request: ptrace::PtraceRequest, pid: Pid, addr: *mut c_void, data: *mut c_void) -> Result<c_long> {
    let ret = unsafe {
        Errno::clear();
        ffi::ptrace(request, pid.into(), addr, data)
    };
    match Errno::result(ret) {
        Ok(..) | Err(Error::Sys(Errno::UnknownErrno)) => Ok(ret),
        err @ Err(..) => err,
    }
}

/// Function for ptrace requests that return values from the data field.
/// Some ptrace get requests populate structs or larger elements than c_long
/// and therefore use the data field to return values. This function handles these
/// requests.
fn ptrace_get_data<T>(request: ptrace::PtraceRequest, pid: Pid) -> Result<T> {
    // Creates an uninitialized pointer to store result in
    let data: T = unsafe { mem::uninitialized() };
    let res = unsafe { ffi::ptrace(request, pid.into(), ptr::null_mut(), &data as *const _ as *const c_void) };
    Errno::result(res)?;
    Ok(data)
}

fn ptrace_other(request: ptrace::PtraceRequest, pid: Pid, addr: *mut c_void, data: *mut c_void) -> Result<c_long> {
    Errno::result(unsafe { ffi::ptrace(request, pid.into(), addr, data) }).map(|_| 0)
}

/// Set options, as with `ptrace(PTRACE_SETOPTIONS,...)`.
pub fn setoptions(pid: Pid, options: ptrace::PtraceOptions) -> Result<()> {
    use self::ptrace::*;
    use std::ptr;

    let res = unsafe { ffi::ptrace(PTRACE_SETOPTIONS, pid.into(), ptr::null_mut(), options as *mut c_void) };
    Errno::result(res).map(|_| ())
}

/// Gets a ptrace event as described by `ptrace(PTRACE_GETEVENTMSG,...)`
pub fn getevent(pid: Pid) -> Result<c_long> {
    use self::ptrace::*;
    ptrace_get_data::<c_long>(PTRACE_GETEVENTMSG, pid)
}

/// Get siginfo as with `ptrace(PTRACE_GETSIGINFO,...)`
pub fn getsiginfo(pid: Pid) -> Result<siginfo_t> {
    use self::ptrace::*;
    ptrace_get_data::<siginfo_t>(PTRACE_GETSIGINFO, pid)
}

/// Set siginfo as with `ptrace(PTRACE_SETSIGINFO,...)`
pub fn setsiginfo(pid: Pid, sig: &siginfo_t) -> Result<()> {
    use self::ptrace::*;
    let ret = unsafe{
        Errno::clear();
        ffi::ptrace(PTRACE_SETSIGINFO, pid.into(), ptr::null_mut(), sig as *const _ as *const c_void)
    };
    match Errno::result(ret) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Sets the process as traceable, as with `ptrace(PTRACE_TRACEME, ...)`
///
/// Indicates that this process is to be traced by its parent.
/// This is the only ptrace request to be issued by the tracee.
pub fn traceme() -> Result<()> {
    unsafe {
        ptrace(
            ptrace::PTRACE_TRACEME,
            Pid::from_raw(0),
            ptr::null_mut(),
            ptr::null_mut(),
        ).map(|_| ()) // ignore the useless return value
    }
}

/// Ask for next syscall, as with `ptrace(PTRACE_SYSCALL, ...)`
///
/// Arranges for the tracee to be stopped at the next entry to or exit from a system call.
pub fn syscall(pid: Pid) -> Result<()> {
    unsafe {
        ptrace(
            ptrace::PTRACE_SYSCALL,
            pid,
            ptr::null_mut(),
            ptr::null_mut(),
        ).map(|_| ()) // ignore the useless return value
    }
}

/// Attach to a running process, as with `ptrace(PTRACE_ATTACH, ...)`
///
/// Attaches to the process specified in pid, making it a tracee of the calling process.
pub fn attach(pid: Pid) -> Result<()> {
    unsafe {
        ptrace(
            ptrace::PTRACE_ATTACH,
            pid,
            ptr::null_mut(),
            ptr::null_mut(),
        ).map(|_| ()) // ignore the useless return value
    }
}

/// Restart the stopped tracee process, as with `ptrace(PTRACE_CONT, ...)`
///
/// No stop request is done as a part of this call.
/// No signal is delivered to the process.
pub fn cont(pid: Pid) -> Result<()> {
    unsafe {
        ptrace(
            ptrace::PTRACE_CONT,
            pid,
            ptr::null_mut(),
            ptr::null_mut(),
        ).map(|_| ()) // ignore the useless return value
    }
}
