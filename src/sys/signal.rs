// Portions of this file are Copyright 2014 The Rust Project Developers.
// See http://rust-lang.org/COPYRIGHT.

use libc;
use errno::Errno;
use std::mem;
use std::ptr;
use {Error, Result};

pub use libc::{
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
pub use self::signal::{HowFlag, SIG_BLOCK, SIG_UNBLOCK, SIG_SETMASK};
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

    bitflags!{
        flags HowFlag: libc::c_int {
            const SIG_BLOCK   = 0,
            const SIG_UNBLOCK = 1,
            const SIG_SETMASK = 2,
        }
    }

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
    #[derive(Clone, Copy)]
    pub struct siginfo {
        pub si_signo: libc::c_int,
        pub si_errno: libc::c_int,
        pub si_code: libc::c_int,
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
    #[derive(Clone, Copy)]
    pub struct sigset_t {
        __val: [libc::c_ulong; 32],
    }

    #[repr(C)]
    #[cfg(target_pointer_width = "64")]
    #[derive(Clone, Copy)]
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

    bitflags!{
        flags HowFlag: libc::c_int {
            const SIG_BLOCK   = 1,
            const SIG_UNBLOCK = 2,
            const SIG_SETMASK = 3,
        }
    }

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
        pub si_signo: libc::c_int,
        pub si_code: libc::c_int,
        pub si_errno: libc::c_int,
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
          target_os = "openbsd",
          target_os = "dragonfly",
          target_os = "netbsd"))]
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

    bitflags!{
        flags HowFlag: libc::c_int {
            const SIG_BLOCK   = 1,
            const SIG_UNBLOCK = 2,
            const SIG_SETMASK = 3,
        }
    }

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

    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "openbsd"))]
    pub type sigset_t = u32;
    #[cfg(target_os = "freebsd")]
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct sigset_t {
        bits: [u32; 4],
    }
    #[cfg(any(target_os = "dragonfly", target_os = "netbsd"))]
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct sigset_t {
        bits: [libc::c_uint; 4],
    }

    // This structure has more fields, but we're not all that interested in
    // them.
    #[cfg(not(target_os = "dragonfly"))]
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct siginfo {
        pub si_signo: libc::c_int,
        pub si_errno: libc::c_int,
        pub si_code: libc::c_int,
        pub pid: libc::pid_t,
        pub uid: libc::uid_t,
        pub status: libc::c_int,
    }

    #[cfg(target_os = "dragonfly")]
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct siginfo {
        pub si_signo: libc::c_int,
        pub si_errno: libc::c_int,
        pub si_code: libc::c_int,
        pub pid: libc::c_int,
        pub uid: libc::c_uint,
        pub status: libc::c_int,
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[repr(C)]
    #[allow(missing_copy_implementations)]
    pub struct sigaction {
        pub sa_handler: extern fn(libc::c_int),
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

    #[cfg(any(target_os = "openbsd", target_os = "netbsd"))]
    #[repr(C)]
    pub struct sigaction {
        pub sa_handler: extern fn(libc::c_int),
        pub sa_mask: sigset_t,
        pub sa_flags: SockFlag,
    }
}

mod ffi {
    use libc::{c_int, pid_t};
    use super::signal::{sigaction, sigset_t};

    #[allow(improper_ctypes)]
    extern {
        pub fn sigaction(signum: c_int,
                         act: *const sigaction,
                         oldact: *mut sigaction) -> c_int;

        pub fn sigaddset(set: *mut sigset_t, signum: c_int) -> c_int;
        pub fn sigdelset(set: *mut sigset_t, signum: c_int) -> c_int;
        pub fn sigemptyset(set: *mut sigset_t) -> c_int;
        pub fn sigfillset(set: *mut sigset_t) -> c_int;
        pub fn sigismember(set: *const sigset_t, signum: c_int) -> c_int;

        pub fn pthread_sigmask(how: c_int, set: *const sigset_t, oldset: *mut sigset_t) -> c_int;

        pub fn kill(pid: pid_t, signum: c_int) -> c_int;
    }
}

#[derive(Clone, Copy)]
pub struct SigSet {
    sigset: sigset_t
}

pub type SigNum = libc::c_int;

impl SigSet {
    pub fn all() -> SigSet {
        let mut sigset: sigset_t = unsafe { mem::uninitialized() };
        let _ = unsafe { ffi::sigfillset(&mut sigset as *mut sigset_t) };

        SigSet { sigset: sigset }
    }

    pub fn empty() -> SigSet {
        let mut sigset: sigset_t = unsafe { mem::uninitialized() };
        let _ = unsafe { ffi::sigemptyset(&mut sigset as *mut sigset_t) };

        SigSet { sigset: sigset }
    }

    pub fn add(&mut self, signum: SigNum) -> Result<()> {
        let res = unsafe { ffi::sigaddset(&mut self.sigset as *mut sigset_t, signum) };

        if res < 0 {
            return Err(Error::Sys(Errno::last()));
        }

        Ok(())
    }

    pub fn remove(&mut self, signum: SigNum) -> Result<()> {
        let res = unsafe { ffi::sigdelset(&mut self.sigset as *mut sigset_t, signum) };

        if res < 0 {
            return Err(Error::Sys(Errno::last()));
        }

        Ok(())
    }

    pub fn contains(&self, signum: SigNum) -> Result<bool> {
        let res = unsafe { ffi::sigismember(&self.sigset as *const sigset_t, signum) };

        match res {
            1 => Ok(true),
            0 => Ok(false),
            _ => Err(Error::Sys(Errno::last()))
        }
    }

    /// Gets the currently blocked (masked) set of signals for the calling thread.
    pub fn thread_get_mask() -> Result<SigSet> {
        let mut oldmask: SigSet = unsafe { mem::uninitialized() };
        try!(pthread_sigmask(HowFlag::empty(), None, Some(&mut oldmask)));
        Ok(oldmask)
    }

    /// Sets the set of signals as the signal mask for the calling thread.
    pub fn thread_set_mask(&self) -> Result<()> {
        pthread_sigmask(SIG_SETMASK, Some(self), None)
    }

    /// Adds the set of signals to the signal mask for the calling thread.
    pub fn thread_block(&self) -> Result<()> {
        pthread_sigmask(SIG_BLOCK, Some(self), None)
    }

    /// Removes the set of signals from the signal mask for the calling thread.
    pub fn thread_unblock(&self) -> Result<()> {
        pthread_sigmask(SIG_UNBLOCK, Some(self), None)
    }

    /// Sets the set of signals as the signal mask, and returns the old mask.
    pub fn thread_swap_mask(&self, how: HowFlag) -> Result<SigSet> {
        let mut oldmask: SigSet = unsafe { mem::uninitialized() };
        try!(pthread_sigmask(how, Some(self), Some(&mut oldmask)));
        Ok(oldmask)
    }
}

impl AsRef<sigset_t> for SigSet {
    fn as_ref(&self) -> &sigset_t {
        &self.sigset
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

pub unsafe fn sigaction(signum: SigNum, sigaction: &SigAction) -> Result<SigAction> {
    let mut oldact = mem::uninitialized::<sigaction_t>();

    let res =
        ffi::sigaction(signum, &sigaction.sigaction as *const sigaction_t, &mut oldact as *mut sigaction_t);

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(SigAction { sigaction: oldact })
}

/// Manages the signal mask (set of blocked signals) for the calling thread.
///
/// If the `set` parameter is `Some(..)`, then the signal mask will be updated with the signal set.
/// The `how` flag decides the type of update. If `set` is `None`, `how` will be ignored,
/// and no modification will take place.
///
/// If the 'oldset' parameter is `Some(..)` then the current signal mask will be written into it.
///
/// If both `set` and `oldset` is `Some(..)`, the current signal mask will be written into oldset,
/// and then it will be updated with `set`.
///
/// If both `set` and `oldset` is None, this function is a no-op.
///
/// For more information, visit the [pthread_sigmask](http://man7.org/linux/man-pages/man3/pthread_sigmask.3.html),
/// or [sigprocmask](http://man7.org/linux/man-pages/man2/sigprocmask.2.html) man pages.
pub fn pthread_sigmask(how: HowFlag,
                       set: Option<&SigSet>,
                       oldset: Option<&mut SigSet>) -> Result<()> {
    if set.is_none() && oldset.is_none() {
        return Ok(())
    }

    let res = unsafe {
        // if set or oldset is None, pass in null pointers instead
        ffi::pthread_sigmask(how.bits(),
                             set.map_or_else(|| ptr::null::<sigset_t>(),
                                             |s| &s.sigset as *const sigset_t),
                             oldset.map_or_else(|| ptr::null_mut::<sigset_t>(),
                                                |os| &mut os.sigset as *mut sigset_t))
    };

    if res != 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(())
}

pub fn kill(pid: libc::pid_t, signum: SigNum) -> Result<()> {
    let res = unsafe { ffi::kill(pid, signum) };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let mut mask = SigSet::empty();
        mask.add(signal::SIGUSR1).unwrap();

        assert_eq!(mask.contains(signal::SIGUSR1), Ok(true));
        assert_eq!(mask.contains(signal::SIGUSR2), Ok(false));

        let all = SigSet::all();
        assert_eq!(all.contains(signal::SIGUSR1), Ok(true));
        assert_eq!(all.contains(signal::SIGUSR2), Ok(true));
    }

    #[test]
    fn test_thread_signal_block() {
        let mut mask = SigSet::empty();
        mask.add(signal::SIGUSR1).unwrap();

        assert!(mask.thread_block().is_ok());
    }

    #[test]
    fn test_thread_signal_swap() {
        let mut mask = SigSet::empty();
        mask.add(signal::SIGUSR1).unwrap();
        mask.thread_block().unwrap();

        assert!(SigSet::thread_get_mask().unwrap().contains(SIGUSR1).unwrap());

        let mask2 = SigSet::empty();
        mask.add(signal::SIGUSR2).unwrap();

        let oldmask = mask2.thread_swap_mask(signal::SIG_SETMASK).unwrap();

        assert!(oldmask.contains(signal::SIGUSR1).unwrap());
        assert!(!oldmask.contains(signal::SIGUSR2).unwrap());
    }
}
