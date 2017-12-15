use nix::unistd::*;
use nix::sys::signal::*;

#[test]
fn test_kill_none() {
    kill(getpid(), None).expect("Should be able to send signal to myself.");
}
