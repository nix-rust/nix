use {Errno, Error, Result};
use libc::{pid_t, c_int, c_void, c_long};

#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod ptrace {
    use libc;

    #[repr(u32)]
    pub enum PtraceRequest {
        PTRACE_TRACEME     = libc::PTRACE_TRACEME,
        PTRACE_PEEKTEXT    = libc::PTRACE_PEEKTEXT,
        PTRACE_PEEKDATA    = libc::PTRACE_PEEKDATA,
        PTRACE_PEEKUSER    = libc::PTRACE_PEEKUSER,
        PTRACE_POKETEXT    = libc::PTRACE_POKETEXT,
        PTRACE_POKEDATA    = libc::PTRACE_POKEDATA,
        PTRACE_POKEUSER    = libc::PTRACE_POKEUSER,
        PTRACE_CONT        = libc::PTRACE_CONT,
        PTRACE_KILL        = libc::PTRACE_KILL,
        PTRACE_SINGLESTEP  = libc::PTRACE_SINGLESTEP,
        PTRACE_GETREGS     = libc::PTRACE_GETREGS,
        PTRACE_SETREGS     = libc::PTRACE_SETREGS,
        PTRACE_GETFPREGS   = libc::PTRACE_GETFPREGS,
        PTRACE_SETFPREGS   = libc::PTRACE_SETFPREGS,
        PTRACE_ATTACH      = libc::PTRACE_ATTACH,
        PTRACE_DETACH      = libc::PTRACE_DETACH,
        PTRACE_GETFPXREGS  = libc::PTRACE_GETFPXREGS,
        PTRACE_SETFPXREGS  = libc::PTRACE_SETFPXREGS,
        PTRACE_SYSCALL     = libc::PTRACE_SYSCALL,
        PTRACE_SETOPTIONS  = libc::PTRACE_SETOPTIONS,
        PTRACE_GETEVENTMSG = libc::PTRACE_GETEVENTMSG,
        PTRACE_GETSIGINFO  = libc::PTRACE_GETSIGINFO,
        PTRACE_SETSIGINFO  = libc::PTRACE_SETSIGINFO,
        PTRACE_GETREGSET   = libc::PTRACE_GETREGSET,
        PTRACE_SETREGSET   = libc::PTRACE_SETREGSET,
        PTRACE_SEIZE       = libc::PTRACE_SEIZE,
        PTRACE_INTERRUPT   = libc::PTRACE_INTERRUPT,
        PTRACE_LISTEN      = libc::PTRACE_LISTEN,
        PTRACE_PEEKSIGINFO = libc::PTRACE_PEEKSIGINFO,
    }

    // These aren't currently in libc.
    #[cfg(any(target_os = "linux", target_os = "android"))]
    #[derive(Eq, PartialEq, Clone, Copy, Debug)]
    #[repr(u32)]
    pub enum PtraceEvent {
        PTRACE_EVENT_FORK       = 1,
        PTRACE_EVENT_VFORK      = 2,
        PTRACE_EVENT_CLONE      = 3,
        PTRACE_EVENT_EXEC       = 4,
        PTRACE_EVENT_VFORK_DONE = 5,
        PTRACE_EVENT_EXIT       = 6,
        PTRACE_EVENT_SECCOMP    = 7,
        PTRACE_EVENT_STOP       = 128,
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    impl PtraceEvent {
        /// Creates a PtraceEvent from the extra bits of a wait status (status >> 16)
        #[inline]
        pub fn from_c_int(event: libc::c_uint) -> Option<PtraceEvent> {
            use std::mem;

            if (event >= PtraceEvent::PTRACE_EVENT_FORK as u32 &&
                event <= PtraceEvent::PTRACE_EVENT_SECCOMP as u32)
                || event == PtraceEvent::PTRACE_EVENT_STOP as u32 {
                Some(unsafe { mem::transmute(event) })
            } else {
                None
            }
        }
    }

    bitflags! {
        flags PtraceOptions: libc::c_uint {
            const PTRACE_O_TRACESYSGOOD   = libc::PTRACE_O_TRACESYSGOOD,
            const PTRACE_O_TRACEFORK      = libc::PTRACE_O_TRACEFORK,
            const PTRACE_O_TRACEVFORK     = libc::PTRACE_O_TRACEVFORK,
            const PTRACE_O_TRACECLONE     = libc::PTRACE_O_TRACECLONE,
            const PTRACE_O_TRACEEXEC      = libc::PTRACE_O_TRACEEXEC,
            const PTRACE_O_TRACEVFORKDONE = libc::PTRACE_O_TRACEVFORKDONE,
            const PTRACE_O_TRACEEXIT      = libc::PTRACE_O_TRACEEXIT,
            const PTRACE_O_TRACESECCOMP   = libc::PTRACE_O_TRACESECCOMP,
        }
    }
}

mod ffi {
    use libc::{pid_t, c_int, c_long, c_void};

    extern {
        pub fn ptrace(request: c_int, pid: pid_t, addr: * const c_void, data: * const c_void) -> c_long;
    }
}

pub fn ptrace(request: ptrace::PtraceRequest, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> Result<c_long> {
    use self::ptrace::*;

    match request {
        PtraceRequest::PTRACE_PEEKTEXT | PtraceRequest::PTRACE_PEEKDATA | PtraceRequest::PTRACE_PEEKUSER => ptrace_peek(request, pid, addr, data),
        _ => ptrace_other(request, pid, addr, data)
    }
}

fn ptrace_peek(request: ptrace::PtraceRequest, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> Result<c_long> {
    let ret = unsafe {
        Errno::clear();
        ffi::ptrace(request as c_int, pid, addr, data)
    };
    match Errno::result(ret) {
        Ok(..) | Err(Error::Sys(Errno::UnknownErrno)) => Ok(ret),
        err @ Err(..) => err,
    }
}

fn ptrace_other(request: ptrace::PtraceRequest, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> Result<c_long> {
    Errno::result(unsafe { ffi::ptrace(request as c_int, pid, addr, data) }).map(|_| 0)
}

/// Set options, as with `ptrace(PTRACE_SETOPTIONS,...)`.
pub fn ptrace_setoptions(pid: pid_t, options: ptrace::PtraceOptions) -> Result<()> {
    use self::ptrace::*;
    use std::ptr;

    ptrace(PtraceRequest::PTRACE_SETOPTIONS, pid, ptr::null_mut(), options.bits() as *mut c_void).map(drop)
}
