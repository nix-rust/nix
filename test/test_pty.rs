use std::path::Path;
use std::os::unix::prelude::*;

use nix::fcntl::{O_RDWR, open};
use nix::pty::*;
use nix::sys::stat;
use nix::sys::termios::*;
use nix::unistd::{write, close};

/// Test equivalence of `ptsname` and `ptsname_r`
#[test]
#[cfg(any(target_os = "android", target_os = "linux"))]
fn test_ptsname_equivalence() {
    // Open a new PTTY master
    let master_fd = posix_openpt(O_RDWR).unwrap();
    assert!(master_fd.as_raw_fd() > 0);

    // Get the name of the slave
    let slave_name = ptsname(&master_fd).unwrap();
    let slave_name_r = ptsname_r(&master_fd).unwrap();
    assert_eq!(slave_name, slave_name_r);
}

/// Test data copying of `ptsname`
// TODO need to run in a subprocess, since ptsname is non-reentrant
#[test]
#[cfg(any(target_os = "android", target_os = "linux"))]
fn test_ptsname_copy() {
    // Open a new PTTY master
    let master_fd = posix_openpt(O_RDWR).unwrap();
    assert!(master_fd.as_raw_fd() > 0);

    // Get the name of the slave
    let slave_name1 = ptsname(&master_fd).unwrap();
    let slave_name2 = ptsname(&master_fd).unwrap();
    assert!(slave_name1 == slave_name2);
    // Also make sure that the string was actually copied and they point to different parts of
    // memory.
    assert!(slave_name1.as_ptr() != slave_name2.as_ptr());
}

/// Test data copying of `ptsname_r`
#[test]
#[cfg(any(target_os = "android", target_os = "linux"))]
fn test_ptsname_r_copy() {
    // Open a new PTTY master
    let master_fd = posix_openpt(O_RDWR).unwrap();
    assert!(master_fd.as_raw_fd() > 0);

    // Get the name of the slave
    let slave_name1 = ptsname_r(&master_fd).unwrap();
    let slave_name2 = ptsname_r(&master_fd).unwrap();
    assert!(slave_name1 == slave_name2);
    assert!(slave_name1.as_ptr() != slave_name2.as_ptr());
}

/// Test that `ptsname` returns different names for different devices
#[test]
#[cfg(any(target_os = "android", target_os = "linux"))]
fn test_ptsname_unique() {
    // Open a new PTTY master
    let master1_fd = posix_openpt(O_RDWR).unwrap();
    assert!(master1_fd.as_raw_fd() > 0);

    // Open a second PTTY master
    let master2_fd = posix_openpt(O_RDWR).unwrap();
    assert!(master2_fd.as_raw_fd() > 0);

    // Get the name of the slave
    let slave_name1 = ptsname(&master1_fd).unwrap();
    let slave_name2 = ptsname(&master2_fd).unwrap();
    assert!(slave_name1 != slave_name2);
}

/// Test opening a master/slave PTTY pair
///
/// This is a single larger test because much of these functions aren't useful by themselves. So for
/// this test we perform the basic act of getting a file handle for a connect master/slave PTTY
/// pair.
#[test]
fn test_open_ptty_pair() {
    // Open a new PTTY master
    let master_fd = posix_openpt(O_RDWR).unwrap();
    assert!(master_fd.as_raw_fd() > 0);

    // Allow a slave to be generated for it
    grantpt(&master_fd).unwrap();
    unlockpt(&master_fd).unwrap();

    // Get the name of the slave
    let slave_name = ptsname(&master_fd).unwrap();

    // Open the slave device
    let slave_fd = open(Path::new(&slave_name), O_RDWR, stat::Mode::empty()).unwrap();
    assert!(slave_fd > 0);
}

#[test]
fn test_openpty() {
    let pty = openpty(None, None).unwrap();
    assert!(pty.master > 0);
    assert!(pty.slave > 0);

    // Writing to one should be readable on the other one
    let string = "foofoofoo\n";
    let mut buf = [0u8; 10];
    write(pty.master, string.as_bytes()).unwrap();
    ::read_exact(pty.slave, &mut buf);

    assert_eq!(&buf, string.as_bytes());

    // Read the echo as well
    let echoed_string = "foofoofoo\r\n";
    let mut buf = [0u8; 11];
    ::read_exact(pty.master, &mut buf);
    assert_eq!(&buf, echoed_string.as_bytes());

    let string2 = "barbarbarbar\n";
    let echoed_string2 = "barbarbarbar\r\n";
    let mut buf = [0u8; 14];
    write(pty.slave, string2.as_bytes()).unwrap();
    ::read_exact(pty.master, &mut buf);

    assert_eq!(&buf, echoed_string2.as_bytes());

    close(pty.master).unwrap();
    close(pty.slave).unwrap();
}

#[test]
fn test_openpty_with_termios() {
    // Open one pty to get attributes for the second one
    let mut termios = {
        let pty = openpty(None, None).unwrap();
        assert!(pty.master > 0);
        assert!(pty.slave > 0);
        let termios = tcgetattr(pty.master).unwrap();
        close(pty.master).unwrap();
        close(pty.slave).unwrap();
        termios
    };
    // Make sure newlines are not transformed so the data is preserved when sent.
    termios.output_flags.remove(ONLCR);

    let pty = openpty(None, &termios).unwrap();
    // Must be valid file descriptors
    assert!(pty.master > 0);
    assert!(pty.slave > 0);

    // Writing to one should be readable on the other one
    let string = "foofoofoo\n";
    let mut buf = [0u8; 10];
    write(pty.master, string.as_bytes()).unwrap();
    ::read_exact(pty.slave, &mut buf);

    assert_eq!(&buf, string.as_bytes());

    // read the echo as well
    let echoed_string = "foofoofoo\n";
    ::read_exact(pty.master, &mut buf);
    assert_eq!(&buf, echoed_string.as_bytes());

    let string2 = "barbarbarbar\n";
    let echoed_string2 = "barbarbarbar\n";
    let mut buf = [0u8; 13];
    write(pty.slave, string2.as_bytes()).unwrap();
    ::read_exact(pty.master, &mut buf);

    assert_eq!(&buf, echoed_string2.as_bytes());

    close(pty.master).unwrap();
    close(pty.slave).unwrap();
}
