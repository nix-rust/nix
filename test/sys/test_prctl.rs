use nix::sys::prctl::{prctl, PrctlOption};

#[test]
fn test_prctl() {
    let name = b"abc\0";
    prctl(PrctlOption::PR_SET_NAME, name.as_ptr() as u64, 0, 0, 0).unwrap();

    let mut buf = [b'z'; 16];
    prctl(PrctlOption::PR_GET_NAME, buf.as_mut_ptr() as u64, 0, 0, 0).unwrap();

    assert_eq!(buf[0..4], name[..]);
}
