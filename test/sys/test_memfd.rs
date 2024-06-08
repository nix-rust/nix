#[test]
fn test_memfd_create() {
    use nix::sys::memfd::memfd_create;
    use nix::sys::memfd::MemFdCreateFlag;
    use nix::unistd::lseek;
    use nix::unistd::read;
    use nix::unistd::{write, Whence};
    use std::os::fd::{AsFd, AsRawFd};

    let fd =
        memfd_create("test_memfd_create_name", MemFdCreateFlag::MFD_CLOEXEC)
            .unwrap();
    let contents = b"hello";
    assert_eq!(write(fd.as_fd(), contents).unwrap(), 5);

    lseek(fd.as_raw_fd(), 0, Whence::SeekSet).unwrap();

    let mut buf = vec![0_u8; contents.len()];
    assert_eq!(read(fd.as_raw_fd(), &mut buf).unwrap(), 5);

    assert_eq!(contents, buf.as_slice());
}
