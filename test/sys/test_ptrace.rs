use nix::Error;
use nix::errno::*;
use nix::unistd::*;
use nix::sys::ptrace::*;
use nix::sys::ptrace::ptrace::*;
use std::ptr;

#[test]
fn test_ptrace() {
    // Just make sure ptrace can be called at all, for now.
    // FIXME: qemu-user doesn't implement ptrace on all arches, so permit ENOSYS
    let err = ptrace(PTRACE_ATTACH, getpid(), ptr::null_mut(), ptr::null_mut()).unwrap_err();
    assert!(err == Error::Sys(Errno::EPERM) || err == Error::Sys(Errno::ENOSYS));
}
