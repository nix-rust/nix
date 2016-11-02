#[macro_use]
extern crate nix;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate rand;
extern crate tempdir;
extern crate tempfile;

extern crate nix_test as nixtest;

mod sys;
mod test_fcntl;
#[cfg(any(target_os = "linux"))]
mod test_mq;
mod test_net;
mod test_nix_path;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod test_poll;
mod test_pty;
#[cfg(any(target_os = "linux", target_os = "android"))]
mod test_sendfile;
mod test_stat;
mod test_unistd;

use nixtest::assert_size_of;

#[test]
pub fn test_size_of_long() {
    // This test is mostly here to ensure that 32bit CI is correctly
    // functioning
    assert_size_of::<usize>("long");
}
