#[macro_use]
extern crate nix;
extern crate libc;
extern crate rand;
extern crate tempdir;
extern crate tempfile;

extern crate nix_test as nixtest;

mod sys;
mod test_fcntl;
mod test_net;
mod test_nix_path;
#[cfg(any(target_os = "linux", target_os = "android"))]
mod test_sendfile;
mod test_stat;
mod test_unistd;

#[cfg(any(target_os = "linux"))]
mod test_mq;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod test_poll;

mod ports {
    use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};
    use std::sync::atomic::Ordering::SeqCst;

    // Helper for getting a unique port for the task run
    // TODO: Reuse ports to not spam the system
    static mut NEXT_PORT: AtomicUsize = ATOMIC_USIZE_INIT;
    const FIRST_PORT: usize = 18080;

    pub fn next_port() -> usize {
        unsafe {
            // If the atomic was never used, set it to the initial port
            NEXT_PORT.compare_and_swap(0, FIRST_PORT, SeqCst);

            // Get and increment the port list
            NEXT_PORT.fetch_add(1, SeqCst)
        }
    }

    pub fn localhost() -> String {
        format!("127.0.0.1:{}", next_port())
    }
}

use nixtest::assert_size_of;

#[test]
pub fn test_size_of_long() {
    // This test is mostly here to ensure that 32bit CI is correctly
    // functioning
    assert_size_of::<usize>("long");
}
