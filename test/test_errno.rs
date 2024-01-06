use nix::errno::{set_errno, Errno};

#[test]
fn errno_set_and_read() {
    Errno::clear();
    set_errno(Errno::ENFILE);
    assert_eq!(Errno::last(), Errno::ENFILE);
}
