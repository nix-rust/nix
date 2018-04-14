use nix::unistd::*;
use nix::sys::signal::*;

#[test]
fn test_kill_none() {
    kill(getpid(), None).expect("Should be able to send signal to myself.");
}

#[test]
fn test_old_sigaction_flags() {
    extern "C" fn handler(_: ::libc::c_int) {}
    let act = SigAction::new(
        SigHandler::Handler(handler),
        SaFlags::empty(),
        SigSet::empty(),
    );
    let oact = unsafe { sigaction(SIGINT, &act) }.unwrap();
    let _flags = oact.flags();
    let oact = unsafe { sigaction(SIGINT, &act) }.unwrap();
    let _flags = oact.flags();
}

#[test]
fn test_sigprocmask_noop() {
    sigprocmask(SigmaskHow::SIG_BLOCK, None, None)
        .expect("this should be an effective noop");
}

#[test]
fn test_sigprocmask() {
    #[allow(unused_variables)]
    let m = ::SIGNAL_MTX.lock().expect("Mutex got poisoned by another test");

    // This needs to be a signal that rust doesn't use in the test harness.
    const SIGNAL: Signal = Signal::SIGCHLD;

    let mut old_signal_set = SigSet::empty();
    sigprocmask(SigmaskHow::SIG_BLOCK, None, Some(&mut old_signal_set))
        .expect("expect to be able to retrieve old signals");

    // Make sure the old set doesn't contain the signal, otherwise the following
    // test don't make sense.
    assert_eq!(old_signal_set.contains(SIGNAL), false,
               "the {:?} signal is already blocked, please change to a \
                different one", SIGNAL);

    // Now block the signal.
    let mut signal_set = SigSet::empty();
    signal_set.add(SIGNAL);
    sigprocmask(SigmaskHow::SIG_BLOCK, Some(&signal_set), None)
        .expect("expect to be able to block signals");

    // And test it again, to make sure the change was effective.
    old_signal_set.clear();
    sigprocmask(SigmaskHow::SIG_BLOCK, None, Some(&mut old_signal_set))
        .expect("expect to be able to retrieve old signals");
    assert_eq!(old_signal_set.contains(SIGNAL), true,
               "expected the {:?} to be blocked", SIGNAL);

    // Reset the signal.
    sigprocmask(SigmaskHow::SIG_UNBLOCK, Some(&signal_set), None)
        .expect("expect to be able to block signals");
}
