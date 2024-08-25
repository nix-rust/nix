use std::ffi::CString;

use nix::spawn::{self, PosixSpawnAttr, PosixSpawnFileActions};
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

#[test]
fn spawn_true() {
    let bin = &CString::new("true").unwrap();
    let args = &[
        CString::new("true").unwrap(),
        CString::new("story").unwrap(),
    ];
    let vars: &[CString] = &[];
    let actions = PosixSpawnFileActions::init().unwrap();
    let attr = PosixSpawnAttr::init().unwrap();

    let pid = spawn::posix_spawnp(bin, &actions, &attr, args, vars).unwrap();

    let status = waitpid(pid, Some(WaitPidFlag::empty())).unwrap();

    match status {
        WaitStatus::Exited(wpid, ret) => {
            assert_eq!(pid, wpid);
            assert_eq!(ret, 0);
        }
        _ => {
            panic!("Invalid WaitStatus");
        }
    };
}

#[test]
fn spawn_sleep() {
    let bin = &CString::new("sleep").unwrap();
    let args = &[CString::new("sleep").unwrap(), CString::new("30").unwrap()];
    let vars: &[CString] = &[];
    let actions = PosixSpawnFileActions::init().unwrap();
    let attr = PosixSpawnAttr::init().unwrap();

    let pid = spawn::posix_spawnp(bin, &actions, &attr, args, vars).unwrap();

    let status =
        waitpid(pid, WaitPidFlag::from_bits(WaitPidFlag::WNOHANG.bits()))
            .unwrap();
    match status {
        WaitStatus::StillAlive => {}
        _ => {
            panic!("Invalid WaitStatus");
        }
    };

    signal::kill(pid, signal::SIGTERM).unwrap();

    let status = waitpid(pid, Some(WaitPidFlag::empty())).unwrap();
    match status {
        WaitStatus::Signaled(wpid, wsignal, _) => {
            assert_eq!(pid, wpid);
            assert_eq!(wsignal, signal::SIGTERM);
        }
        _ => {
            panic!("Invalid WaitStatus");
        }
    };
}
