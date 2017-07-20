use nix::Error;
use nix::errno::*;
use nix::unistd::*;
use nix::sys::ptrace;

use std::{mem, ptr};

#[test]
fn test_ptrace() {
    use nix::sys::ptrace::ptrace::*;
    // Just make sure ptrace can be called at all, for now.
    // FIXME: qemu-user doesn't implement ptrace on all arches, so permit ENOSYS
    let err = ptrace::ptrace(PTRACE_ATTACH, getpid(), ptr::null_mut(), ptr::null_mut()).unwrap_err();
    assert!(err == Error::Sys(Errno::EPERM) || err == Error::Sys(Errno::ENOSYS));
}

// Just make sure ptrace_setoptions can be called at all, for now.
#[test]
fn test_ptrace_setoptions() {
    use nix::sys::ptrace::ptrace::*; // for PTRACE_O_TRACESYSGOOD
    let err = ptrace::setoptions(getpid(), PTRACE_O_TRACESYSGOOD).unwrap_err();
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
    match ptrace::getsiginfo(getpid()) {
        Err(Error::UnsupportedOperation) => panic!("ptrace_getsiginfo returns Error::UnsupportedOperation!"),
        _ => (),
    }
}

// Just make sure ptrace_setsiginfo can be called at all, for now.
#[test]
fn test_ptrace_setsiginfo() {
    let siginfo = unsafe { mem::uninitialized() };
    match ptrace::setsiginfo(getpid(), &siginfo) {
        Err(Error::UnsupportedOperation) => panic!("ptrace_setsiginfo returns Error::UnsupportedOperation!"),
        _ => (),
    }
}
