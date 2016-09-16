// Portions of this file are Copyright 2014 The Rust Project Developers.
// See http://rust-lang.org/COPYRIGHT.

use libc;
use {Errno, Error, Result};
use std::mem;
use std::ptr;

// Currently there is only one definition of c_int in libc, as well as only one
// type for signal constants.
// We would prefer to use the libc::c_int alias in the repr attribute. Unfortunately
// this is not (yet) possible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum Signal {
    SIGHUP = libc::SIGHUP,
    SIGINT = libc::SIGINT,
    SIGQUIT = libc::SIGQUIT,
    SIGILL = libc::SIGILL,
    SIGTRAP = libc::SIGTRAP,
    SIGABRT = libc::SIGABRT,
    SIGBUS = libc::SIGBUS,
    SIGFPE = libc::SIGFPE,
    SIGKILL = libc::SIGKILL,
    SIGUSR1 = libc::SIGUSR1,
    SIGSEGV = libc::SIGSEGV,
    SIGUSR2 = libc::SIGUSR2,
    SIGPIPE = libc::SIGPIPE,
    SIGALRM = libc::SIGALRM,
    SIGTERM = libc::SIGTERM,
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "emscripten"))]
    SIGSTKFLT = libc::SIGSTKFLT,
    SIGCHLD = libc::SIGCHLD,
    SIGCONT = libc::SIGCONT,
    SIGSTOP = libc::SIGSTOP,
    SIGTSTP = libc::SIGTSTP,
    SIGTTIN = libc::SIGTTIN,
    SIGTTOU = libc::SIGTTOU,
    SIGURG = libc::SIGURG,
    SIGXCPU = libc::SIGXCPU,
    SIGXFSZ = libc::SIGXFSZ,
    SIGVTALRM = libc::SIGVTALRM,
    SIGPROF = libc::SIGPROF,
    SIGWINCH = libc::SIGWINCH,
    SIGIO = libc::SIGIO,
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "emscripten"))]
    SIGPWR = libc::SIGPWR,
    SIGSYS = libc::SIGSYS,
    #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "emscripten")))]
    SIGEMT = libc::SIGEMT,
    #[cfg(not(any(target_os = "linux", target_os = "android", target_os = "emscripten")))]
    SIGINFO = libc::SIGINFO,
}

pub use self::Signal::*;

#[cfg(any(target_os = "linux", target_os = "android", target_os = "emscripten"))]
const SIGNALS: [Signal; 31] = [
    SIGHUP,
    SIGINT,
    SIGQUIT,
    SIGILL,
    SIGTRAP,
    SIGABRT,
    SIGBUS,
    SIGFPE,
    SIGKILL,
    SIGUSR1,
    SIGSEGV,
    SIGUSR2,
    SIGPIPE,
    SIGALRM,
    SIGTERM,
    SIGSTKFLT,
    SIGCHLD,
    SIGCONT,
    SIGSTOP,
    SIGTSTP,
    SIGTTIN,
    SIGTTOU,
    SIGURG,
    SIGXCPU,
    SIGXFSZ,
    SIGVTALRM,
    SIGPROF,
    SIGWINCH,
    SIGIO,
    SIGPWR,
    SIGSYS];
#[cfg(not(any(target_os = "linux", target_os = "android", target_os = "emscripten")))]
const SIGNALS: [Signal; 31] = [
    SIGHUP,
    SIGINT,
    SIGQUIT,
    SIGILL,
    SIGTRAP,
    SIGABRT,
    SIGBUS,
    SIGFPE,
    SIGKILL,
    SIGUSR1,
    SIGSEGV,
    SIGUSR2,
    SIGPIPE,
    SIGALRM,
    SIGTERM,
    SIGCHLD,
    SIGCONT,
    SIGSTOP,
    SIGTSTP,
    SIGTTIN,
    SIGTTOU,
    SIGURG,
    SIGXCPU,
    SIGXFSZ,
    SIGVTALRM,
    SIGPROF,
    SIGWINCH,
    SIGIO,
    SIGSYS,
    SIGEMT,
    SIGINFO];

pub const NSIG: libc::c_int = 32;

pub struct SignalIterator {
    next: usize,
}

impl Iterator for SignalIterator {
    type Item = Signal;

    fn next(&mut self) -> Option<Signal> {
        if self.next < SIGNALS.len() {
            let next_signal = SIGNALS[self.next];
            self.next += 1;
            Some(next_signal)
        } else {
            None
        }
    }
}

impl Signal {
    pub fn iterator() -> SignalIterator {
        SignalIterator{next: 0}
    }

