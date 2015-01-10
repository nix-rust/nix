// Portions of this file are Copyright 2014 The Rust Project Developers.
// See http://rust-lang.org/COPYRIGHT.

use libc;
use core::mem;
use errno::{SysError, SysResult};

pub use libc::consts::os::posix88::{
    SIGHUP,   // 1
    SIGINT,   // 2
    SIGQUIT,  // 3
    SIGILL,   // 4
    SIGABRT,  // 6
    SIGFPE,   // 8
    SIGKILL,  // 9
    SIGSEGV,  // 11
    SIGPIPE,  // 13
    SIGALRM,  // 14
    SIGTERM,  // 15
};

pub use self::signal::{
    SIGTRAP,
    SIGIOT,
    SIGBUS,
    SIGSYS,
    SIGURG,
    SIGSTOP,
    SIGTSTP,
    SIGCONT,
    SIGCHLD,
    SIGTTIN,
    SIGTTOU,
    SIGIO,
    SIGXCPU,
    SIGXFSZ,
    SIGVTALRM,
    SIGPROF,
    SIGWINCH,
    SIGUSR1,
    SIGUSR2,
};

pub use self::signal::SockFlag;
pub use self::signal::sigset_t;

// This doesn't always exist, but when it does, it's 7
pub const SIGEMT: libc::c_int = 7;

#[cfg(any(all(target_os = "linux",
              any(target_arch = "x86",
                  target_arch = "x86_64",
                  target_arch = "arm")),
          target_os = "android"))]
pub mod signal {
    use libc;

    bitflags!(
        flags SockFlag: libc::c_ulong {
            const SA_NOCLDSTOP = 0x00000001,
            const SA_NOCLDWAIT = 0x00000002,
            const SA_NODEFER   = 0x40000000,
            const SA_ONSTACK   = 0x08000000,
            const SA_RESETHAND = 0x80000000,
            const SA_RESTART   = 0x10000000,
            const SA_SIGINFO   = 0x00000004,
        }
    );

    pub const SIGTRAP:      libc::c_int = 5;
    pub const SIGIOT:       libc::c_int = 6;
    pub const SIGBUS:       libc::c_int = 7;
    pub const SIGUSR1:      libc::c_int = 10;
    pub const SIGUSR2:      libc::c_int = 12;
    pub const SIGSTKFLT:    libc::c_int = 16;
    pub const SIGCHLD:      libc::c_int = 17;
    pub const SIGCONT:      libc::c_int = 18;
    pub const SIGSTOP:      libc::c_int = 19;
    pub const SIGTSTP:      libc::c_int = 20;
    pub const SIGTTIN:      libc::c_int = 21;
    pub const SIGTTOU:      libc::c_int = 22;
    pub const SIGURG:       libc::c_int = 23;
    pub const SIGXCPU:      libc::c_int = 24;
    pub const SIGXFSZ:      libc::c_int = 25;
    pub const SIGVTALRM:    libc::c_int = 26;
    pub const SIGPROF:      libc::c_int = 27;
    pub const SIGWINCH:     libc::c_int = 28;
    pub const SIGIO:        libc::c_int = 29;
    pub const SIGPOLL:      libc::c_int = 29;
    pub const SIGPWR:       libc::c_int = 30;
    pub const SIGSYS:       libc::c_int = 31;
    pub const SIGUNUSED:    libc::c_int = 31;

    // This definition is not as accurate as it could be, {pid, uid, status} is
    // actually a giant union. Currently we're only interested in these fields,
    // however.
    #[repr(C)]
    #[derive(Copy)]
    pub struct siginfo {
        si_signo: libc::c_int,
        si_errno: libc::c_int,
        si_code: libc::c_int,
        pub pid: libc::pid_t,
        pub uid: libc::uid_t,
        pub status: libc::c_int,
    }

    #[repr(C)]
    #[allow(missing_copy_implementations)]
    pub struct sigaction {
        pub sa_handler: extern fn(libc::c_int),
        pub sa_mask: sigset_t,
        pub sa_flags: SockFlag,
        sa_restorer: *mut libc::c_void,
    }

    #[repr(C)]
    #[cfg(target_pointer_width = "32")]
    #[derive(Copy)]
    pub struct sigset_t {
        __val: [libc::c_ulong; 32],
    }

