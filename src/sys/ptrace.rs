//! Interface for `ptrace`
//!
//! For detailed description of the ptrace requests, consult [`ptrace`(2)].
//! [`ptrace`(2)]: http://man7.org/linux/man-pages/man2/ptrace.2.html

use std::{mem, ptr};
use {Errno, Error, Result};
use libc::{self, c_void, c_long, siginfo_t};
use ::unistd::Pid;
use sys::signal::Signal;


cfg_if! {
    if #[cfg(any(all(target_os = "linux", arch = "s390x"),
                 all(target_os = "linux", target_env = "gnu")))] {
        #[doc(hidden)]
        pub type RequestType = ::libc::c_uint;
    } else {
        #[doc(hidden)]
        pub type RequestType = ::libc::c_int;
    }
}

libc_enum!{
    #[cfg_attr(not(any(target_env = "musl", target_os = "android")), repr(u32))]
    #[cfg_attr(any(target_env = "musl", target_os = "android"), repr(i32))]
    /// Ptrace Request enum defining the action to be taken.
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
        #[cfg(all(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"), not(target_os = "android")))]
        PTRACE_GETREGS,
        #[cfg(all(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"), not(target_os = "android")))]
        PTRACE_SETREGS,
        #[cfg(all(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"), not(target_os = "android")))]
        PTRACE_GETFPREGS,
        #[cfg(all(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"), not(target_os = "android")))]
        PTRACE_SETFPREGS,
        PTRACE_ATTACH,
        PTRACE_DETACH,
        #[cfg(all(any(target_env = "musl", target_arch ="x86_64"), not(target_os = "android")))]
        PTRACE_GETFPXREGS,
        #[cfg(all(any(target_env = "musl", target_arch ="x86_64"), not(target_os = "android")))]
        PTRACE_SETFPXREGS,
        PTRACE_SYSCALL,
        PTRACE_SETOPTIONS,
        PTRACE_GETEVENTMSG,
        PTRACE_GETSIGINFO,
        PTRACE_SETSIGINFO,
        #[cfg(all(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"), not(target_os = "android")))]
        PTRACE_GETREGSET,
        #[cfg(all(any(target_env = "musl", target_arch ="x86_64", target_arch = "s390x"), not(target_os = "android")))]
        PTRACE_SETREGSET,
        #[cfg(not(any(target_os = "android", target_arch = "mips", target_arch = "mips64")))]
        PTRACE_SEIZE,
        #[cfg(not(any(target_os = "android", target_arch = "mips", target_arch = "mips64")))]
        PTRACE_INTERRUPT,
        #[cfg(not(any(target_os = "android", target_arch = "mips", target_arch = "mips64")))]
        PTRACE_LISTEN,
        #[cfg(not(any(target_os = "android", target_arch = "mips", target_arch = "mips64")))]
        PTRACE_PEEKSIGINFO,
    }
}

libc_enum!{
    #[repr(i32)]
    /// Using the ptrace options the tracer can configure the tracee to stop
    /// at certain events. This enum is used to define those events as defined
    /// in `man ptrace`.
    pub enum Event {
        /// Event that stops before a return from fork or clone.
        PTRACE_EVENT_FORK,
        /// Event that stops before a return from vfork or clone.
        PTRACE_EVENT_VFORK,
        /// Event that stops before a return from clone.
        PTRACE_EVENT_CLONE,
        /// Event that stops before a return from execve.
        PTRACE_EVENT_EXEC,
        /// Event for a return from vfork.
        PTRACE_EVENT_VFORK_DONE,
        /// Event for a stop before an exit. Unlike the waitpid Exit status program.
        /// registers can still be examined
        PTRACE_EVENT_EXIT,
        /// STop triggered by a seccomp rule on a tracee.
        PTRACE_EVENT_SECCOMP,
        // PTRACE_EVENT_STOP not provided by libc because it's defined in glibc 2.26
    }
}

