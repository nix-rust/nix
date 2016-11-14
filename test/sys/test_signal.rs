use nix::unistd::*;
use nix::sys::signal::*;

#[test]
fn test_kill_none() {
    kill(getpid(), None).ok().expect("Should be able to send signal to myself.");
}
