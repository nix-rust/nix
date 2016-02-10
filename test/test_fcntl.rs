#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux_android {
    use std::io::prelude::*;
    use std::os::unix::prelude::*;

    use libc::loff_t;

    use nix::fcntl::{SpliceFFlags, splice, tee, vmsplice};
    use nix::sys::uio::IoVec;
    use nix::unistd::{close, pipe, read, write};

    use tempfile::tempfile;

    #[test]
    fn test_splice() {
        const CONTENTS: &'static [u8] = b"abcdef123456";
        let mut tmp = tempfile().unwrap();
        tmp.write(CONTENTS).unwrap();

        let (rd, wr) = pipe().unwrap();
        let mut offset: loff_t = 5;
        let res = splice(tmp.as_raw_fd(), Some(&mut offset),
            wr, None, 2, SpliceFFlags::empty()).unwrap();

        assert_eq!(2, res);

        let mut buf = [0u8; 1024];
        assert_eq!(2, read(rd, &mut buf).unwrap());
        assert_eq!(b"f1", &buf[0..2]);
        assert_eq!(7, offset);

        close(rd).unwrap();
        close(wr).unwrap();
    }

    #[test]
    fn test_tee() {
        let (rd1, wr1) = pipe().unwrap();
        let (rd2, wr2) = pipe().unwrap();

        write(wr1, b"abc").unwrap();
        let res = tee(rd1, wr2, 2, SpliceFFlags::empty()).unwrap();

        assert_eq!(2, res);

        let mut buf = [0u8; 1024];

        // Check the tee'd bytes are at rd2.
        assert_eq!(2, read(rd2, &mut buf).unwrap());
        assert_eq!(b"ab", &buf[0..2]);

        // Check all the bytes are still at rd1.
        assert_eq!(3, read(rd1, &mut buf).unwrap());
        assert_eq!(b"abc", &buf[0..3]);

        close(rd1).unwrap();
        close(wr1).unwrap();
        close(rd2).unwrap();
        close(wr2).unwrap();
    }

    #[test]
    fn test_vmsplice() {
        let (rd, wr) = pipe().unwrap();

        let buf1 = b"abcdef";
        let buf2 = b"defghi";
        let mut iovecs = Vec::with_capacity(2);
        iovecs.push(IoVec::from_slice(&buf1[0..3]));
        iovecs.push(IoVec::from_slice(&buf2[0..3]));

        let res = vmsplice(wr, &iovecs[..], SpliceFFlags::empty()).unwrap();

        assert_eq!(6, res);

        // Check the bytes can be read at rd.
        let mut buf = [0u8; 32];
        assert_eq!(6, read(rd, &mut buf).unwrap());
        assert_eq!(b"abcdef", &buf[0..6]);

        close(rd).unwrap();
        close(wr).unwrap();
    }

}
