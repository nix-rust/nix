use nix::Error;
use nix::errno::Errno;
use nix::unistd::getpid;
use nix::sys::ptrace;
use nix::libc;

use std::{mem, ptr};

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
    let err = ptrace::setoptions(getpid(), ptrace::PTRACE_O_TRACESYSGOOD).unwrap_err();
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

#[test]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn test_ptrace_peekpoke() {
    use nix::sys::ptrace;
    use nix::sys::signal::{raise, Signal};
    use nix::sys::wait::{waitpid, WaitStatus};
    use nix::unistd::fork;
    use nix::unistd::ForkResult::*;

    let _m = ::FORK_MTX.lock().expect("Mutex got poisoned by another test");

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

    match fork() {
        Ok(Child) => {
            ptrace::traceme().unwrap();
            // As recommended by ptrace(2), raise SIGTRAP to pause the child
            // until the parent is ready to continue
            raise(Signal::SIGTRAP).unwrap();
            unsafe {
                let size = 10000;
                let ptr = libc::calloc(size, 1);
                libc::getcwd(ptr as *mut i8, size);
                libc::free(ptr);
                libc::getpriority(0, 42);
                libc::_exit(0);
            }
        }
        Ok(Parent { child }) => {
            assert_eq!(
                waitpid(child, None),
                Ok(WaitStatus::Stopped(child, Signal::SIGTRAP))
            );

            let mut syscall_no = None;
            let mut getpriority_checked = false;
            let mut getcwd_checked = false;

            ptrace::setoptions(child, ptrace::PTRACE_O_TRACESYSGOOD).unwrap();

            loop {
                ptrace::syscall(child).unwrap();
                match waitpid(child, None).unwrap() {
                    WaitStatus::PtraceSyscall(child) => {
                        match syscall_no {
                            None => {
                                let no = ptrace::peekuser(child, syscall_arg!(0)).unwrap();
                                syscall_no = Some(no);
                                if no as i64 == libc::SYS_getpriority as i64 {
                                    let arg2 = ptrace::peekuser(child, syscall_arg!(2)).unwrap();
                                    assert_eq!(arg2, 42);
                                    unsafe {
                                        ptrace::pokeuser(child, syscall_arg!(2), 0).unwrap();
                                    }
                                    let arg2 = ptrace::peekuser(child, syscall_arg!(2)).unwrap();
                                    assert_eq!(arg2, 0);

                                    getpriority_checked = true;
                                }
                            }
                            Some(no) => {
                                syscall_no = None;
                                if no as i64 == libc::SYS_getcwd as i64 {
                                    let ret = ptrace::peekuser(child, syscall_arg!(0)).unwrap();
                                    assert!(ret != 0); // no error occured
                                    let buf = ptrace::peekuser(child, syscall_arg!(1)).unwrap();
                                    unsafe {
                                        let word = ptrace::peekdata(child, buf).unwrap();
                                        assert!(word != 0); // something was written to the buffer
                                        ptrace::pokedata(child, buf, 0).unwrap();
                                        let new_word = ptrace::peekdata(child, buf).unwrap();
                                        assert_eq!(new_word, 0);
                                    }

                                    getcwd_checked = true;
                                }
                            }
                        }
                    }
                    WaitStatus::Exited(_, code) => {
                        assert_eq!(code, 0);
                        break;
                    }
                    _ => {}
                }
            }

            assert!(getpriority_checked);
            assert!(getcwd_checked);
        }
        Err(_) => panic!("Error: Fork Failed"),
    }
}

#[test]
fn test_ptrace_cont() {
    use nix::sys::ptrace;
    use nix::sys::signal::{raise, Signal};
    use nix::sys::wait::{waitpid, WaitStatus};
    use nix::unistd::fork;
    use nix::unistd::ForkResult::*;

    let _m = ::FORK_MTX.lock().expect("Mutex got poisoned by another test");

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

    match fork() {
        Ok(Child) => {
            ptrace::traceme().unwrap();
            // As recommended by ptrace(2), raise SIGTRAP to pause the child
            // until the parent is ready to continue
            loop {
                raise(Signal::SIGTRAP).unwrap();
            }

        },
        Ok(Parent { child }) => {
            assert_eq!(waitpid(child, None), Ok(WaitStatus::Stopped(child, Signal::SIGTRAP)));
            ptrace::cont(child, None).unwrap();
            assert_eq!(waitpid(child, None), Ok(WaitStatus::Stopped(child, Signal::SIGTRAP)));
            ptrace::cont(child, Signal::SIGKILL).unwrap();
            match waitpid(child, None) {
                Ok(WaitStatus::Signaled(pid, Signal::SIGKILL, _)) if pid == child => {}
                _ => panic!("The process should have been killed"),
            }
        },
        Err(_) => panic!("Error: Fork Failed")
    }
}
