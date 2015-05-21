use nix::sys::ioctl::*;

// See C code for source of values for op calculations:
// https://gist.github.com/posborne/83ea6880770a1aef332e

#[test]
fn test_op_none() {
    assert_eq!(op_none('q' as u8, 10), 0x0000710A);
    assert_eq!(op_none('a' as u8, 255), 0x000061FF);
}

#[test]
fn test_op_write() {
    assert_eq!(op_write('z' as u8, 10, 1), 0x40017A0A);
    assert_eq!(op_write('z' as u8, 10, 512), 0x42007A0A);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn test_op_write_64() {
    assert_eq!(op_write('z' as u8, 10, 1 << 32), 0x40007A0A);
}

#[test]
fn test_op_read() {
    assert_eq!(op_read('z' as u8, 10, 1), 0x80017A0A);
    assert_eq!(op_read('z' as u8, 10, 512), 0x82007A0A);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn test_op_read_64() {
    assert_eq!(op_read('z' as u8, 10, 1 << 32), 0x80007A0A);
}

#[test]
fn test_op_read_write() {
    assert_eq!(op_read_write('z' as u8, 10, 1), 0xC0017A0A);
    assert_eq!(op_read_write('z' as u8, 10, 512), 0xC2007A0A);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn test_op_read_write_64() {
    assert_eq!(op_read_write('z' as u8, 10, 1 << 32), 0xC0007A0A);
}
