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

// Make sure documentation works
ioctl! {
    /// This documents the ioctl function
    bad none do_bad_docs with 0x1234
}
ioctl! {
    /// This documents the ioctl function
    bad read do_bad_read_docs with 0x1234; u16
}
ioctl! {
    /// This documents the ioctl function
    bad write_int do_bad_write_int_docs with 0x1234
}
ioctl! {
    /// This documents the ioctl function
    bad write_ptr do_bad_write_ptr_docs with 0x1234; u8
}
ioctl! {
    /// This documents the ioctl function
    bad readwrite do_bad_readwrite_docs with 0x1234; u32
}
ioctl! {
    /// This documents the ioctl function
    none do_none_docs with 0, 0
}
ioctl! {
    /// This documents the ioctl function
    read do_read_docs with 0, 0; u32
}
ioctl! {
    /// This documents the ioctl function
    write_int do_write_int_docs with 0, 0
}
ioctl! {
    /// This documents the ioctl function
    write_ptr do_write_ptr_docs with 0, 0; u32
}
ioctl! {
    /// This documents the ioctl function
    readwrite do_readwrite_docs with 0, 0; u32
}
ioctl! {
    /// This documents the ioctl function
    read_buf do_read_buf_docs with 0, 0; u32
}
ioctl! {
    /// This documents the ioctl function
    write_buf do_write_buf_docs with 0, 0; u32
}
ioctl! {
    /// This documents the ioctl function
    readwrite_buf do_readwrite_buf_docs with 0, 0; u32
}

