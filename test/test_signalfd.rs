extern crate nix;

use nix::unistd;
use nix::sys::signalfd::*;

fn main() {
    let mut mask = SigSet::empty();
    mask.add(signal::SIGUSR1).unwrap();
    mask.thread_block().unwrap();

    let mut fd = SignalFd::new(&mask).unwrap();

    let pid = unistd::getpid();
    signal::kill(pid, signal::SIGUSR1).unwrap();

    let res = fd.read_signal();
    assert!(res.is_ok());

    let opt = res.ok().unwrap();
    assert!(opt.is_some());

    let info = opt.unwrap();
    assert_eq!(info.ssi_signo as i32, signal::SIGUSR1);
}
