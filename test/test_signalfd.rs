extern crate nix;

#[cfg(feature = "signalfd")]

use nix::sys::signalfd::SignalFd;
use nix::sys::signal;
use nix::unistd;

#[cfg(feature = "signalfd")]
fn main() {
    print!("test test_signalfd ... ");

    let mut mask = signal::SigSet::empty();
    mask.add(signal::SIGUSR1).unwrap();
    mask.thread_block().unwrap();

    let mut fd = SignalFd::new(&mask).unwrap();

    let pid = unistd::getpid();
    signal::kill(pid, signal::SIGUSR1).unwrap();

    let res = fd.read_signal();

    assert_eq!(res.unwrap().unwrap().ssi_signo as i32, signal::SIGUSR1);
    println!("ok");
}

#[cfg(not(feature = "signalfd"))]
fn main() {}
