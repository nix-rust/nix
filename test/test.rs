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
#[cfg(target_os = "linux")]
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
use std::os::unix::io::RawFd;
use std::sync::Mutex;
use nix::unistd::read;

/// Helper function analogous to std::io::Read::read_exact, but for `RawFD`s
fn read_exact(f: RawFd, buf: &mut  [u8]) {
    let mut len = 0;
    while len < buf.len() {
        // get_mut would be better than split_at_mut, but it requires nightly
        let (_, remaining) = buf.split_at_mut(len);
        len += read(f, remaining).unwrap();
    }
}

lazy_static! {
    /// Any test that changes the process's current working directory must grab
    /// this mutex
    pub static ref CWD_MTX: Mutex<()> = Mutex::new(());
    /// Any test that creates child processes must grab this mutex, regardless
    /// of what it does with those children.
    pub static ref FORK_MTX: Mutex<()> = Mutex::new(());
    /// Any test that alters signal handling must grab this mutex.
    pub static ref SIGNAL_MTX: Mutex<()> = Mutex::new(());
}

#[test]
pub fn test_size_of_long() {
    // This test is mostly here to ensure that 32bit CI is correctly
    // functioning
    assert_size_of::<usize>("long");
}
