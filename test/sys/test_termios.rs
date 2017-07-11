use std::os::unix::prelude::*;

use nix::{Error, fcntl, unistd};
use nix::errno::Errno;
use nix::pty::openpty;
use nix::sys::termios::{self, ECHO, OPOST, OCRNL, Termios, tcgetattr};
use nix::unistd::{read, write, close};

/// Helper function analogous to std::io::Write::write_all, but for `RawFd`s
fn write_all(f: RawFd, buf: &[u8]) {
    let mut len = 0;
    while len < buf.len() {
        len += write(f, &buf[len..]).unwrap();
    }
}

#[test]
fn test_tcgetattr() {
    for fd in 0..5 {
        let termios = termios::tcgetattr(fd);
        match unistd::isatty(fd) {
            // If `fd` is a TTY, tcgetattr must succeed.
            Ok(true) => assert!(termios.is_ok()),
            // If it's an invalid file descriptor, tcgetattr should also return
            // the same error
            Err(Error::Sys(Errno::EBADF)) => {
                assert_eq!(termios.err(), Some(Error::Sys(Errno::EBADF)));
            },
            // Otherwise it should return any error
            _ => assert!(termios.is_err())
        }
    }
}

// Test modifying output flags
#[test]
fn test_output_flags() {
    // Open one pty to get attributes for the second one
    let mut termios = {
        let pty = openpty(None, None).unwrap();
        assert!(pty.master > 0);
        assert!(pty.slave > 0);
        let termios = tcgetattr(pty.master).unwrap();
        termios
    };

    // Make sure postprocessing '\r' isn't specified by default or this test is useless.
    assert!(!termios.output_flags.contains(OPOST | OCRNL));

    // Specify that '\r' characters should be transformed to '\n'
    // OPOST is specified to enable post-processing
    termios.output_flags.insert(OPOST | OCRNL);

    // Open a pty
    let pty = openpty(None, &termios).unwrap();
    assert!(pty.master > 0);
    assert!(pty.slave > 0);

    // Write into the master
    let string = "foofoofoo\r";
    write_all(pty.master, string.as_bytes());

    // Read from the slave verifying that the output has been properly transformed
    let mut buf = [0u8; 10];
    ::read_exact(pty.slave, &mut buf);
    let transformed_string = "foofoofoo\n";
    assert_eq!(&buf, transformed_string.as_bytes());
}

// Test modifying local flags
#[test]
fn test_local_flags() {
    // Open one pty to get attributes for the second one
    let mut termios = {
        let pty = openpty(None, None).unwrap();
        assert!(pty.master > 0);
        assert!(pty.slave > 0);
        let termios = tcgetattr(pty.master).unwrap();
        termios
    };

    // Make sure echo is specified by default or this test is useless.
    assert!(termios.local_flags.contains(ECHO));

    // Disable local echo
    termios.local_flags.remove(ECHO);

    // Open a new pty with our modified termios settings
    let pty = openpty(None, &termios).unwrap();
    assert!(pty.master > 0);
    assert!(pty.slave > 0);

    // Set the master is in nonblocking mode or reading will never return.
    let flags = fcntl::fcntl(pty.master, fcntl::F_GETFL).unwrap();
    let new_flags = fcntl::OFlag::from_bits(flags).unwrap() | fcntl::O_NONBLOCK;
    fcntl::fcntl(pty.master, fcntl::F_SETFL(new_flags)).unwrap();

    // Write into the master
    let string = "foofoofoo\r";
    write_all(pty.master, string.as_bytes());

    // Try to read from the master, which should not have anything as echoing was disabled.
    let mut buf = [0u8; 10];
    let read = read(pty.master, &mut buf).unwrap_err();
    assert_eq!(read, Error::Sys(Errno::EAGAIN));
}

#[test]
fn test_cfmakeraw() {
    let mut termios = unsafe { Termios::default_uninit() };
    termios::cfmakeraw(&mut termios);
}
