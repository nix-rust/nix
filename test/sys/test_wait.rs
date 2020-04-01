use nix::Error;
use nix::unistd::*;
use nix::unistd::ForkResult::*;
use nix::sys::signal::*;
use nix::sys::wait::*;
use libc::_exit;

#[test]
fn test_wait_signal() {
    let _ = ::FORK_MTX.lock().expect("Mutex got poisoned by another test");

    // Safe: The child only calls `pause` and/or `_exit`, which are async-signal-safe.
    match fork().expect("Error: Fork Failed") {
      Child => {
          pause();
          unsafe { _exit(123) }
      },
      Parent { child } => {
          kill(child, Some(SIGKILL)).expect("Error: Kill Failed");
          assert_eq!(waitpid(child, None), Ok(WaitStatus::Signaled(child, SIGKILL, false)));
      },
    }
}

#[test]
fn test_wait_exit() {
    let _m = ::FORK_MTX.lock().expect("Mutex got poisoned by another test");

    // Safe: Child only calls `_exit`, which is async-signal-safe.
    match fork().expect("Error: Fork Failed") {
      Child => unsafe { _exit(12); },
      Parent { child } => {
          assert_eq!(waitpid(child, None), Ok(WaitStatus::Exited(child, 12)));
      },
    }
}

#[test]
fn test_waitstatus_from_raw() {
    let pid = Pid::from_raw(1);
    assert_eq!(WaitStatus::from_raw(pid, 0x0002), Ok(WaitStatus::Signaled(pid, Signal::SIGINT, false)));
    assert_eq!(WaitStatus::from_raw(pid, 0x0200), Ok(WaitStatus::Exited(pid, 2)));
    assert_eq!(WaitStatus::from_raw(pid, 0x7f7f), Err(Error::invalid_argument()));
}

#[test]
fn test_waitstatus_pid() {
    let _m = ::FORK_MTX.lock().expect("Mutex got poisoned by another test");

    match fork().unwrap() {
        Child => unsafe { _exit(0) },
        Parent { child } => {
            let status = waitpid(child, None).unwrap();
            assert_eq!(status.pid(), Some(child));
        }
    }
}

#[test]
fn test_child_iterator() {
    let _m = ::FORK_MTX
        .lock()
        .expect("Mutex got poisoned by another test");

    let mut children = Vec::new();

    // create some child_processes and kill them immediatly so we can collect the exit codes
    for _ in 0..10 {
        // Safe: The child only calls `pause` and/or `_exit`, which are async-signal-safe.
        match fork().expect("Error: Fork Failed") {
            Child => {
                pause();
                unsafe { _exit(123) }
            }
            Parent { child } => {
                kill(child, Some(SIGKILL)).expect("Error: Kill Failed");
                children.push(child);
            }
        }
    }

    let start_time = std::time::Instant::now();

    let mut exited_children = Vec::new();
    
    // this runs until all children have been signaled. It might have to call the iterator a few times, because the events might
    // be delivered in chunks
    while exited_children.len() < children.len() {
        // this prevents the test from running for ever in case of a bug 
        assert!(
            start_time.elapsed() < std::time::Duration::from_secs(2), 
            "It takes the children way too long to exit, something is probably broken"
        );
        exited_children.extend(child_event_iter());
    }

    assert_eq!(
        exited_children.len(),
        children.len(),
        "There should be the same amount of exited children as created children"
    );

    for exit in &exited_children {
        let is_as_expected = if let Ok(WaitStatus::Signaled(pid, signal, dumped)) = exit {
            children.contains(&pid) && *signal == Signal::SIGKILL && !dumped
        }else{
            false
        };
        assert!(is_as_expected, "This exited child did not exit as expected: {:?}", exit);
    }
}


#[cfg(any(target_os = "linux", target_os = "android"))]
// FIXME: qemu-user doesn't implement ptrace on most arches
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod ptrace {
    use nix::sys::ptrace::{self, Options, Event};
    use nix::sys::signal::*;
    use nix::sys::wait::*;
    use nix::unistd::*;
    use nix::unistd::ForkResult::*;
    use libc::_exit;

    fn ptrace_child() -> ! {
        ptrace::traceme().unwrap();
        // As recommended by ptrace(2), raise SIGTRAP to pause the child
        // until the parent is ready to continue
        raise(SIGTRAP).unwrap();
        unsafe { _exit(0) }
    }

    fn ptrace_parent(child: Pid) {
        // Wait for the raised SIGTRAP
        assert_eq!(waitpid(child, None), Ok(WaitStatus::Stopped(child, SIGTRAP)));
        // We want to test a syscall stop and a PTRACE_EVENT stop
        assert!(ptrace::setoptions(child, Options::PTRACE_O_TRACESYSGOOD | Options::PTRACE_O_TRACEEXIT).is_ok());

        // First, stop on the next system call, which will be exit()
        assert!(ptrace::syscall(child, None).is_ok());
        assert_eq!(waitpid(child, None), Ok(WaitStatus::PtraceSyscall(child)));
        // Then get the ptrace event for the process exiting
        assert!(ptrace::cont(child, None).is_ok());
        assert_eq!(waitpid(child, None), Ok(WaitStatus::PtraceEvent(child, SIGTRAP, Event::PTRACE_EVENT_EXIT as i32)));
        // Finally get the normal wait() result, now that the process has exited
        assert!(ptrace::cont(child, None).is_ok());
        assert_eq!(waitpid(child, None), Ok(WaitStatus::Exited(child, 0)));
    }

    #[test]
    fn test_wait_ptrace() {
        require_capability!(CAP_SYS_PTRACE);
        let _m = ::FORK_MTX.lock().expect("Mutex got poisoned by another test");

        match fork().expect("Error: Fork Failed") {
            Child => ptrace_child(),
            Parent { child } => ptrace_parent(child),
        }
    }
}