    // We do not implement the From trait, because it is supposed to be infallible.
    // With Rust RFC 1542 comes the appropriate trait TryFrom. Once it is
    // implemented, we'll replace this function.
    #[inline]
    pub fn from_c_int(signum: libc::c_int) -> Result<Signal> {
        match 0 < signum && signum < NSIG {
            true => Ok(unsafe { mem::transmute(signum) }),
            false => Err(Error::invalid_argument()),
        }
    }
}

pub const SIGIOT : Signal = SIGABRT;
pub const SIGPOLL : Signal = SIGIO;
pub const SIGUNUSED : Signal = SIGSYS;

bitflags!{
    flags SaFlags: libc::c_int {
        const SA_NOCLDSTOP = libc::SA_NOCLDSTOP,
        const SA_NOCLDWAIT = libc::SA_NOCLDWAIT,
        const SA_NODEFER   = libc::SA_NODEFER,
        const SA_ONSTACK   = libc::SA_ONSTACK,
        const SA_RESETHAND = libc::SA_RESETHAND,
        const SA_RESTART   = libc::SA_RESTART,
        const SA_SIGINFO   = libc::SA_SIGINFO,
    }
}

bitflags!{
    flags SigFlags: libc::c_int {
        const SIG_BLOCK   = libc::SIG_BLOCK,
        const SIG_UNBLOCK = libc::SIG_UNBLOCK,
        const SIG_SETMASK = libc::SIG_SETMASK,
    }
}

#[derive(Clone, Copy)]
pub struct SigSet {
    sigset: libc::sigset_t
}


impl SigSet {
    pub fn all() -> SigSet {
        let mut sigset: libc::sigset_t = unsafe { mem::uninitialized() };
        let _ = unsafe { libc::sigfillset(&mut sigset as *mut libc::sigset_t) };

        SigSet { sigset: sigset }
    }

    pub fn empty() -> SigSet {
        let mut sigset: libc::sigset_t = unsafe { mem::uninitialized() };
        let _ = unsafe { libc::sigemptyset(&mut sigset as *mut libc::sigset_t) };

        SigSet { sigset: sigset }
    }

    pub fn add(&mut self, signal: Signal) {
        unsafe { libc::sigaddset(&mut self.sigset as *mut libc::sigset_t, signal as libc::c_int) };
    }

    pub fn clear(&mut self) {
        unsafe { libc::sigemptyset(&mut self.sigset as *mut libc::sigset_t) };
    }

    pub fn remove(&mut self, signal: Signal) {
        unsafe { libc::sigdelset(&mut self.sigset as *mut libc::sigset_t, signal as libc::c_int) };
    }

    pub fn contains(&self, signal: Signal) -> bool {
        let res = unsafe { libc::sigismember(&self.sigset as *const libc::sigset_t, signal as libc::c_int) };

        match res {
            1 => true,
            0 => false,
            _ => unreachable!("unexpected value from sigismember"),
        }
    }

    pub fn extend(&mut self, other: &SigSet) {
        for signal in Signal::iterator() {
            if other.contains(signal) {
                self.add(signal);
            }
        }
    }

    /// Gets the currently blocked (masked) set of signals for the calling thread.
    pub fn thread_get_mask() -> Result<SigSet> {
        let mut oldmask: SigSet = unsafe { mem::uninitialized() };
        try!(pthread_sigmask(SigFlags::empty(), None, Some(&mut oldmask)));
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
    pub fn thread_swap_mask(&self, how: SigFlags) -> Result<SigSet> {
        let mut oldmask: SigSet = unsafe { mem::uninitialized() };
        try!(pthread_sigmask(how, Some(self), Some(&mut oldmask)));
        Ok(oldmask)
    }

    /// Suspends execution of the calling thread until one of the signals in the
    /// signal mask becomes pending, and returns the accepted signal.
    pub fn wait(&self) -> Result<Signal> {
        let mut signum: libc::c_int = unsafe { mem::uninitialized() };
        let res = unsafe { libc::sigwait(&self.sigset as *const libc::sigset_t, &mut signum) };

        Errno::result(res).map(|_| Signal::from_c_int(signum).unwrap())
    }
}

impl AsRef<libc::sigset_t> for SigSet {
    fn as_ref(&self) -> &libc::sigset_t {
        &self.sigset
    }
}

#[allow(unknown_lints)]
#[cfg_attr(not(raw_pointer_derive_allowed), allow(raw_pointer_derive))]
#[derive(Clone, Copy, PartialEq)]
pub enum SigHandler {
    SigDfl,
    SigIgn,
    Handler(extern fn(libc::c_int)),
    SigAction(extern fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void))
}

