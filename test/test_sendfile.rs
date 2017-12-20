use std::io::prelude::*;
use std::os::unix::prelude::*;

use tempfile::tempfile;

use libc::off_t;

use nix::unistd::{close, pipe, read};
use nix::sys::sendfile::sendfile;

#[test]
fn test_sendfile() {
    const CONTENTS: &[u8] = b"abcdef123456";
    let mut tmp = tempfile().unwrap();
    tmp.write_all(CONTENTS).unwrap();

    let (rd, wr) = pipe().unwrap();
    let mut offset: off_t = 5;
    let res = sendfile(wr, tmp.as_raw_fd(), Some(&mut offset), 2).unwrap();

    assert_eq!(2, res);

    let mut buf = [0u8; 1024];
    assert_eq!(2, read(rd, &mut buf).unwrap());
    assert_eq!(b"f1", &buf[0..2]);
    assert_eq!(7, offset);

    close(rd).unwrap();
    close(wr).unwrap();
}
