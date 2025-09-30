//! Traces a child process using `ptrace`.
//!
//! The child issues a single `write` syscall, which is printed upon entry and exit.

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn main() {
    let pid = unsafe { nix::unistd::fork().unwrap() };

    match pid {
        nix::unistd::ForkResult::Child => {
            nix::sys::ptrace::traceme().unwrap();
            nix::sys::signal::raise(nix::sys::signal::Signal::SIGCONT).unwrap();
            println!("I'm issuing a syscall!");
        }
        nix::unistd::ForkResult::Parent { child } => {
            nix::sys::wait::waitpid(Some(child), None).unwrap();
            nix::sys::ptrace::setoptions(
                child,
                nix::sys::ptrace::Options::PTRACE_O_TRACESYSGOOD,
            )
            .unwrap();

            nix::sys::ptrace::syscall(child, None).unwrap();
            nix::sys::wait::waitpid(Some(child), None).unwrap();
            let syscall_info = nix::sys::ptrace::syscall_info(child).unwrap();
            println!("{syscall_info:?}");
            assert!(syscall_info.op == libc::PTRACE_SYSCALL_INFO_ENTRY);

            nix::sys::ptrace::syscall(child, None).unwrap();
            nix::sys::wait::waitpid(Some(child), None).unwrap();
            let syscall_info = nix::sys::ptrace::syscall_info(child).unwrap();
            println!("{syscall_info:?}");
            assert!(syscall_info.op == libc::PTRACE_SYSCALL_INFO_EXIT);
        }
    }
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
fn main() {}