    #[repr(C)]
    #[cfg(target_pointer_width = "64")]
    #[derive(Copy)]
    pub struct sigset_t {
        __val: [libc::c_ulong; 16],
    }
}

#[cfg(all(target_os = "linux",
          any(target_arch = "mips", target_arch = "mipsel")))]
pub mod signal {
    use libc;

    bitflags!(
        flags SockFlag: libc::c_uint {
            const SA_NOCLDSTOP = 0x00000001,
            const SA_NOCLDWAIT = 0x00001000,
            const SA_NODEFER   = 0x40000000,
            const SA_ONSTACK   = 0x08000000,
            const SA_RESETHAND = 0x80000000,
            const SA_RESTART   = 0x10000000,
            const SA_SIGINFO   = 0x00000008,
        }
    );

    pub const SIGTRAP:      libc::c_int = 5;
    pub const SIGIOT:       libc::c_int = 6;
    pub const SIGBUS:       libc::c_int = 10;
    pub const SIGSYS:       libc::c_int = 12;
    pub const SIGUSR1:      libc::c_int = 16;
    pub const SIGUSR2:      libc::c_int = 17;
    pub const SIGCHLD:      libc::c_int = 18;
    pub const SIGCLD:       libc::c_int = 18;
    pub const SIGPWR:       libc::c_int = 19;
    pub const SIGWINCH:     libc::c_int = 20;
    pub const SIGURG:       libc::c_int = 21;
    pub const SIGIO:        libc::c_int = 22;
    pub const SIGPOLL:      libc::c_int = 22;
    pub const SIGSTOP:      libc::c_int = 23;
    pub const SIGTSTP:      libc::c_int = 24;
    pub const SIGCONT:      libc::c_int = 25;
    pub const SIGTTIN:      libc::c_int = 26;
    pub const SIGTTOU:      libc::c_int = 27;
    pub const SIGVTALRM:    libc::c_int = 28;
    pub const SIGPROF:      libc::c_int = 29;
    pub const SIGXCPU:      libc::c_int = 30;
    pub const SIGFSZ:       libc::c_int = 31;

    // This definition is not as accurate as it could be, {pid, uid, status} is
    // actually a giant union. Currently we're only interested in these fields,
    // however.
    #[repr(C)]
    pub struct siginfo {
        si_signo: libc::c_int,
        si_code: libc::c_int,
        si_errno: libc::c_int,
        pub pid: libc::pid_t,
        pub uid: libc::uid_t,
        pub status: libc::c_int,
    }

    #[repr(C)]
    pub struct sigaction {
        pub sa_flags: SockFlag,
        pub sa_handler: extern fn(libc::c_int),
        pub sa_mask: sigset_t,
        sa_restorer: *mut libc::c_void,
        sa_resv: [libc::c_int; 1],
    }

    #[repr(C)]
    pub struct sigset_t {
        __val: [libc::c_ulong; 32],
    }
}

#[cfg(any(target_os = "macos",
          target_os = "ios",
          target_os = "freebsd",
          target_os = "dragonfly"))]
pub mod signal {
    use libc;

    bitflags!(
        flags SockFlag: libc::c_int {
            const SA_NOCLDSTOP = 0x0008,
            const SA_NOCLDWAIT = 0x0020,
            const SA_NODEFER   = 0x0010,
            const SA_ONSTACK   = 0x0001,
            const SA_RESETHAND = 0x0004,
            const SA_RESTART   = 0x0002,
            const SA_SIGINFO   = 0x0040,
        }
    );

