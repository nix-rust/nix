use nix::Error;
use nix::errno::Errno;
use nix::unistd::getpid;
use nix::sys::ptrace::{self, Options};

use std::mem;

#[test]
fn test_ptrace() {
    // Just make sure ptrace can be called at all, for now.
    // FIXME: qemu-user doesn't implement ptrace on all arches, so permit ENOSYS
    let err = ptrace::attach(getpid()).unwrap_err();
    assert!(err == Error::Sys(Errno::EPERM) || err == Error::Sys(Errno::ENOSYS));
}

// Just make sure ptrace_setoptions can be called at all, for now.
#[test]
fn test_ptrace_setoptions() {
    let err = ptrace::setoptions(getpid(), Options::PTRACE_O_TRACESYSGOOD).unwrap_err();
    assert!(err != Error::UnsupportedOperation);
}

// Just make sure ptrace_getevent can be called at all, for now.
#[test]
fn test_ptrace_getevent() {
    let err = ptrace::getevent(getpid()).unwrap_err();
    assert!(err != Error::UnsupportedOperation);
}

// Just make sure ptrace_getsiginfo can be called at all, for now.
#[test]
fn test_ptrace_getsiginfo() {
    if let Err(Error::UnsupportedOperation) = ptrace::getsiginfo(getpid()) {
        panic!("ptrace_getsiginfo returns Error::UnsupportedOperation!");
    }
}

// Just make sure ptrace_setsiginfo can be called at all, for now.
#[test]
fn test_ptrace_setsiginfo() {
    let siginfo = unsafe { mem::uninitialized() };
    if let Err(Error::UnsupportedOperation) = ptrace::setsiginfo(getpid(), &siginfo) {
        panic!("ptrace_setsiginfo returns Error::UnsupportedOperation!");
    }
}


#[test]
fn test_ptrace_cont() {
    use nix::sys::ptrace;
    use nix::sys::signal::{raise, Signal};
    use nix::sys::wait::{waitpid, WaitStatus};
    use nix::unistd::fork;
    use nix::unistd::ForkResult::*;

    // FIXME: qemu-user doesn't implement ptrace on all architectures
    // and retunrs ENOSYS in this case.
    // We (ab)use this behavior to detect the affected platforms
    // and skip the test then.
    // On valid platforms the ptrace call should return Errno::EPERM, this
    // is already tested by `test_ptrace`.
    let err = ptrace::attach(getpid()).unwrap_err();
    if err == Error::Sys(Errno::ENOSYS) {
        return;
    }

    match fork().expect("Error: Fork Failed") {
        Child => {
            ptrace::traceme().unwrap();
            // As recommended by ptrace(2), raise SIGTRAP to pause the child
            // until the parent is ready to continue
            loop {
                raise(Signal::SIGTRAP).unwrap();
            }

        },
        Parent { child } => {
            assert_eq!(waitpid(child, None), Ok(WaitStatus::Stopped(child, Signal::SIGTRAP)));
            ptrace::cont(child, None).unwrap();
            assert_eq!(waitpid(child, None), Ok(WaitStatus::Stopped(child, Signal::SIGTRAP)));
            ptrace::cont(child, Signal::SIGKILL).unwrap();
            match waitpid(child, None) {
                Ok(WaitStatus::Signaled(pid, Signal::SIGKILL, _)) if pid == child => {}
                _ => panic!("The process should have been killed"),
            }
        },
    }
}
