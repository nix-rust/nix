use nix::{
    errno::Errno,
    sys::pidfd::{pidfd_open, PidfdFlags},
    unistd::getpid,
};

#[test]
fn test_pidfd_open() {
    match pidfd_open(getpid(), PidfdFlags::empty()) {
        Ok(_) => (),
        Err(Errno::ENOSYS) => (),
        Err(e) => panic!("{e}"),
    }
}

#[cfg(feature = "signal")]
mod send_signal {
    use nix::{
        errno::Errno,
        sys::{
            pidfd::{
                pidfd_open, pidfd_send_signal, PidfdFlags, PidfdSignalFlags,
            },
            signal::Signal,
            wait::{waitpid, WaitStatus},
        },
        unistd::{fork, getpid, ForkResult},
    };

    #[test]
    fn test_pidfd_send_signal() {
        // NOTE: This function MUST be async-signal-safe.
        fn child_process() -> ! {
            let pidfd = match pidfd_open(getpid(), PidfdFlags::empty()) {
                Ok(x) => x,
                Err(Errno::ENOSYS) => std::process::exit(2),
                Err(_) => std::process::exit(1),
            };

            // Code beyond this call should be unreachable.
            match pidfd_send_signal(
                &pidfd,
                Signal::SIGKILL,
                None,
                PidfdSignalFlags::empty(),
            ) {
                Ok(()) => (),
                Err(Errno::ENOSYS) => std::process::exit(2),
                Err(_) => std::process::exit(1),
            }

            std::process::exit(1)
        }

        // SAFETY: `child_process` is async-signal-safe.
        let child = unsafe {
            match fork().expect("should be able to fork") {
                ForkResult::Parent { child } => child,
                ForkResult::Child => child_process(),
            }
        };

        match waitpid(child, None) {
            // Kernel has PIDFD syscalls and they worked.
            Ok(WaitStatus::Signaled(x, Signal::SIGKILL, _)) if x == child => (),

            // Kernel does not have PIDFD syscalls.
            Ok(WaitStatus::Exited(x, 2)) if x == child => (),

            _ => panic!("unexpected waitpid result"),
        }
    }
}
