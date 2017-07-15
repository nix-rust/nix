extern crate tempdir;

use nix::unistd::*;
use nix::unistd::ForkResult::*;
use nix::sys::wait::*;
use nix::sys::stat;
use std::iter;
use std::ffi::CString;
use std::fs::File;
use std::io::Write;
use std::os::unix::prelude::*;
use std::env::current_dir;
use tempfile::tempfile;
use tempdir::TempDir;
use libc::off_t;

#[test]
fn test_fork_and_waitpid() {
    let pid = fork();
    match pid {
        Ok(Child) => {} // ignore child here
        Ok(Parent { child }) => {
            // assert that child was created and pid > 0
            let child_raw: ::libc::pid_t = child.into();
            assert!(child_raw > 0);
            let wait_status = waitpid(child, None);
            match wait_status {
                // assert that waitpid returned correct status and the pid is the one of the child
                Ok(WaitStatus::Exited(pid_t, _)) =>  assert!(pid_t == child),

                // panic, must never happen
                Ok(_) => panic!("Child still alive, should never happen"),

                // panic, waitpid should never fail
                Err(_) => panic!("Error: waitpid Failed")
            }

        },
        // panic, fork should never fail unless there is a serious problem with the OS
        Err(_) => panic!("Error: Fork Failed")
    }
}

#[test]
fn test_wait() {
    let pid = fork();
    match pid {
        Ok(Child) => {} // ignore child here
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
    let result = mkstemp("/tmp/nix_tempfile.XXXXXX");
    match result {
        Ok((fd, path)) => {
            close(fd).unwrap();
            unlink(path.as_path()).unwrap();
        },
        Err(e) => panic!("mkstemp failed: {}", e)
    }

    let result = mkstemp("/tmp/");
    match result {
        Ok(_) => {
            panic!("mkstemp succeeded even though it should fail (provided a directory)");
        },
        Err(_) => {}
    }
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
        // The `exec`d process will write to `writer`, and we'll read that
        // data from `reader`.
        let (reader, writer) = pipe().unwrap();

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
    let tmpdir = TempDir::new("test_fchdir").unwrap();
    let tmpdir_path = tmpdir.path().canonicalize().unwrap();
    let tmpdir_fd = File::open(&tmpdir_path).unwrap().into_raw_fd();
    let olddir_path = getcwd().unwrap();
    let olddir_fd = File::open(&olddir_path).unwrap().into_raw_fd();

    assert!(fchdir(tmpdir_fd).is_ok());
    assert_eq!(getcwd().unwrap(), tmpdir_path);

    assert!(fchdir(olddir_fd).is_ok());
    assert_eq!(getcwd().unwrap(), olddir_path);

    assert!(close(olddir_fd).is_ok());
    assert!(close(tmpdir_fd).is_ok());
}

#[test]
fn test_getcwd() {
    let tmp_dir = TempDir::new("test_getcwd").unwrap();
    assert!(chdir(tmp_dir.path()).is_ok());
    assert_eq!(getcwd().unwrap(), current_dir().unwrap());

    // make path 500 chars longer so that buffer doubling in getcwd kicks in.
    // Note: One path cannot be longer than 255 bytes (NAME_MAX)
    // whole path cannot be longer than PATH_MAX (usually 4096 on linux, 1024 on macos)
    let mut inner_tmp_dir = tmp_dir.path().to_path_buf();
    for _ in 0..5 {
        let newdir = iter::repeat("a").take(100).collect::<String>();
        //inner_tmp_dir = inner_tmp_dir.join(newdir).path();
        inner_tmp_dir.push(newdir);
        assert!(mkdir(inner_tmp_dir.as_path(), stat::S_IRWXU).is_ok());
    }
    assert!(chdir(inner_tmp_dir.as_path()).is_ok());
    assert_eq!(getcwd().unwrap(), current_dir().unwrap());
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

#[cfg(any(target_os = "linux", target_os = "android"))]
#[cfg(feature = "execvpe")]
execve_test_factory!(test_execvpe, execvpe, b"sh", b"sh");

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
