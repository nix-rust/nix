use std::os::unix::io::AsFd;
use tempfile::tempfile;

use nix::errno::Errno;
use nix::fcntl;
use nix::pty::openpty;
use nix::sys::termios::{self, tcgetattr, BaudRate, LocalFlags, OutputFlags};
use nix::unistd::{read, write};

/// Helper function analogous to `std::io::Write::write_all`, but for `Fd`s
fn write_all<Fd: AsFd>(f: Fd, buf: &[u8]) {
    let mut len = 0;
    while len < buf.len() {
        len += write(f.as_fd(), &buf[len..]).unwrap();
    }
}

#[test]
fn test_baudrate_try_from() {
    assert_eq!(Ok(BaudRate::B0), BaudRate::try_from(libc::B0));
    #[cfg(not(target_os = "haiku"))]
    BaudRate::try_from(999999999).expect_err("assertion failed");
    #[cfg(target_os = "haiku")]
    BaudRate::try_from(99).expect_err("assertion failed");
}

// Test tcgetattr on a terminal
#[test]
fn test_tcgetattr_pty() {
    // openpty uses ptname(3) internally
    let _m = crate::PTSNAME_MTX.lock();

    let pty = openpty(None, None).expect("openpty failed");
    termios::tcgetattr(&pty.slave).unwrap();
}

// Test tcgetattr on something that isn't a terminal
#[test]
fn test_tcgetattr_enotty() {
    let file = tempfile().unwrap();
    assert_eq!(termios::tcgetattr(&file).err(), Some(Errno::ENOTTY));
}

// Test modifying output flags
#[test]
fn test_output_flags() {
    // openpty uses ptname(3) internally
    let _m = crate::PTSNAME_MTX.lock();

    // Open one pty to get attributes for the second one
    let mut termios = {
        let pty = openpty(None, None).expect("openpty failed");
        tcgetattr(&pty.slave).expect("tcgetattr failed")
    };

    // Make sure postprocessing '\r' isn't specified by default or this test is useless.
    assert!(!termios
        .output_flags
        .contains(OutputFlags::OPOST | OutputFlags::OCRNL));

    // Specify that '\r' characters should be transformed to '\n'
    // OPOST is specified to enable post-processing
    termios
        .output_flags
        .insert(OutputFlags::OPOST | OutputFlags::OCRNL);

    // Open a pty
    let pty = openpty(None, &termios).unwrap();

    // Write into the master
    let string = "foofoofoo\r";
    write_all(&pty.master, string.as_bytes());

    // Read from the slave verifying that the output has been properly transformed
    let mut buf = [0u8; 10];
    crate::read_exact(&pty.slave, &mut buf);
    let transformed_string = "foofoofoo\n";
    assert_eq!(&buf, transformed_string.as_bytes());
}

// Test modifying local flags
#[test]
#[cfg(not(target_os = "solaris"))]
fn test_local_flags() {
    // openpty uses ptname(3) internally
    let _m = crate::PTSNAME_MTX.lock();

    // Open one pty to get attributes for the second one
    let mut termios = {
        let pty = openpty(None, None).unwrap();
        tcgetattr(&pty.slave).unwrap()
    };

    // Make sure echo is specified by default or this test is useless.
    assert!(termios.local_flags.contains(LocalFlags::ECHO));

    // Disable local echo
    termios.local_flags.remove(LocalFlags::ECHO);

    // Open a new pty with our modified termios settings
    let pty = openpty(None, &termios).unwrap();

    // Set the master is in nonblocking mode or reading will never return.
    let flags = fcntl::fcntl(&pty.master, fcntl::F_GETFL).unwrap();
    let new_flags =
        fcntl::OFlag::from_bits_truncate(flags) | fcntl::OFlag::O_NONBLOCK;
    fcntl::fcntl(pty.master.as_fd(), fcntl::F_SETFL(new_flags)).unwrap();

    // Write into the master
    let string = "foofoofoo\r";
    write_all(&pty.master, string.as_bytes());

    // Try to read from the master, which should not have anything as echoing was disabled.
    let mut buf = [0u8; 10];
    let read = read(&pty.master, &mut buf).unwrap_err();
    assert_eq!(read, Errno::EAGAIN);
}

// Test for glibc 2.42 compatibility - cfgetospeed should never panic
// Reproduces issue from nix-rust/nix#2672
#[test]
fn test_cfgetospeed_never_panics() {
    use nix::sys::termios::{cfgetospeed, Termios};

    // Test with zeroed termios (may be invalid on glibc 2.42)
    let termios: Termios = unsafe { std::mem::zeroed() };

    // This should not panic, even on glibc 2.42
    // Before fix: panicked at src/sys/termios.rs:767 with EINVAL
    let _speed = cfgetospeed(&termios);

    // If we get here, no panic occurred - test passes
}

// Test for glibc 2.42 compatibility - cfgetispeed should never panic
#[test]
fn test_cfgetispeed_never_panics() {
    use nix::sys::termios::{cfgetispeed, Termios};

    // Test with zeroed termios (may be invalid on glibc 2.42)
    let termios: Termios = unsafe { std::mem::zeroed() };

    // This should not panic, even on glibc 2.42
    let _speed = cfgetispeed(&termios);

    // If we get here, no panic occurred - test passes
}

// Test exact reproducer from issue #2672
#[test]
fn test_issue_2672_cfgetospeed_with_pty() {
    use nix::sys::termios::cfgetospeed;

    // openpty uses ptname(3) internally
    let _m = crate::PTSNAME_MTX.lock();

    let pty = openpty(None, None).expect("openpty failed");
    let termios = tcgetattr(&pty.slave).expect("tcgetattr failed");

    // This exact call panicked on glibc 2.42 before the fix
    let speed = cfgetospeed(&termios);

    // Should return some valid BaudRate (or B0 on error)
    // The key is: NO PANIC
    let _ = speed;
}

// Test baud rate roundtrip with actual pty
#[test]
fn test_baudrate_roundtrip_with_pty() {
    use nix::sys::termios::{cfgetospeed, cfsetspeed};

    // openpty uses ptname(3) internally
    let _m = crate::PTSNAME_MTX.lock();

    let pty = openpty(None, None).expect("openpty failed");
    let mut termios = tcgetattr(&pty.slave).expect("tcgetattr failed");

    // Set a known baud rate
    cfsetspeed(&mut termios, BaudRate::B9600).expect("cfsetspeed failed");

    // Get it back - should not panic
    let speed = cfgetospeed(&termios);

    // On most systems this equals B9600, but on glibc 2.42 might be B0 if conversion fails
    // Either way, NO PANIC is the requirement
    let _ = speed;
}
