use nix::unistd::*;
use nix::unistd::ForkResult::*;
use nix::sys::wait::*;
use std::ffi::CString;

#[test]
fn test_fork_and_waitpid() {
    let pid = fork();
    match pid {
      Ok(Child) => {} // ignore child here
      Ok(Parent { child }) => {
          // assert that child was created and pid > 0
          assert!(child > 0);
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
fn test_getpid() {
    let pid = getpid();
    let ppid = getppid();
    assert!(pid > 0);
    assert!(ppid > 0);
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux_android {
    use nix::unistd::gettid;

    #[test]
    fn test_gettid() {
        let tid = gettid();
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

execve_test_factory!(test_execve, execve, b"/bin/sh", b"/system/bin/sh");

#[cfg(any(target_os = "linux", target_os = "android"))]
#[cfg(feature = "execvpe")]
execve_test_factory!(test_execvpe, execvpe, b"sh", b"sh");
