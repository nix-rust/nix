use nix::{
    sys::{
        pidfd::{pid_open, pidfd_send_signal},
        signal::Signal,
        signalfd::SigSet,
        wait::waitpid,
    },
    unistd::{fork, ForkResult},
};

#[test]
fn test_pidfd_send_signal() {
    match unsafe { fork().unwrap() } {
        ForkResult::Parent { child } => {
            // Send SIGUSR1
            let pid_fd = pid_open(child, false).unwrap();
            pidfd_send_signal(pid_fd, Signal::SIGUSR1, None).unwrap();
            // Wait for child to exit.
            waitpid(child, None).unwrap();
        }
        ForkResult::Child => {
            // Wait for SIGUSR1
            let mut mask = SigSet::empty();
            mask.add(Signal::SIGUSR1);
            assert_eq!(mask.wait().unwrap(), Signal::SIGUSR1);
        }
    }
}
