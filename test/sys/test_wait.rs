use nix::unistd::*;
use nix::unistd::Fork::*;
use nix::sys::signal::*;
use nix::sys::wait::*;
use libc::exit;

#[test]
fn test_wait_signal() {
    match fork() {
      Ok(Child) => loop { /* Wait for signal */ },
      Ok(Parent(child_pid)) => {
          kill(child_pid, SIGKILL).ok().expect("Error: Kill Failed");
          assert_eq!(waitpid(child_pid, None), Ok(WaitStatus::Signaled(child_pid, SIGKILL, false)));
      },
      // panic, fork should never fail unless there is a serious problem with the OS
      Err(_) => panic!("Error: Fork Failed")
    }
}

#[test]
fn test_wait_exit() {
    match fork() {
      Ok(Child) => unsafe { exit(12); },
      Ok(Parent(child_pid)) => {
          assert_eq!(waitpid(child_pid, None), Ok(WaitStatus::Exited(child_pid, 12)));
      },
      // panic, fork should never fail unless there is a serious problem with the OS
      Err(_) => panic!("Error: Fork Failed")
    }
}