// See C code for source of values for op calculations (does NOT work for mips/powerpc):
// https://gist.github.com/posborne/83ea6880770a1aef332e
//
// TODO:  Need a way to compute these constants at test time.  Using precomputed
// values is fragile and needs to be maintained.

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux {
    #[test]
    fn test_op_none() {
        if cfg!(any(target_arch = "mips", target_arch = "mips64", target_arch="powerpc", target_arch="powerpc64")){
            assert_eq!(io!(b'q', 10), 0x2000710A);
            assert_eq!(io!(b'a', 255), 0x200061FF);
        } else {
            assert_eq!(io!(b'q', 10), 0x0000710A);
            assert_eq!(io!(b'a', 255), 0x000061FF);
        }
    }

    #[test]
    fn test_op_write() {
        if cfg!(any(target_arch = "mips", target_arch = "mips64", target_arch="powerpc", target_arch="powerpc64")){
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
        if cfg!(any(target_arch = "mips64", target_arch="powerpc64")){
            assert_eq!(iow!(b'z', 10, (1 as u64) << 32), 0x80007A0A);
        } else {
            assert_eq!(iow!(b'z', 10, (1 as u64) << 32), 0x40007A0A);
        }

    }

    #[test]
    fn test_op_read() {
        if cfg!(any(target_arch = "mips", target_arch = "mips64", target_arch="powerpc", target_arch="powerpc64")){
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
        if cfg!(any(target_arch = "mips64", target_arch="powerpc64")){
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

#[cfg(any(target_os = "android", target_os = "linux"))]
mod linux_ioctls {
    use std::mem;
    use std::os::unix::io::AsRawFd;

    use tempfile::tempfile;
    use libc::{TCGETS, TCSBRK, TCSETS, TIOCNXCL, termios};

    use nix::Error::Sys;
    use nix::errno::Errno::{ENOTTY, ENOSYS};

    ioctl!(bad none tiocnxcl with TIOCNXCL);
    #[test]
    fn test_ioctl_bad_none() {
        let file = tempfile().unwrap();
        let res = unsafe { tiocnxcl(file.as_raw_fd()) };
        assert_eq!(res, Err(Sys(ENOTTY)));
    }

    ioctl!(bad read tcgets with TCGETS; termios);
    #[test]
    fn test_ioctl_bad_read() {
        let file = tempfile().unwrap();
        let mut termios = unsafe { mem::uninitialized() };
        let res = unsafe { tcgets(file.as_raw_fd(), &mut termios) };
        assert_eq!(res, Err(Sys(ENOTTY)));
    }

    ioctl!(bad write_int tcsbrk with TCSBRK);
    #[test]
    fn test_ioctl_bad_write_int() {
        let file = tempfile().unwrap();
        let res = unsafe { tcsbrk(file.as_raw_fd(), 0) };
        assert_eq!(res, Err(Sys(ENOTTY)));
    }

    ioctl!(bad write_ptr tcsets with TCSETS; termios);
    #[test]
    fn test_ioctl_bad_write_ptr() {
        let file = tempfile().unwrap();
        let termios: termios = unsafe { mem::uninitialized() };
        let res = unsafe { tcsets(file.as_raw_fd(), &termios) };
        assert_eq!(res, Err(Sys(ENOTTY)));
    }

    // FIXME: Find a suitable example for "bad readwrite".

    // From linux/videodev2.h
    ioctl!(none log_status with b'V', 70);
    #[test]
    fn test_ioctl_none() {
        let file = tempfile().unwrap();
        let res = unsafe { log_status(file.as_raw_fd()) };
        assert!(res == Err(Sys(ENOTTY)) || res == Err(Sys(ENOSYS)));
    }

    #[repr(C)]
    pub struct v4l2_audio {
        index: u32,
        name: [u8; 32],
        capability: u32,
        mode: u32,
        reserved: [u32; 2],
    }

    // From linux/videodev2.h
    ioctl!(write_ptr s_audio with b'V', 34; v4l2_audio);
    #[test]
    fn test_ioctl_read() {
        let file = tempfile().unwrap();
        let data: v4l2_audio = unsafe { mem::uninitialized() };
        let res = unsafe { g_audio(file.as_raw_fd(), &data) };
        assert!(res == Err(Sys(ENOTTY)) || res == Err(Sys(ENOSYS)));
    }

    // From linux/net/bluetooth/hci_sock.h
    const HCI_IOC_MAGIC: u8 = b'H';
    const HCI_IOC_HCIDEVUP: u8 = 201;
    ioctl!(write_int hcidevup with HCI_IOC_MAGIC, HCI_IOC_HCIDEVUP);
    #[test]
    fn test_ioctl_write_int() {
        let file = tempfile().unwrap();
        let res = unsafe { hcidevup(file.as_raw_fd(), 0) };
        assert!(res == Err(Sys(ENOTTY)) || res == Err(Sys(ENOSYS)));
    }

    // From linux/videodev2.h
    ioctl!(write_ptr g_audio with b'V', 33; v4l2_audio);
    #[test]
    fn test_ioctl_write_ptr() {
        let file = tempfile().unwrap();
        let mut data: v4l2_audio = unsafe { mem::uninitialized() };
        let res = unsafe { g_audio(file.as_raw_fd(), &mut data) };
        assert!(res == Err(Sys(ENOTTY)) || res == Err(Sys(ENOSYS)));
    }

    // From linux/videodev2.h
    ioctl!(readwrite enum_audio with b'V', 65; v4l2_audio);
    #[test]
    fn test_ioctl_readwrite() {
        let file = tempfile().unwrap();
        let mut data: v4l2_audio = unsafe { mem::uninitialized() };
        let res = unsafe { enum_audio(file.as_raw_fd(), &mut data) };
        assert!(res == Err(Sys(ENOTTY)) || res == Err(Sys(ENOSYS)));
    }

    // FIXME: Find a suitable example for read_buf.

    #[repr(C)]
    pub struct spi_ioc_transfer {
        tx_buf: u64,
        rx_buf: u64,
        len: u32,
        speed_hz: u32,
        delay_usecs: u16,
        bits_per_word: u8,
        cs_change: u8,
        tx_nbits: u8,
        rx_nbits: u8,
        pad: u16,
    }

    // From linux/spi/spidev.h
    ioctl!(write_buf spi_ioc_message with super::SPI_IOC_MAGIC, super::SPI_IOC_MESSAGE; spi_ioc_transfer);
    #[test]
    fn test_ioctl_write_buf() {
        let file = tempfile().unwrap();
        let mut data: [spi_ioc_transfer; 4] = unsafe { mem::uninitialized() };
        let res = unsafe { spi_ioc_message(file.as_raw_fd(), &mut data[..]) };
        assert!(res == Err(Sys(ENOTTY)) || res == Err(Sys(ENOSYS)));
    }

    // FIXME: Find a suitable example for readwrite_buf.
}

#[cfg(target_os = "freebsd")]
mod freebsd_ioctls {
    use std::mem;
    use std::os::unix::io::AsRawFd;

    use tempfile::tempfile;
    use libc::termios;

    use nix::Error::Sys;
    use nix::errno::Errno::ENOTTY;

    // From sys/sys/ttycom.h
    const TTY_IOC_MAGIC: u8 = b't';
    const TTY_IOC_TYPE_NXCL: u8 = 14;
    const TTY_IOC_TYPE_GETA: u8 = 19;
    const TTY_IOC_TYPE_SETA: u8 = 20;

    ioctl!(none tiocnxcl with TTY_IOC_MAGIC, TTY_IOC_TYPE_NXCL);
    #[test]
    fn test_ioctl_none() {
        let file = tempfile().unwrap();
        let res = unsafe { tiocnxcl(file.as_raw_fd()) };
        assert_eq!(res, Err(Sys(ENOTTY)));
    }

    ioctl!(read tiocgeta with TTY_IOC_MAGIC, TTY_IOC_TYPE_GETA; termios);
    #[test]
    fn test_ioctl_read() {
        let file = tempfile().unwrap();
        let mut termios = unsafe { mem::uninitialized() };
        let res = unsafe { tiocgeta(file.as_raw_fd(), &mut termios) };
        assert_eq!(res, Err(Sys(ENOTTY)));
    }

    ioctl!(write_ptr tiocseta with TTY_IOC_MAGIC, TTY_IOC_TYPE_SETA; termios);
    #[test]
    fn test_ioctl_write_ptr() {
        let file = tempfile().unwrap();
        let termios: termios = unsafe { mem::uninitialized() };
        let res = unsafe { tiocseta(file.as_raw_fd(), &termios) };
        assert_eq!(res, Err(Sys(ENOTTY)));
    }
}
