use nix::errno::Errno;

#[test]
fn errno_set_and_read() {
    Errno::clear();
    Errno::set(Errno::ENFILE);
    assert_eq!(Errno::last(), Errno::ENFILE);
}
