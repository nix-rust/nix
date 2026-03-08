use nix::{
    errno::Errno,
    sys::pidfd::{pidfd_open, PidfdFlags},
    unistd::getpid,
};

#[test]
fn test_pidfd_open() {
    match pidfd_open(getpid(), PidfdFlags::empty()) {
        Ok(_) => (),
        Err(Errno::ENOSYS) => (),
        Err(e) => panic!("{e}"),
    }
}
