use nix::fcntl::{openat, open, OFlag, O_RDONLY, readlink, readlinkat, rename, renameat};
use nix::sys::stat::Mode;
use nix::unistd::{close, read};
use tempdir::TempDir;
use tempfile::NamedTempFile;
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::fs;

#[test]
fn test_openat() {
    const CONTENTS: &'static [u8] = b"abcd";
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write(CONTENTS).unwrap();

    let dirfd = open(tmp.path().parent().unwrap(),
                     OFlag::empty(),
                     Mode::empty()).unwrap();
    let fd = openat(dirfd,
                    tmp.path().file_name().unwrap(),
                    O_RDONLY,
                    Mode::empty()).unwrap();

    let mut buf = [0u8; 1024];
    assert_eq!(4, read(fd, &mut buf).unwrap());
    assert_eq!(CONTENTS, &buf[0..4]);

    close(fd).unwrap();
    close(dirfd).unwrap();
}

#[test]
fn test_readlink() {
    let tempdir = TempDir::new("nix-test_readdir")
        .unwrap_or_else(|e| panic!("tempdir failed: {}", e));
    let src = tempdir.path().join("a");
    let dst = tempdir.path().join("b");
    println!("a: {:?}, b: {:?}", &src, &dst);
    fs::symlink(&src.as_path(), &dst.as_path()).unwrap();
    let dirfd = open(tempdir.path(),
                     OFlag::empty(),
                     Mode::empty()).unwrap();

    let mut buf = vec![0; src.to_str().unwrap().len() + 1];
    assert_eq!(readlink(&dst, &mut buf).unwrap().to_str().unwrap(),
               src.to_str().unwrap());
    assert_eq!(readlinkat(dirfd, "b", &mut buf).unwrap().to_str().unwrap(),
               src.to_str().unwrap());
}

#[test]
fn test_rename() {
    let tempdir = TempDir::new("nix-test_rename")
        .unwrap_or_else(|e| panic!("tempdir failed: {}", e));
    let src = tempdir.path().join("a");
    let dst = tempdir.path().join("b");
    File::create(&src).unwrap();

    rename(&src, &dst).unwrap();
    assert!(dst.exists());

    let dir = open(tempdir.path(),
                   OFlag::empty(),
                   Mode::empty()).unwrap();
    renameat(dir, "b", dir, "a").unwrap();
    assert!(src.exists());
}

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
