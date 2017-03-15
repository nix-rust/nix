use std::path::Path;
use std::os::unix::prelude::*;
use nix::fcntl::{O_RDWR, open};
use nix::pty::*;
use nix::sys::stat;

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