    pub const SIGTRAP:      libc::c_int = 5;
    pub const SIGIOT:       libc::c_int = 6;
    pub const SIGBUS:       libc::c_int = 10;
    pub const SIGSYS:       libc::c_int = 12;
    pub const SIGURG:       libc::c_int = 16;
    pub const SIGSTOP:      libc::c_int = 17;
    pub const SIGTSTP:      libc::c_int = 18;
    pub const SIGCONT:      libc::c_int = 19;
    pub const SIGCHLD:      libc::c_int = 20;
    pub const SIGTTIN:      libc::c_int = 21;
    pub const SIGTTOU:      libc::c_int = 22;
    pub const SIGIO:        libc::c_int = 23;
    pub const SIGXCPU:      libc::c_int = 24;
    pub const SIGXFSZ:      libc::c_int = 25;
    pub const SIGVTALRM:    libc::c_int = 26;
    pub const SIGPROF:      libc::c_int = 27;
    pub const SIGWINCH:     libc::c_int = 28;
    pub const SIGINFO:      libc::c_int = 29;
    pub const SIGUSR1:      libc::c_int = 30;
    pub const SIGUSR2:      libc::c_int = 31;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub type sigset_t = u32;
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    #[repr(C)]
    pub struct sigset_t {
        bits: [u32; 4],
    }

    // This structure has more fields, but we're not all that interested in
    // them.
    #[repr(C)]
    #[derive(Copy)]
    pub struct siginfo {
        pub si_signo: libc::c_int,
        pub si_errno: libc::c_int,
        pub si_code: libc::c_int,
        pub pid: libc::pid_t,
        pub uid: libc::uid_t,
        pub status: libc::c_int,
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[repr(C)]
    #[allow(missing_copy_implementations)]
    pub struct sigaction {
        pub sa_handler: extern fn(libc::c_int),
        sa_tramp: *mut libc::c_void,
        pub sa_mask: sigset_t,
        pub sa_flags: SockFlag,
    }

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    #[repr(C)]
    pub struct sigaction {
        pub sa_handler: extern fn(libc::c_int),
        pub sa_flags: SockFlag,
        pub sa_mask: sigset_t,
    }

}

mod ffi {
    use libc;
    use super::signal::{sigaction, sigset_t};

    #[allow(improper_ctypes)]
    extern {
        pub fn sigaction(signum: libc::c_int,
                         act: *const sigaction,
                         oldact: *mut sigaction) -> libc::c_int;

        pub fn sigaddset(set: *mut sigset_t, signum: libc::c_int) -> libc::c_int;
        pub fn sigdelset(set: *mut sigset_t, signum: libc::c_int) -> libc::c_int;
        pub fn sigemptyset(set: *mut sigset_t) -> libc::c_int;

        pub fn kill(pid: libc::pid_t, signum: libc::c_int) -> libc::c_int;
    }
}

#[derive(Copy)]
pub struct SigSet {
    sigset: sigset_t
}

pub type SigNum = libc::c_int;

impl SigSet {
    pub fn empty() -> SigSet {
        let mut sigset = unsafe { mem::uninitialized::<sigset_t>() };
        let _ = unsafe { ffi::sigemptyset(&mut sigset as *mut sigset_t) };

        SigSet { sigset: sigset }
    }

    pub fn add(&mut self, signum: SigNum) -> SysResult<()> {
        let res = unsafe { ffi::sigaddset(&mut self.sigset as *mut sigset_t, signum) };

        if res < 0 {
            return Err(SysError::last());
        }

        Ok(())
    }

    pub fn remove(&mut self, signum: SigNum) -> SysResult<()> {
        let res = unsafe { ffi::sigdelset(&mut self.sigset as *mut sigset_t, signum) };

        if res < 0 {
            return Err(SysError::last());
        }

        Ok(())
    }
}

type sigaction_t = self::signal::sigaction;

pub struct SigAction {
    sigaction: sigaction_t
}

impl SigAction {
    pub fn new(handler: extern fn(libc::c_int), flags: SockFlag, mask: SigSet) -> SigAction {
        let mut s = unsafe { mem::uninitialized::<sigaction_t>() };
        s.sa_handler = handler;
        s.sa_flags = flags;
        s.sa_mask = mask.sigset;

        SigAction { sigaction: s }
    }
}

pub fn sigaction(signum: SigNum, sigaction: &SigAction) -> SysResult<SigAction> {
    let mut oldact = unsafe { mem::uninitialized::<sigaction_t>() };

    let res = unsafe {
        ffi::sigaction(signum, &sigaction.sigaction as *const sigaction_t, &mut oldact as *mut sigaction_t)
    };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(SigAction { sigaction: oldact })
}

pub fn kill(pid: libc::pid_t, signum: SigNum) -> SysResult<()> {
    let res = unsafe { ffi::kill(pid, signum) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(())
}
