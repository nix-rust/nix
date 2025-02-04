use super::FORK_MTX;
use nix::spawn::{self, PosixSpawnAttr, PosixSpawnFileActions};
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use std::ffi::CString;

/// Helper function to find a binary in the $PATH
fn which(exe_name: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(exe_name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

#[test]
fn spawn_true() {
    let _guard = FORK_MTX.lock();

    let bin = which("true").unwrap();
    let args = &[
        CString::new("true").unwrap(),
        CString::new("story").unwrap(),
    ];
    let vars: &[CString] = &[];
    let actions = PosixSpawnFileActions::init().unwrap();
    let attr = PosixSpawnAttr::init().unwrap();

    let pid =
        spawn::posix_spawn(bin.as_path(), &actions, &attr, args, vars).unwrap();

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
    let _guard = FORK_MTX.lock();

    let bin = which("sleep").unwrap();
    let args = &[CString::new("sleep").unwrap(), CString::new("30").unwrap()];
    let vars: &[CString] = &[];
    let actions = PosixSpawnFileActions::init().unwrap();
    let attr = PosixSpawnAttr::init().unwrap();

    let pid =
        spawn::posix_spawn(bin.as_path(), &actions, &attr, args, vars).unwrap();

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

#[test]
fn spawn_cmd_does_not_exist() {
    let _guard = FORK_MTX.lock();

    let args = &[CString::new("buzz").unwrap()];
    let envs: &[CString] = &[];
    let actions = PosixSpawnFileActions::init().unwrap();
    let attr = PosixSpawnAttr::init().unwrap();

    let bin = "2b7433c4-523b-470c-abb5-d7ee9fd295d5-fdasf";
    let _pid = spawn::posix_spawn(bin, &actions, &attr, args, envs).unwrap();
}

#[test]
fn spawnp_true() {
    let _guard = FORK_MTX.lock();

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
fn spawnp_sleep() {
    let _guard = FORK_MTX.lock();

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