libc_bitflags! {
    /// Ptrace options used in conjunction with the PTRACE_SETOPTIONS request.
    /// See `man ptrace` for more details.
    pub struct Options: libc::c_int {
        /// When delivering system call traps set a bit to allow tracer to
        /// distinguish between normal stops or syscall stops. May not work on
        /// all systems.
        PTRACE_O_TRACESYSGOOD;
        /// Stop tracee at next fork and start tracing the forked process.
        PTRACE_O_TRACEFORK;
        /// Stop tracee at next vfork call and trace the vforked process.
        PTRACE_O_TRACEVFORK;
        /// Stop tracee at next clone call and trace the cloned process.
        PTRACE_O_TRACECLONE;
        /// Stop tracee at next execve call.
        PTRACE_O_TRACEEXEC;
        /// Stop tracee at vfork completion.
        PTRACE_O_TRACEVFORKDONE;
        /// Stop tracee at next exit call. Stops before exit commences allowing
        /// tracer to see location of exit and register states.
        PTRACE_O_TRACEEXIT;
        /// Stop tracee when a SECCOMP_RET_TRACE rule is triggered. See `man seccomp` for more
        /// details.
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

unsafe fn ptrace_peek(
    request: Request,
    pid: Pid,
    addr: *mut c_void,
    data: *mut c_void
) -> Result<c_long> {

    Errno::clear();
    let ret = libc::ptrace(
        request as RequestType,
        libc::pid_t::from(pid),
        addr,
        data
    );
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
        ptrace_other(Request::PTRACE_ATTACH, pid, ptr::null_mut(), ptr::null_mut()).map(|_| ())
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

/// Represents all possible ptrace-accessible registers on x86_64
#[cfg(target_arch = "x86_64")]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Register {
    R15 = 8 * ::libc::R15 as isize,
    R14 = 8 * ::libc::R14 as isize,
    R13 = 8 * ::libc::R13 as isize,
    R12 = 8 * ::libc::R12 as isize,
    RBP = 8 * ::libc::RBP as isize,
    RBX = 8 * ::libc::RBX as isize,
    R11 = 8 * ::libc::R11 as isize,
    R10 = 8 * ::libc::R10 as isize,
    R9 = 8 * ::libc::R9 as isize,
    R8 = 8 * ::libc::R8 as isize,
    RAX = 8 * ::libc::RAX as isize,
    RCX = 8 * ::libc::RCX as isize,
    RDX = 8 * ::libc::RDX as isize,
    RSI = 8 * ::libc::RSI as isize,
    RDI = 8 * ::libc::RDI as isize,
    ORIG_RAX = 8 * ::libc::ORIG_RAX as isize,
    RIP = 8 * ::libc::RIP as isize,
    CS = 8 * ::libc::CS as isize,
    EFLAGS = 8 * ::libc::EFLAGS as isize,
    RSP = 8 * ::libc::RSP as isize,
    SS = 8 * ::libc::SS as isize,
    FS_BASE = 8 * ::libc::FS_BASE as isize,
    GS_BASE = 8 * ::libc::GS_BASE as isize,
    DS = 8 * ::libc::DS as isize,
    ES = 8 * ::libc::ES as isize,
    FS = 8 * ::libc::FS as isize,
    GS = 8 * ::libc::GS as isize,
}

/// Represents all possible ptrace-accessible registers on x86
#[cfg(target_arch = "x86")]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Register {
    EBX = 4 * ::libc::EBX as isize,
    ECX = 4 * ::libc::ECX as isize,
    EDX = 4 * ::libc::EDX as isize,
    ESI = 4 * ::libc::ESI as isize,
    EDI = 4 * ::libc::EDI as isize,
    EBP = 4 * ::libc::EBP as isize,
    EAX = 4 * ::libc::EAX as isize,
    DS = 4 * ::libc::DS as isize,
    ES = 4 * ::libc::ES as isize,
    FS = 4 * ::libc::FS as isize,
    GS = 4 * ::libc::GS as isize,
    ORIG_EAX = 4 * ::libc::ORIG_EAX as isize,
    EIP = 4 * ::libc::EIP as isize,
    CS = 4 * ::libc::CS as isize,
    EFL = 4 * ::libc::EFL as isize,
    UESP = 4 * ::libc::UESP as isize,
    SS = 4 * ::libc::SS as isize,
}

/// Returns the register containing nth register argument.
///
/// 0th argument is considered to be the syscall number.
/// Please note that these mappings are only valid for 64-bit programs.
/// Use [`syscall_arg32`] for tracing 32-bit programs instead.
///
/// [`syscall_arg32`]: macro.syscall_arg32.html
/// # Examples
///
/// ```
/// # #[macro_use] extern crate nix;
/// # fn main() {
/// assert_eq!(syscall_arg!(1), nix::sys::ptrace::Register::RDI);
/// # }
#[cfg(target_arch = "x86_64")]
#[macro_export]
macro_rules! syscall_arg {
    (0) => ($crate::sys::ptrace::Register::ORIG_RAX);
    (1) => ($crate::sys::ptrace::Register::RDI);
    (2) => ($crate::sys::ptrace::Register::RSI);
    (3) => ($crate::sys::ptrace::Register::RDX);
    (4) => ($crate::sys::ptrace::Register::R10);
    (5) => ($crate::sys::ptrace::Register::R8);
    (6) => ($crate::sys::ptrace::Register::R9);
}

/// Returns the register containing nth register argument for 32-bit programs
///
/// 0th argument is considered to be the syscall number.
/// Please note that these mappings are only valid for 32-bit programs.
/// Use [`syscall_arg`] for tracing 64-bit programs instead.
///
/// [`syscall_arg`]: macro.syscall_arg.html
/// # Examples
///
/// ```
/// # #[macro_use] extern crate nix;
/// # fn main() {
/// assert_eq!(syscall_arg32!(1), nix::sys::ptrace::Register::RBX);
/// # }
#[cfg(target_arch = "x86_64")]
#[macro_export]
macro_rules! syscall_arg32 {
    (0) => ($crate::sys::ptrace::Register::ORIG_RAX);
    (1) => ($crate::sys::ptrace::Register::RBX);
    (2) => ($crate::sys::ptrace::Register::RCX);
    (3) => ($crate::sys::ptrace::Register::RDX);
    (4) => ($crate::sys::ptrace::Register::RSI);
    (5) => ($crate::sys::ptrace::Register::RDI);
    (6) => ($crate::sys::ptrace::Register::RBP);
}

/// Returns the register containing nth register argument.
///
/// 0th argument is considered to be the syscall number.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate nix;
/// # fn main() {
/// assert_eq!(syscall_arg!(1), nix::sys::ptrace::Register::RDI);
/// # }
#[cfg(target_arch = "x86")]
#[macro_export]
macro_rules! syscall_arg {
    (0) => ($crate::sys::ptrace::Register::ORIG_EAX);
    (1) => ($crate::sys::ptrace::Register::EBX);
    (2) => ($crate::sys::ptrace::Register::ECX);
    (3) => ($crate::sys::ptrace::Register::EDX);
    (4) => ($crate::sys::ptrace::Register::ESI);
    (5) => ($crate::sys::ptrace::Register::EDI);
    (6) => ($crate::sys::ptrace::Register::EBP);
}

/// An integer type, whose size equals a machine word
///
/// `ptrace` always returns a machine word. This type provides an abstraction
/// of the fact that on *nix systems, `c_long` is always a machine word,
/// so as to prevent the library from leaking C implementation-dependent types.
type Word = usize;

/// Peeks a user-accessible register, as with `ptrace(PTRACE_PEEKUSER, ...)`
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn peekuser(pid: Pid, reg: Register) -> Result<Word> {
    let reg_arg = (reg as i32) as *mut c_void;
    unsafe {
        ptrace_peek(Request::PTRACE_PEEKUSER, pid, reg_arg, ptr::null_mut()).map(|r| r as Word)
    }
}

/// Sets the value of a user-accessible register, as with `ptrace(PTRACE_POKEUSER, ...)`
///
/// # Safety
/// When incorrectly used, may change the registers to bad values,
/// causing e.g. memory being corrupted by a syscall, thus is marked unsafe
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub unsafe fn pokeuser(pid: Pid, reg: Register, val: Word) -> Result<()> {
    let reg_arg = (reg as u64) as *mut c_void;
    ptrace_other(Request::PTRACE_POKEUSER, pid, reg_arg, val as *mut c_void).map(|_| ()) // ignore the useless return value
}

/// Peeks the memory of a process, as with `ptrace(PTRACE_PEEKDATA, ...)`
///
/// A memory chunk of a size of a machine word is returned.
/// # Safety
/// This function allows for accessing arbitrary data in the traced process
/// and may crash the inferior if used incorrectly and is thus marked `unsafe`.
pub unsafe fn peekdata(pid: Pid, addr: usize) -> Result<Word> {
    ptrace_peek(
        Request::PTRACE_PEEKDATA,
        pid,
        addr as *mut c_void,
        ptr::null_mut(),
    ).map(|r| r as Word)
}

/// Modifies the memory of a process, as with `ptrace(PTRACE_POKEUSER, ...)`
///
/// A memory chunk of a size of a machine word is overwriten in the requested
/// place in the memory of a process.
///
/// # Safety
/// This function allows for accessing arbitrary data in the traced process
/// and may crash the inferior or introduce race conditions if used
/// incorrectly and is thus marked `unsafe`.
pub unsafe fn pokedata(pid: Pid, addr: usize, val: Word) -> Result<()> {
    ptrace_other(
        Request::PTRACE_POKEDATA,
        pid,
        addr as *mut c_void,
        val as *mut c_void,
    ).map(|_| ()) // ignore the useless return value
}

#[cfg(test)]
mod tests {
    use super::Word;
    use std::mem::size_of;
    use libc::c_long;

    #[test]
    fn test_types() {
        // c_long is implementation defined, so make sure
        // its width matches
        assert_eq!(size_of::<Word>(), size_of::<c_long>());
    }
}
