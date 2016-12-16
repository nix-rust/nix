#![allow(dead_code)]

// Simple tests to ensure macro generated fns compile
ioctl!(do_bad with 0x1234);
ioctl!(none do_none with 0, 0);
ioctl!(read read_test with 0, 0; u32);
ioctl!(write write_test with 0, 0; u64);
ioctl!(readwrite readwrite_test with 0, 0; u64);
ioctl!(read buf readbuf_test with 0, 0; u32);
ioctl!(write buf writebuf_test with 0, 0; u32);
ioctl!(readwrite buf readwritebuf_test with 0, 0; u32);

// See C code for source of values for op calculations:
// https://gist.github.com/posborne/83ea6880770a1aef332e

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux {
    #[test]
    fn test_op_none() {
        assert_eq!(io!(b'q', 10), 0x0000710A);
        assert_eq!(io!(b'a', 255), 0x000061FF);
    }

    #[test]
    fn test_op_write() {
        assert_eq!(iow!(b'z', 10, 1), 0x40017A0A);
        assert_eq!(iow!(b'z', 10, 512), 0x42007A0A);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_op_write_64() {
        assert_eq!(iow!(b'z', 10, (1 as u64) << 32), 0x40007A0A);
    }

    #[test]
    fn test_op_read() {
        assert_eq!(ior!(b'z', 10, 1), 0x80017A0A);
        assert_eq!(ior!(b'z', 10, 512), 0x82007A0A);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_op_read_64() {
        assert_eq!(ior!(b'z', 10, (1 as u64) << 32), 0x80007A0A);
    }

    #[test]
    fn test_op_read_write() {
        assert_eq!(iorw!(b'z', 10, 1), 0xC0017A0A);
        assert_eq!(iorw!(b'z', 10, 512), 0xC2007A0A);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_op_read_write_64() {
        assert_eq!(iorw!(b'z', 10, (1 as u64) << 32), 0xC0007A0A);
    } 
}

#[cfg(any(target_os = "macos",
          target_os = "ios",
          target_os = "netbsd",
          target_os = "openbsd",
          target_os = "freebsd",
          target_os = "dragonfly"))]
mod bsd {
    #[test]
    fn test_op_none() {
        assert_eq!(io!(b'q', 10), 0x2000710A);
        assert_eq!(io!(b'a', 255), 0x200061FF);
    }

    #[test]
    fn test_op_write() {
        assert_eq!(iow!(b'z', 10, 1), 0x80017A0A);
        assert_eq!(iow!(b'z', 10, 512), 0x82007A0A);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_op_write_64() {
        assert_eq!(iow!(b'z', 10, (1 as u64) << 32), 0x80007A0A);
    }

    #[test]
    fn test_op_read() {
        assert_eq!(ior!(b'z', 10, 1), 0x40017A0A);
        assert_eq!(ior!(b'z', 10, 512), 0x42007A0A);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_op_read_64() {
        assert_eq!(ior!(b'z', 10, (1 as u64) << 32), 0x40007A0A);
    }

    #[test]
    fn test_op_read_write() {
        assert_eq!(iorw!(b'z', 10, 1), 0xC0017A0A);
        assert_eq!(iorw!(b'z', 10, 512), 0xC2007A0A);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_op_read_write_64() {
        assert_eq!(iorw!(b'z', 10, (1 as u64) << 32), 0xC0007A0A);
    } 
}
