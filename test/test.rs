#[macro_use]
extern crate cfg_if;
#[cfg_attr(not(any(target_os = "redox", target_os = "haiku")), macro_use)]
extern crate nix;

mod common;
mod sys;
#[cfg(not(target_os = "redox"))]
mod test_dir;
mod test_fcntl;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod test_kmod;
#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "fushsia",
    target_os = "linux",
    target_os = "netbsd"
))]
mod test_mq;
#[cfg(not(target_os = "redox"))]
mod test_net;
mod test_nix_path;
#[cfg(target_os = "freebsd")]
mod test_nmount;
mod test_poll;
#[cfg(not(any(
    target_os = "redox",
    target_os = "fuchsia",
    target_os = "haiku"
)))]
mod test_pty;
mod test_resource;
#[cfg(any(
    target_os = "android",
    target_os = "dragonfly",
    all(target_os = "freebsd", fbsd14),
    target_os = "linux"
))]
mod test_sched;
#[cfg(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos"
))]
mod test_sendfile;
mod test_stat;
mod test_time;
#[cfg(all(
    any(
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd"
    ),
    feature = "time",
    feature = "signal"
))]
mod test_timer;
mod test_unistd;

use nix::unistd::{chdir, getcwd, read};
use parking_lot::{Mutex, RwLock, RwLockWriteGuard};
use std::os::unix::io::{AsFd, AsRawFd};
use std::path::PathBuf;

/// Helper function analogous to `std::io::Read::read_exact`, but for `Fd`s
fn read_exact<Fd: AsFd>(f: Fd, buf: &mut [u8]) {
    let mut len = 0;
    while len < buf.len() {
        // get_mut would be better than split_at_mut, but it requires nightly
        let (_, remaining) = buf.split_at_mut(len);
        len += read(f.as_fd().as_raw_fd(), remaining).unwrap();
    }
}

/// Any test that creates child processes must grab this mutex, regardless
/// of what it does with those children.
pub static FORK_MTX: std::sync::Mutex<()> = std::sync::Mutex::new(());
/// Any test that changes the process's current working directory must grab
/// the RwLock exclusively.  Any process that cares about the current
/// working directory must grab it shared.
pub static CWD_LOCK: RwLock<()> = RwLock::new(());
/// Any test that changes the process's supplementary groups must grab this
/// mutex
pub static GROUPS_MTX: Mutex<()> = Mutex::new(());
/// Any tests that loads or unloads kernel modules must grab this mutex
pub static KMOD_MTX: Mutex<()> = Mutex::new(());
/// Any test that calls ptsname(3) must grab this mutex.
pub static PTSNAME_MTX: Mutex<()> = Mutex::new(());
/// Any test that alters signal handling must grab this mutex.
pub static SIGNAL_MTX: Mutex<()> = Mutex::new(());

/// RAII object that restores a test's original directory on drop
struct DirRestore<'a> {
    d: PathBuf,
    _g: RwLockWriteGuard<'a, ()>,
}

impl<'a> DirRestore<'a> {
    fn new() -> Self {
        let guard = crate::CWD_LOCK.write();
        DirRestore {
            _g: guard,
            d: getcwd().unwrap(),
        }
    }
}

impl<'a> Drop for DirRestore<'a> {
    fn drop(&mut self) {
        let r = chdir(&self.d);
        if std::thread::panicking() {
            r.unwrap();
        }
    }
}
