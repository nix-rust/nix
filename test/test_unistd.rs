extern crate tempdir;

use nix::unistd::*;
use nix::unistd::ForkResult::*;
use nix::sys::wait::*;
use nix::sys::stat;
use std::{env, iter};
use std::ffi::CString;
use std::fs::File;
use std::io::Write;
use std::os::unix::prelude::*;
use tempfile::tempfile;
use tempdir::TempDir;
use libc::{_exit, off_t};

#[test]
fn test_fork_and_waitpid() {
    #[allow(unused_variables)]
    let m = ::FORK_MTX.lock().expect("Mutex got poisoned by another test");

    // Safe: Child only calls `_exit`, which is signal-safe
    let pid = fork();
    match pid {
        Ok(Child) => unsafe { _exit(0) },
        Ok(Parent { child }) => {
            // assert that child was created and pid > 0
            let child_raw: ::libc::pid_t = child.into();
            assert!(child_raw > 0);
            let wait_status = waitpid(child, None);
            match wait_status {
                // assert that waitpid returned correct status and the pid is the one of the child
                Ok(WaitStatus::Exited(pid_t, _)) =>  assert!(pid_t == child),

                // panic, must never happen
                s @ Ok(_) => panic!("Child exited {:?}, should never happen", s),

                // panic, waitpid should never fail
                Err(s) => panic!("Error: waitpid returned Err({:?}", s)
            }

        },
        // panic, fork should never fail unless there is a serious problem with the OS
        Err(_) => panic!("Error: Fork Failed")
    }
}

#[test]
fn test_wait() {
    // Grab FORK_MTX so wait doesn't reap a different test's child process
    #[allow(unused_variables)]
    let m = ::FORK_MTX.lock().expect("Mutex got poisoned by another test");

    // Safe: Child only calls `_exit`, which is signal-safe
    let pid = fork();
    match pid {
        Ok(Child) => unsafe { _exit(0) },
        Ok(Parent { child }) => {
            let wait_status = wait();

            // just assert that (any) one child returns with WaitStatus::Exited
            assert_eq!(wait_status, Ok(WaitStatus::Exited(child, 0)));
        },
        // panic, fork should never fail unless there is a serious problem with the OS
        Err(_) => panic!("Error: Fork Failed")
    }
}

#[test]
fn test_mkstemp() {
    let mut path = env::temp_dir();
    path.push("nix_tempfile.XXXXXX");

    let result = mkstemp(&path);
    match result {
        Ok((fd, path)) => {
            close(fd).unwrap();
            unlink(path.as_path()).unwrap();
        },
        Err(e) => panic!("mkstemp failed: {}", e)
    }
}

#[test]
fn test_mkstemp_directory() {
    // mkstemp should fail if a directory is given
    assert!(mkstemp(&env::temp_dir()).is_err());
}

