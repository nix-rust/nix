use {Error, Result};
use errno::Errno;
use libc::{pid_t, c_void};

#[cfg(all(target_os = "linux",
          any(target_arch = "x86",
              target_arch = "x86_64")),
          )]
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
}

mod ffi {
    use libc::{pid_t, c_int, c_long, c_void};

    extern {
        pub fn ptrace(request: c_int, pid: pid_t, addr: * const c_void, data: * const c_void) -> c_long;
    }
}

pub fn ptrace(request: ptrace::PtraceRequest, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> Result<i64> {
    use self::ptrace::*;

    match request {
        PTRACE_PEEKTEXT | PTRACE_PEEKDATA | PTRACE_PEEKUSER => ptrace_peek(request, pid, addr, data),
        _ => ptrace_other(request, pid, addr, data)
    }
}

fn ptrace_peek(request: ptrace::PtraceRequest, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> Result<i64> {
    let ret = unsafe {
        Errno::clear();
        ffi::ptrace(request, pid, addr, data)
    };
    if ret == -1 && Errno::last() != Errno::UnknownErrno {
        return Err(Error::Sys(Errno::last()));
    }
    Ok::<i64, Error>(ret)
}

fn ptrace_other(request: ptrace::PtraceRequest, pid: pid_t, addr: *mut c_void, data: *mut c_void) -> Result<i64> {
    match unsafe { ffi::ptrace(request, pid, addr, data) } {
        -1 => Err(Error::Sys(Errno::last())),
        _  => Ok(0)
    }
}