pub struct SigAction {
    sigaction: libc::sigaction
}

impl SigAction {
    /// This function will set or unset the flag `SA_SIGINFO` depending on the
    /// type of the `handler` argument.
    pub fn new(handler: SigHandler, flags: SaFlags, mask: SigSet) -> SigAction {
        let mut s = unsafe { mem::uninitialized::<libc::sigaction>() };
        s.sa_sigaction = match handler {
            SigHandler::SigDfl => unsafe { mem::transmute(libc::SIG_DFL) },
            SigHandler::SigIgn => unsafe { mem::transmute(libc::SIG_IGN) },
            SigHandler::Handler(f) => unsafe { mem::transmute(f) },
            SigHandler::SigAction(f) => unsafe { mem::transmute(f) },
        };
        s.sa_flags = match handler {
            SigHandler::SigAction(_) => (flags | SA_SIGINFO).bits(),
            _ => (flags - SA_SIGINFO).bits(),
        };
        s.sa_mask = mask.sigset;

        SigAction { sigaction: s }
    }
}

pub unsafe fn sigaction(signal: Signal, sigaction: &SigAction) -> Result<SigAction> {
    let mut oldact = mem::uninitialized::<libc::sigaction>();

    let res =
        libc::sigaction(signal as libc::c_int, &sigaction.sigaction as *const libc::sigaction, &mut oldact as *mut libc::sigaction);

    Errno::result(res).map(|_| SigAction { sigaction: oldact })
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
pub fn pthread_sigmask(how: SigFlags,
                       set: Option<&SigSet>,
                       oldset: Option<&mut SigSet>) -> Result<()> {
    if set.is_none() && oldset.is_none() {
        return Ok(())
    }

    let res = unsafe {
        // if set or oldset is None, pass in null pointers instead
        libc::pthread_sigmask(how.bits(),
                             set.map_or_else(|| ptr::null::<libc::sigset_t>(),
                                             |s| &s.sigset as *const libc::sigset_t),
                             oldset.map_or_else(|| ptr::null_mut::<libc::sigset_t>(),
                                                |os| &mut os.sigset as *mut libc::sigset_t))
    };

    Errno::result(res).map(drop)
}

pub fn kill(pid: libc::pid_t, signal: Signal) -> Result<()> {
    let res = unsafe { libc::kill(pid, signal as libc::c_int) };

    Errno::result(res).map(drop)
}

pub fn raise(signal: Signal) -> Result<()> {
    let res = unsafe { libc::raise(signal as libc::c_int) };

    Errno::result(res).map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let mut mask = SigSet::empty();
        mask.add(SIGUSR1);

        assert!(mask.contains(SIGUSR1));
        assert!(!mask.contains(SIGUSR2));

        let all = SigSet::all();
        assert!(all.contains(SIGUSR1));
        assert!(all.contains(SIGUSR2));
    }

    #[test]
    fn test_clear() {
        let mut set = SigSet::all();
        set.clear();
        for signal in Signal::iterator() {
            assert!(!set.contains(signal));
        }
    }

    #[test]
    fn test_extend() {
        let mut one_signal = SigSet::empty();
        one_signal.add(SIGUSR1);

        let mut two_signals = SigSet::empty();
        two_signals.add(SIGUSR2);
        two_signals.extend(&one_signal);

        assert!(two_signals.contains(SIGUSR1));
        assert!(two_signals.contains(SIGUSR2));
    }

    #[test]
    fn test_thread_signal_block() {
        let mut mask = SigSet::empty();
        mask.add(SIGUSR1);

        assert!(mask.thread_block().is_ok());
    }

    #[test]
    fn test_thread_signal_swap() {
        let mut mask = SigSet::empty();
        mask.add(SIGUSR1);
        mask.thread_block().unwrap();

        assert!(SigSet::thread_get_mask().unwrap().contains(SIGUSR1));

        let mask2 = SigSet::empty();
        mask.add(SIGUSR2);

        let oldmask = mask2.thread_swap_mask(SIG_SETMASK).unwrap();

        assert!(oldmask.contains(SIGUSR1));
        assert!(!oldmask.contains(SIGUSR2));
    }

    // TODO(#251): Re-enable after figuring out flakiness.
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    #[test]
    fn test_sigwait() {
        let mut mask = SigSet::empty();
        mask.add(SIGUSR1);
        mask.add(SIGUSR2);
        mask.thread_block().unwrap();

        raise(SIGUSR1).unwrap();
        assert_eq!(mask.wait().unwrap(), SIGUSR1);
    }
}
