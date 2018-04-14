extern crate nix;
extern crate tempfile;

use nix::sys::aio::*;
use nix::sys::signal::*;
use std::os::unix::io::AsRawFd;
use tempfile::tempfile;

// Test dropping an AioCb that hasn't yet finished.
// This must happen in its own process, because on OSX this test seems to hose
// the AIO subsystem and causes subsequent tests to fail
#[test]
#[should_panic(expected = "Dropped an in-progress AioCb")]
#[cfg(not(target_env = "musl"))]
fn test_drop() {
    const WBUF: &[u8] = b"CDEF";

    let f = tempfile().unwrap();
    f.set_len(6).unwrap();
    let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           WBUF,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_NOP);
    aiocb.write().unwrap();
}
