//! For detailed description of the ptrace requests, consult `man ptrace`.

use std::{mem, ptr};
use {Errno, Error, Result};
use libc::{self, c_void, c_long, siginfo_t};
use ::unistd::Pid;
use sys::signal::Signal;


cfg_if! {
    if #[cfg(any(all(target_os = "linux", arch = "s390x"),
                all(target_os = "linux", target_env = "gnu")))] {
        pub type RequestType = ::libc::c_uint;
    } else {
        pub type RequestType = ::libc::c_int;
    }
}

libc_enum!{
    #[cfg_attr(all(any(all(target_os = "linux", arch = "s390x"),
    all(target_os = "linux", target_env = "gnu"))), repr(u32))] 
    #[cfg_attr(not(any(all(target_os = "linux", arch = "s390x"),
    all(target_os = "linux", target_env = "gnu"))), repr(i32))] 
    pub enum Request {
        PTRACE_TRACEME, 
        PTRACE_PEEKTEXT,
        PTRACE_PEEKDATA,
        PTRACE_PEEKUSER,
        PTRACE_POKETEXT,
        PTRACE_POKEDATA,
        PTRACE_POKEUSER,
        PTRACE_CONT,
        PTRACE_KILL,
        PTRACE_SINGLESTEP,
        #[cfg(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"))]
        PTRACE_GETREGS,
        #[cfg(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"))]
        PTRACE_SETREGS,
        #[cfg(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"))]
        PTRACE_GETFPREGS,
        #[cfg(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"))]
        PTRACE_SETFPREGS,
        PTRACE_ATTACH,
        PTRACE_DETACH,
        #[cfg(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"))]
        PTRACE_GETFPXREGS,
        #[cfg(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"))]
        PTRACE_SETFPXREGS,
        PTRACE_SYSCALL,
        PTRACE_SETOPTIONS,
        PTRACE_GETEVENTMSG,
        PTRACE_GETSIGINFO,
        PTRACE_SETSIGINFO,
        #[cfg(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"))]
        PTRACE_GETREGSET,
        #[cfg(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"))]
        PTRACE_SETREGSET,
        PTRACE_SEIZE,
        PTRACE_INTERRUPT,
        PTRACE_LISTEN,
        PTRACE_PEEKSIGINFO,
    }
}
      
libc_enum!{
    #[repr(i32)]
    pub enum Event {
        PTRACE_EVENT_FORK,
        PTRACE_EVENT_VFORK,
        PTRACE_EVENT_CLONE,
        PTRACE_EVENT_EXEC,
        PTRACE_EVENT_VFORK_DONE,
        PTRACE_EVENT_EXIT,
        PTRACE_EVENT_SECCOMP,
        // PTRACE_EVENT_STOP not provided by libc because it's defined in glibc 2.26
    }
}

libc_bitflags! {
    pub struct Options: libc::c_int {
        PTRACE_O_TRACESYSGOOD;
        PTRACE_O_TRACEFORK;
        PTRACE_O_TRACEVFORK;
        PTRACE_O_TRACECLONE;
        PTRACE_O_TRACEEXEC;
        PTRACE_O_TRACEVFORKDONE;
        PTRACE_O_TRACEEXIT;
        PTRACE_O_TRACESECCOMP;
    }
}

/// Performs a ptrace request. If the request in question is provided by a specialised function
/// this function will return an unsupported operation error.
#[deprecated(
    since="0.10.0",
    note="usages of `ptrace()` should be replaced with the specialized helper functions instead"
)]
pub unsafe fn ptrace(request: Request, pid: Pid, addr: *mut c_void, data: *mut c_void) -> Result<c_long> {
    use self::Request::*;
    match request {
        PTRACE_PEEKTEXT | PTRACE_PEEKDATA | PTRACE_PEEKUSER => ptrace_peek(request, pid, addr, data),
        PTRACE_GETSIGINFO | PTRACE_GETEVENTMSG | PTRACE_SETSIGINFO | PTRACE_SETOPTIONS => Err(Error::UnsupportedOperation),
        _ => ptrace_other(request, pid, addr, data)
    }
}

