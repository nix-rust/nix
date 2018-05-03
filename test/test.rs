extern crate bytes;
#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate nix;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate rand;
extern crate tempdir;
extern crate tempfile;

mod sys;
mod test_fcntl;
#[cfg(any(target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "fushsia",
          target_os = "linux",
          target_os = "netbsd"))]
mod test_mq;
mod test_net;
mod test_nix_path;
mod test_poll;
mod test_pty;
#[cfg(any(target_os = "android",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "linux",
          target_os = "macos"))]
mod test_sendfile;
mod test_stat;
mod test_unistd;

use std::os::unix::io::RawFd;
use std::sync::Mutex;
use nix::unistd::read;

/// Helper function analogous to `std::io::Read::read_exact`, but for `RawFD`s
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
    /// Any test that changes the process's supplementary groups must grab this
    /// mutex
    pub static ref GROUPS_MTX: Mutex<()> = Mutex::new(());
    /// Any test that creates child processes must grab this mutex, regardless
    /// of what it does with those children.
    pub static ref FORK_MTX: Mutex<()> = Mutex::new(());
    /// Any test that calls ptsname(3) must grab this mutex.
    pub static ref PTSNAME_MTX: Mutex<()> = Mutex::new(());
    /// Any test that alters signal handling must grab this mutex.
    pub static ref SIGNAL_MTX: Mutex<()> = Mutex::new(());
}
