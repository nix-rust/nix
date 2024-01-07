use nix::errno::Errno;

#[test]
fn errno_set_and_read() {
    Errno::clear();
    Errno::ENFILE.set();
    assert_eq!(Errno::last(), Errno::ENFILE);
}
