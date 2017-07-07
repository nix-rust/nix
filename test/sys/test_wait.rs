use nix::unistd::*;
use nix::unistd::ForkResult::*;
use nix::sys::signal::*;
use nix::sys::wait::*;
use libc::exit;

#[test]
fn test_wait_signal() {
    #[allow(unused_variables)]
    let m = ::FORK_MTX.lock().expect("Mutex got poisoned by another test");

    match fork() {
      Ok(Child) => pause().unwrap_or(()),
      Ok(Parent { child }) => {
          kill(child, Some(SIGKILL)).ok().expect("Error: Kill Failed");
          assert_eq!(waitpid(PidGroup::ProcessID(child), None), Ok(WaitStatus::Signaled(child, SIGKILL, false)));
      },
      // panic, fork should never fail unless there is a serious problem with the OS
      Err(_) => panic!("Error: Fork Failed")
    }
}

#[test]
fn test_wait_exit() {
    #[allow(unused_variables)]
    let m = ::FORK_MTX.lock().expect("Mutex got poisoned by another test");

    match fork() {
      Ok(Child) => unsafe { exit(12); },
      Ok(Parent { child }) => {
          assert_eq!(waitpid(PidGroup::ProcessID(child), None), Ok(WaitStatus::Exited(child, 12)));
      },
      // panic, fork should never fail unless there is a serious problem with the OS
      Err(_) => panic!("Error: Fork Failed")
    }
}
