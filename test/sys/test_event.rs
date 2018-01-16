use std::time::Duration;

use libc::timespec;
use nix::sys::event::{kqueue, kevent};

#[test]
pub fn test_kevent() {
    let kq = kqueue().unwrap();

    let mut events = Vec::new();

    let timeout = Duration::from_millis(0);
    assert_eq!(kevent(kq, &[], &mut events, Some(timeout)).unwrap(), 0);

    let timeout = timespec { tv_sec: 0, tv_nsec: 0};
    assert_eq!(kevent(kq, &[], &mut events, Some(timeout)).unwrap(), 0);

    assert_eq!(kevent::<timespec>(kq, &[], &mut events, None).unwrap(), 0);
}