fn ptrace_peek(request: Request, pid: Pid, addr: *mut c_void, data: *mut c_void) -> Result<c_long> {
    let ret = unsafe {
        Errno::clear();
        libc::ptrace(request as RequestType, libc::pid_t::from(pid), addr, data)
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
fn ptrace_get_data<T>(request: Request, pid: Pid) -> Result<T> {
    // Creates an uninitialized pointer to store result in
    let data: T = unsafe { mem::uninitialized() };
    let res = unsafe { 
        libc::ptrace(request as RequestType, 
                     libc::pid_t::from(pid), 
                     ptr::null_mut::<T>(), 
                     &data as *const _ as *const c_void) 
    };
    Errno::result(res)?;
    Ok(data)
}

unsafe fn ptrace_other(request: Request, pid: Pid, addr: *mut c_void, data: *mut c_void) -> Result<c_long> {
    Errno::result(libc::ptrace(request as RequestType, libc::pid_t::from(pid), addr, data)).map(|_| 0)
}

/// Set options, as with `ptrace(PTRACE_SETOPTIONS,...)`.
pub fn setoptions(pid: Pid, options: Options) -> Result<()> {
    use std::ptr;

    let res = unsafe { 
        libc::ptrace(Request::PTRACE_SETOPTIONS as RequestType, 
                     libc::pid_t::from(pid), 
                     ptr::null_mut::<libc::c_void>(), 
                     options.bits() as *mut c_void) 
    };
    Errno::result(res).map(|_| ())
}

/// Gets a ptrace event as described by `ptrace(PTRACE_GETEVENTMSG,...)`
pub fn getevent(pid: Pid) -> Result<c_long> {
    ptrace_get_data::<c_long>(Request::PTRACE_GETEVENTMSG, pid)
}

/// Get siginfo as with `ptrace(PTRACE_GETSIGINFO,...)`
pub fn getsiginfo(pid: Pid) -> Result<siginfo_t> {
    ptrace_get_data::<siginfo_t>(Request::PTRACE_GETSIGINFO, pid)
}

/// Set siginfo as with `ptrace(PTRACE_SETSIGINFO,...)`
pub fn setsiginfo(pid: Pid, sig: &siginfo_t) -> Result<()> {
    let ret = unsafe{
        Errno::clear();
        libc::ptrace(Request::PTRACE_SETSIGINFO as RequestType, 
                     libc::pid_t::from(pid), 
                     ptr::null_mut::<libc::c_void>(), 
                     sig as *const _ as *const c_void)
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
        ptrace_other(
            Request::PTRACE_TRACEME,
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
        ptrace_other(
            Request::PTRACE_SYSCALL,
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
        ptrace_other(
            Request::PTRACE_ATTACH,
            pid,
            ptr::null_mut(),
            ptr::null_mut(),
        ).map(|_| ()) // ignore the useless return value
    }
}

/// Detaches the current running process, as with `ptrace(PTRACE_DETACH, ...)`
///
/// Detaches from the process specified in pid allowing it to run freely 
pub fn detach(pid: Pid) -> Result<()> {
    unsafe {
        ptrace_other(
            Request::PTRACE_DETACH,
            pid,
            ptr::null_mut(),
            ptr::null_mut()
            ).map(|_| ())
    }
}

/// Restart the stopped tracee process, as with `ptrace(PTRACE_CONT, ...)`
///
/// Continues the execution of the process with PID `pid`, optionally
/// delivering a signal specified by `sig`.
pub fn cont<T: Into<Option<Signal>>>(pid: Pid, sig: T) -> Result<()> {
    let data = match sig.into() {
        Some(s) => s as i32 as *mut c_void,
        None => ptr::null_mut(),
    };
    unsafe {
        ptrace_other(Request::PTRACE_CONT, pid, ptr::null_mut(), data).map(|_| ()) // ignore the useless return value
    }
}

