#![allow(dead_code)]

// Simple tests to ensure macro generated fns compile
ioctl!(bad none do_bad with 0x1234);
ioctl!(bad read do_bad_read with 0x1234; u16);
ioctl!(bad write_int do_bad_write_int with 0x1234);
ioctl!(bad write_ptr do_bad_write_ptr with 0x1234; u8);
ioctl!(bad readwrite do_bad_readwrite with 0x1234; u32);
ioctl!(none do_none with 0, 0);
ioctl!(read read_test with 0, 0; u32);
ioctl!(write_int write_ptr_int with 0, 0);
ioctl!(write_ptr write_ptr_u8 with 0, 0; u8);
ioctl!(write_ptr write_ptr_u32 with 0, 0; u32);
ioctl!(write_ptr write_ptr_u64 with 0, 0; u64);
ioctl!(readwrite readwrite_test with 0, 0; u64);
ioctl!(read_buf readbuf_test with 0, 0; u32);
const SPI_IOC_MAGIC: u8 = b'k';
const SPI_IOC_MESSAGE: u8 = 0;
ioctl!(write_buf writebuf_test_consts with SPI_IOC_MAGIC, SPI_IOC_MESSAGE; u8);
ioctl!(write_buf writebuf_test_u8 with 0, 0; u8);
ioctl!(write_buf writebuf_test_u32 with 0, 0; u32);
ioctl!(write_buf writebuf_test_u64 with 0, 0; u64);
ioctl!(readwrite_buf readwritebuf_test with 0, 0; u32);

// See C code for source of values for op calculations (does NOT work for mips/powerpc):
// https://gist.github.com/posborne/83ea6880770a1aef332e
//
// TODO:  Need a way to compute these constants at test time.  Using precomputed
// values is fragile and needs to be maintained.

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux {
    #[test]
    fn test_op_none() {
        if cfg!(any(target_arch = "mips", target_arch="powerpc", target_arch="powerpc64")){
            assert_eq!(io!(b'q', 10), 0x2000710A);
            assert_eq!(io!(b'a', 255), 0x200061FF);
        } else {
            assert_eq!(io!(b'q', 10), 0x0000710A);
            assert_eq!(io!(b'a', 255), 0x000061FF);
        }
    }

    #[test]
    fn test_op_write() {
        if cfg!(any(target_arch = "mips", target_arch="powerpc", target_arch="powerpc64")){
            assert_eq!(iow!(b'z', 10, 1), 0x80017A0A);
            assert_eq!(iow!(b'z', 10, 512), 0x82007A0A);
        } else {
            assert_eq!(iow!(b'z', 10, 1), 0x40017A0A);
            assert_eq!(iow!(b'z', 10, 512), 0x42007A0A);
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_op_write_64() {
        if cfg!(any(target_arch="powerpc64")){
            assert_eq!(iow!(b'z', 10, (1 as u64) << 32), 0x80007A0A);
        } else {
            assert_eq!(iow!(b'z', 10, (1 as u64) << 32), 0x40007A0A);
        }

    }

    #[test]
    fn test_op_read() {
        if cfg!(any(target_arch = "mips", target_arch="powerpc", target_arch="powerpc64")){
            assert_eq!(ior!(b'z', 10, 1), 0x40017A0A);
            assert_eq!(ior!(b'z', 10, 512), 0x42007A0A);
        } else {
            assert_eq!(ior!(b'z', 10, 1), 0x80017A0A);
            assert_eq!(ior!(b'z', 10, 512), 0x82007A0A);
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_op_read_64() {
        if cfg!(any(target_arch="powerpc64")){
            assert_eq!(ior!(b'z', 10, (1 as u64) << 32), 0x40007A0A);
        } else {
            assert_eq!(ior!(b'z', 10, (1 as u64) << 32), 0x80007A0A);
        }
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
