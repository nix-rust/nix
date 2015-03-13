use nix::unistd::*;
use nix::unistd::Fork::*;
use nix::sys::wait::*;
use std::ffi::CString;

#[test]
fn test_fork_and_waitpid() {
    let pid = fork();
    match pid {
      Ok(Child) => {} // ignore child here
      Ok(Parent(child_pid)) => {
          // assert that child was created and pid > 0
          assert!(child_pid > 0);
          let wait_status = waitpid(child_pid, None);
          match wait_status {
              // assert that waitpid returned correct status and the pid is the one of the child
              Ok(WaitStatus::Exited(pid_t)) =>  assert!(pid_t == child_pid),

              // panic, must never happen
              Ok(WaitStatus::StillAlive) => panic!("Child still alive, should never happen"),

              // panic, waitpid should never fail
              Err(_) => panic!("Error: waitpid Failed")
          }

      },
      // panic, fork should never fail unless there is a serious problem with the OS
      Err(_) => panic!("Error: Fork Failed")
    }
}


#[test]
fn test_execve() {
    // The `exec`d process will write to `writer`, and we'll read that
    // data from `reader`.
    let (reader, writer) = pipe().unwrap();

    match fork().unwrap() {
        Child => {
            #[cfg(not(target_os = "android"))]
            const SH_PATH: &'static [u8] = b"/bin/sh";

            #[cfg(target_os = "android")]
            const SH_PATH: &'static [u8] = b"/system/bin/sh";

            // Close stdout.
            close(1).unwrap();
            // Make `writer` be the stdout of the new process.
            dup(writer).unwrap();
            // exec!
            execve(&CString::new(SH_PATH).unwrap(),
                   &[CString::new(b"").unwrap(),
                     CString::new(b"-c").unwrap(),
                     CString::new(b"echo nix!!! && echo foo=$foo && echo baz=$baz").unwrap()],
                   &[CString::new(b"foo=bar").unwrap(),
                     CString::new(b"baz=quux").unwrap()]).unwrap();
        },
        Parent(child_pid) => {
            // Wait for the child to exit.
            waitpid(child_pid, None).unwrap();
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