#[test]
fn test_getpid() {
    let pid: ::libc::pid_t = getpid().into();
    let ppid: ::libc::pid_t = getppid().into();
    assert!(pid > 0);
    assert!(ppid > 0);
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux_android {
    use nix::unistd::gettid;

    #[test]
    fn test_gettid() {
        let tid: ::libc::pid_t = gettid().into();
        assert!(tid > 0);
    }
}

macro_rules! execve_test_factory(
    ($test_name:ident, $syscall:ident, $unix_sh:expr, $android_sh:expr) => (
    #[test]
    fn $test_name() {
        #[allow(unused_variables)]
        let m = ::FORK_MTX.lock().expect("Mutex got poisoned by another test");
        // The `exec`d process will write to `writer`, and we'll read that
        // data from `reader`.
        let (reader, writer) = pipe().unwrap();

        // Safe: Child calls `exit`, `dup`, `close` and the provided `exec*` family function.
        // NOTE: Technically, this makes the macro unsafe to use because you could pass anything.
        //       The tests make sure not to do that, though.
        match fork().unwrap() {
            Child => {
                #[cfg(not(target_os = "android"))]
                const SH_PATH: &'static [u8] = $unix_sh;

                #[cfg(target_os = "android")]
                const SH_PATH: &'static [u8] = $android_sh;

                // Close stdout.
                close(1).unwrap();
                // Make `writer` be the stdout of the new process.
                dup(writer).unwrap();
                // exec!
                $syscall(
                    &CString::new(SH_PATH).unwrap(),
                    &[CString::new(b"".as_ref()).unwrap(),
                      CString::new(b"-c".as_ref()).unwrap(),
                      CString::new(b"echo nix!!! && echo foo=$foo && echo baz=$baz"
                                   .as_ref()).unwrap()],
                    &[CString::new(b"foo=bar".as_ref()).unwrap(),
                      CString::new(b"baz=quux".as_ref()).unwrap()]).unwrap();
            },
            Parent { child } => {
                // Wait for the child to exit.
                waitpid(child, None).unwrap();
                // Read 1024 bytes.
                let mut buf = [0u8; 1024];
                read(reader, &mut buf).unwrap();
                // It should contain the things we printed using `/bin/sh`.
                let string = String::from_utf8_lossy(&buf);
                assert!(string.contains("nix!!!"));
                assert!(string.contains("foo=bar"));
                assert!(string.contains("baz=quux"));
            }
        }
    }
    )
);

#[test]
fn test_fchdir() {
    // fchdir changes the process's cwd
    #[allow(unused_variables)]
    let m = ::CWD_MTX.lock().expect("Mutex got poisoned by another test");

    let tmpdir = TempDir::new("test_fchdir").unwrap();
    let tmpdir_path = tmpdir.path().canonicalize().unwrap();
    let tmpdir_fd = File::open(&tmpdir_path).unwrap().into_raw_fd();

    assert!(fchdir(tmpdir_fd).is_ok());
    assert_eq!(getcwd().unwrap(), tmpdir_path);

    assert!(close(tmpdir_fd).is_ok());
}

#[test]
fn test_getcwd() {
    // chdir changes the process's cwd
    #[allow(unused_variables)]
    let m = ::CWD_MTX.lock().expect("Mutex got poisoned by another test");

    let tmpdir = TempDir::new("test_getcwd").unwrap();
    let tmpdir_path = tmpdir.path().canonicalize().unwrap();
    assert!(chdir(&tmpdir_path).is_ok());
    assert_eq!(getcwd().unwrap(), tmpdir_path);

    // make path 500 chars longer so that buffer doubling in getcwd
    // kicks in.  Note: One path cannot be longer than 255 bytes
    // (NAME_MAX) whole path cannot be longer than PATH_MAX (usually
    // 4096 on linux, 1024 on macos)
    let mut inner_tmp_dir = tmpdir_path.to_path_buf();
    for _ in 0..5 {
        let newdir = iter::repeat("a").take(100).collect::<String>();
        inner_tmp_dir.push(newdir);
        assert!(mkdir(inner_tmp_dir.as_path(), stat::S_IRWXU).is_ok());
    }
    assert!(chdir(inner_tmp_dir.as_path()).is_ok());
    assert_eq!(getcwd().unwrap(), inner_tmp_dir.as_path());
}

#[test]
fn test_lseek() {
    const CONTENTS: &'static [u8] = b"abcdef123456";
    let mut tmp = tempfile().unwrap();
    tmp.write_all(CONTENTS).unwrap();
    let tmpfd = tmp.into_raw_fd();

    let offset: off_t = 5;
    lseek(tmpfd, offset, Whence::SeekSet).unwrap();

    let mut buf = [0u8; 7];
    ::read_exact(tmpfd, &mut buf);
    assert_eq!(b"f123456", &buf);

    close(tmpfd).unwrap();
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[test]
fn test_lseek64() {
    const CONTENTS: &'static [u8] = b"abcdef123456";
    let mut tmp = tempfile().unwrap();
    tmp.write(CONTENTS).unwrap();
    let tmpfd = tmp.into_raw_fd();

    lseek64(tmpfd, 5, Whence::SeekSet).unwrap();

    let mut buf = [0u8; 7];
    ::read_exact(tmpfd, &mut buf);
    assert_eq!(b"f123456", &buf);

    close(tmpfd).unwrap();
}

execve_test_factory!(test_execve, execve, b"/bin/sh", b"/system/bin/sh");

#[test]
fn test_fpathconf_limited() {
    let f = tempfile().unwrap();
    // AFAIK, PATH_MAX is limited on all platforms, so it makes a good test
    let path_max = fpathconf(f.as_raw_fd(), PathconfVar::PATH_MAX);
    assert!(path_max.expect("fpathconf failed").expect("PATH_MAX is unlimited") > 0);
}

#[test]
fn test_pathconf_limited() {
    // AFAIK, PATH_MAX is limited on all platforms, so it makes a good test
    let path_max = pathconf(".", PathconfVar::PATH_MAX);
    assert!(path_max.expect("pathconf failed").expect("PATH_MAX is unlimited") > 0);
}

#[test]
fn test_sysconf_limited() {
    // AFAIK, OPEN_MAX is limited on all platforms, so it makes a good test
    let open_max = sysconf(SysconfVar::OPEN_MAX);
    assert!(open_max.expect("sysconf failed").expect("OPEN_MAX is unlimited") > 0);
}

#[cfg(target_os = "freebsd")]
#[test]
fn test_sysconf_unsupported() {
    // I know of no sysconf variables that are unsupported everywhere, but
    // _XOPEN_CRYPT is unsupported on FreeBSD 11.0, which is one of the platforms
    // we test.
    let open_max = sysconf(SysconfVar::_XOPEN_CRYPT);
    assert!(open_max.expect("sysconf failed").is_none())
}
