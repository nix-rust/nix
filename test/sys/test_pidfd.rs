use nix::poll::{poll, PollFd, PollFlags};
use nix::sys::pidfd::*;
use nix::sys::signal::*;
use nix::unistd::*;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::prelude::{AsRawFd, FromRawFd};

#[test]
fn test_pidfd_open() {
    let pidfd = pidfd_open(getpid(), PidFdOpenFlag::empty()).unwrap();
    close(pidfd).unwrap();
}

#[test]
fn test_pidfd_getfd() {
    let pidfd = pidfd_open(getpid(), PidFdOpenFlag::empty()).unwrap();

    let mut tempfile = tempfile::tempfile().unwrap();
    tempfile.write_all(b"hello").unwrap();
    tempfile.seek(SeekFrom::Start(0)).unwrap();

    let tempfile2 = pidfd_getfd(pidfd, tempfile.as_raw_fd()).unwrap();
    let mut tempfile2 = unsafe { std::fs::File::from_raw_fd(tempfile2) };

    // Drop the original file. Since `tempfile2` should hold the same file, it would not be deleted.
    drop(tempfile);
    let mut buf = String::new();
    tempfile2.read_to_string(&mut buf).unwrap();
    assert_eq!(buf, "hello");

    close(pidfd).unwrap();
}

#[test]
fn test_pidfd_poll_send_signal() {
    let me_pidfd = pidfd_open(getpid(), PidFdOpenFlag::empty()).unwrap();

    let child = match unsafe { fork() }.expect("Error: Fork Failed") {
        ForkResult::Child => {
            sleep(1);
            unsafe { libc::_exit(42) }
        }
        ForkResult::Parent { child } => child,
    };

    let child_pidfd = pidfd_open(child, PidFdOpenFlag::empty()).unwrap();
    let mut poll_fds = [
        PollFd::new(me_pidfd, PollFlags::POLLIN),
        PollFd::new(child_pidfd, PollFlags::POLLIN),
    ];

    // Timeout.
    assert_eq!(poll(&mut poll_fds, 100).unwrap(), 0);
    // Both parent and child are running.
    assert!(!poll_fds[0].revents().unwrap().contains(PollFlags::POLLIN));
    assert!(!poll_fds[1].revents().unwrap().contains(PollFlags::POLLIN));

    pidfd_send_signal(child_pidfd, Signal::SIGINT, None).unwrap();

    // Child pidfd is ready.
    assert_eq!(poll(&mut poll_fds, 100).unwrap(), 1);
    // Parent is still running.
    assert!(!poll_fds[0].revents().unwrap().contains(PollFlags::POLLIN));
    // Child is dead.
    assert!(poll_fds[1].revents().unwrap().contains(PollFlags::POLLIN));

    close(me_pidfd).unwrap();
    close(child_pidfd).unwrap();
}
