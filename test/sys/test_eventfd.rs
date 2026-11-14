use nix::errno::Errno;
use nix::sys::eventfd::{EfdFlags, EventFd};

#[test]
fn test_eventfd_counter() {
    let efd = EventFd::from_value(3).unwrap();
    assert_eq!(efd.write(4).unwrap(), 8);
    // A non-semaphore read returns the whole counter and resets it.
    assert_eq!(efd.read().unwrap(), 7);
}

#[test]
fn test_eventfd_semaphore() {
    let efd = EventFd::from_value_and_flags(
        2,
        EfdFlags::EFD_SEMAPHORE | EfdFlags::EFD_NONBLOCK,
    )
    .unwrap();
    // In semaphore mode each read decrements the counter by one.
    assert_eq!(efd.read().unwrap(), 1);
    assert_eq!(efd.read().unwrap(), 1);
    assert_eq!(efd.read(), Err(Errno::EAGAIN));
}

#[test]
fn test_eventfd_nonblock_empty() {
    let efd = EventFd::from_flags(EfdFlags::EFD_NONBLOCK).unwrap();
    assert_eq!(efd.read(), Err(Errno::EAGAIN));
}
