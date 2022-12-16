#[cfg(not(target_os = "redox"))]
use nix::errno::*;
#[cfg(not(target_os = "redox"))]
use nix::fcntl::{open, readlink, OFlag};
#[cfg(not(target_os = "redox"))]
use nix::fcntl::{openat, readlinkat, renameat};
#[cfg(all(
    target_os = "linux",
    target_env = "gnu",
    any(
        target_arch = "x86_64",
        target_arch = "x32",
        target_arch = "powerpc",
        target_arch = "s390x"
    )
))]
use nix::fcntl::{renameat2, RenameFlags};
#[cfg(not(target_os = "redox"))]
use nix::sys::stat::Mode;
#[cfg(not(target_os = "redox"))]
use nix::unistd::{close, read};
#[cfg(not(target_os = "redox"))]
use std::fs::File;
#[cfg(not(target_os = "redox"))]
use std::io::prelude::*;
#[cfg(not(target_os = "redox"))]
use std::os::unix::fs;
#[cfg(not(target_os = "redox"))]
use tempfile::{self, NamedTempFile};

#[test]
#[cfg(not(target_os = "redox"))]
// QEMU does not handle openat well enough to satisfy this test
// https://gitlab.com/qemu-project/qemu/-/issues/829
#[cfg_attr(qemu, ignore)]
fn test_openat() {
    const CONTENTS: &[u8] = b"abcd";
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(CONTENTS).unwrap();

    let dirfd =
        open(tmp.path().parent().unwrap(), OFlag::empty(), Mode::empty())
            .unwrap();
    let fd = openat(
        dirfd,
        tmp.path().file_name().unwrap(),
        OFlag::O_RDONLY,
        Mode::empty(),
    )
    .unwrap();

    let mut buf = [0u8; 1024];
    assert_eq!(4, read(fd, &mut buf).unwrap());
    assert_eq!(CONTENTS, &buf[0..4]);

    close(fd).unwrap();
    close(dirfd).unwrap();
}

#[test]
#[cfg(not(target_os = "redox"))]
fn test_renameat() {
    let old_dir = tempfile::tempdir().unwrap();
    let old_dirfd =
        open(old_dir.path(), OFlag::empty(), Mode::empty()).unwrap();
    let old_path = old_dir.path().join("old");
    File::create(old_path).unwrap();
    let new_dir = tempfile::tempdir().unwrap();
    let new_dirfd =
        open(new_dir.path(), OFlag::empty(), Mode::empty()).unwrap();
    renameat(Some(old_dirfd), "old", Some(new_dirfd), "new").unwrap();
    assert_eq!(
        renameat(Some(old_dirfd), "old", Some(new_dirfd), "new").unwrap_err(),
        Errno::ENOENT
    );
    close(old_dirfd).unwrap();
    close(new_dirfd).unwrap();
    assert!(new_dir.path().join("new").exists());
}

#[test]
#[cfg(all(
    target_os = "linux",
    target_env = "gnu",
    any(
        target_arch = "x86_64",
        target_arch = "x32",
        target_arch = "powerpc",
        target_arch = "s390x"
    )
))]
fn test_renameat2_behaves_like_renameat_with_no_flags() {
    let old_dir = tempfile::tempdir().unwrap();
    let old_dirfd =
        open(old_dir.path(), OFlag::empty(), Mode::empty()).unwrap();
    let old_path = old_dir.path().join("old");
    File::create(old_path).unwrap();
    let new_dir = tempfile::tempdir().unwrap();
    let new_dirfd =
        open(new_dir.path(), OFlag::empty(), Mode::empty()).unwrap();
    renameat2(
        Some(old_dirfd),
        "old",
        Some(new_dirfd),
        "new",
        RenameFlags::empty(),
    )
    .unwrap();
    assert_eq!(
        renameat2(
            Some(old_dirfd),
            "old",
            Some(new_dirfd),
            "new",
            RenameFlags::empty()
        )
        .unwrap_err(),
        Errno::ENOENT
    );
    close(old_dirfd).unwrap();
    close(new_dirfd).unwrap();
    assert!(new_dir.path().join("new").exists());
}

#[test]
#[cfg(all(
    target_os = "linux",
    target_env = "gnu",
    any(
        target_arch = "x86_64",
        target_arch = "x32",
        target_arch = "powerpc",
        target_arch = "s390x"
    )
))]
fn test_renameat2_exchange() {
    let old_dir = tempfile::tempdir().unwrap();
    let old_dirfd =
        open(old_dir.path(), OFlag::empty(), Mode::empty()).unwrap();
    let old_path = old_dir.path().join("old");
    {
        let mut old_f = File::create(&old_path).unwrap();
        old_f.write_all(b"old").unwrap();
    }
    let new_dir = tempfile::tempdir().unwrap();
    let new_dirfd =
        open(new_dir.path(), OFlag::empty(), Mode::empty()).unwrap();
    let new_path = new_dir.path().join("new");
    {
        let mut new_f = File::create(&new_path).unwrap();
        new_f.write_all(b"new").unwrap();
    }
    renameat2(
        Some(old_dirfd),
        "old",
        Some(new_dirfd),
        "new",
        RenameFlags::RENAME_EXCHANGE,
    )
    .unwrap();
    let mut buf = String::new();
    let mut new_f = File::open(&new_path).unwrap();
    new_f.read_to_string(&mut buf).unwrap();
    assert_eq!(buf, "old");
    buf = "".to_string();
    let mut old_f = File::open(&old_path).unwrap();
    old_f.read_to_string(&mut buf).unwrap();
    assert_eq!(buf, "new");
    close(old_dirfd).unwrap();
    close(new_dirfd).unwrap();
}

#[test]
#[cfg(all(
    target_os = "linux",
    target_env = "gnu",
    any(
        target_arch = "x86_64",
        target_arch = "x32",
        target_arch = "powerpc",
        target_arch = "s390x"
    )
))]
fn test_renameat2_noreplace() {
    let old_dir = tempfile::tempdir().unwrap();
    let old_dirfd =
        open(old_dir.path(), OFlag::empty(), Mode::empty()).unwrap();
    let old_path = old_dir.path().join("old");
    File::create(old_path).unwrap();
    let new_dir = tempfile::tempdir().unwrap();
    let new_dirfd =
        open(new_dir.path(), OFlag::empty(), Mode::empty()).unwrap();
    let new_path = new_dir.path().join("new");
    File::create(new_path).unwrap();
    assert_eq!(
        renameat2(
            Some(old_dirfd),
            "old",
            Some(new_dirfd),
            "new",
            RenameFlags::RENAME_NOREPLACE
        )
        .unwrap_err(),
        Errno::EEXIST
    );
    close(old_dirfd).unwrap();
    close(new_dirfd).unwrap();
    assert!(new_dir.path().join("new").exists());
    assert!(old_dir.path().join("old").exists());
}

#[test]
#[cfg(not(target_os = "redox"))]
fn test_readlink() {
    let tempdir = tempfile::tempdir().unwrap();
    let src = tempdir.path().join("a");
    let dst = tempdir.path().join("b");
    println!("a: {:?}, b: {:?}", &src, &dst);
    fs::symlink(src.as_path(), dst.as_path()).unwrap();
    let dirfd = open(tempdir.path(), OFlag::empty(), Mode::empty()).unwrap();
    let expected_dir = src.to_str().unwrap();

    assert_eq!(readlink(&dst).unwrap().to_str().unwrap(), expected_dir);
    assert_eq!(
        readlinkat(dirfd, "b").unwrap().to_str().unwrap(),
        expected_dir
    );
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux_android {
    use libc::loff_t;
    use std::io::prelude::*;
    use std::io::IoSlice;
    use std::os::unix::prelude::*;

    use nix::fcntl::*;
    use nix::unistd::{close, pipe, read, write};

    use tempfile::tempfile;
    #[cfg(any(target_os = "linux"))]
    use tempfile::NamedTempFile;

    use crate::*;

    /// This test creates a temporary file containing the contents
    /// 'foobarbaz' and uses the `copy_file_range` call to transfer
    /// 3 bytes at offset 3 (`bar`) to another empty file at offset 0. The
    /// resulting file is read and should contain the contents `bar`.
    /// The from_offset should be updated by the call to reflect
    /// the 3 bytes read (6).
    #[test]
    // QEMU does not support copy_file_range. Skip under qemu
    #[cfg_attr(qemu, ignore)]
    fn test_copy_file_range() {
        const CONTENTS: &[u8] = b"foobarbaz";

        let mut tmp1 = tempfile().unwrap();
        let mut tmp2 = tempfile().unwrap();

        tmp1.write_all(CONTENTS).unwrap();
        tmp1.flush().unwrap();

        let mut from_offset: i64 = 3;
        copy_file_range(
            tmp1.as_raw_fd(),
            Some(&mut from_offset),
            tmp2.as_raw_fd(),
            None,
            3,
        )
        .unwrap();

        let mut res: String = String::new();
        tmp2.rewind().unwrap();
        tmp2.read_to_string(&mut res).unwrap();

        assert_eq!(res, String::from("bar"));
        assert_eq!(from_offset, 6);
    }

    #[test]
    fn test_splice() {
        const CONTENTS: &[u8] = b"abcdef123456";
        let mut tmp = tempfile().unwrap();
        tmp.write_all(CONTENTS).unwrap();

        let (rd, wr) = pipe().unwrap();
        let mut offset: loff_t = 5;
        let res = splice(
            tmp.as_raw_fd(),
            Some(&mut offset),
            wr,
            None,
            2,
            SpliceFFlags::empty(),
        )
        .unwrap();

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
        let iovecs = vec![IoSlice::new(&buf1[0..3]), IoSlice::new(&buf2[0..3])];

        let res = vmsplice(wr, &iovecs[..], SpliceFFlags::empty()).unwrap();

        assert_eq!(6, res);

        // Check the bytes can be read at rd.
        let mut buf = [0u8; 32];
        assert_eq!(6, read(rd, &mut buf).unwrap());
        assert_eq!(b"abcdef", &buf[0..6]);

        close(rd).unwrap();
        close(wr).unwrap();
    }

    #[cfg(any(target_os = "linux"))]
    #[test]
    fn test_fallocate() {
        let tmp = NamedTempFile::new().unwrap();

        let fd = tmp.as_raw_fd();
        fallocate(fd, FallocateFlags::empty(), 0, 100).unwrap();

        // Check if we read exactly 100 bytes
        let mut buf = [0u8; 200];
        assert_eq!(100, read(fd, &mut buf).unwrap());
    }

    // The tests below are disabled for the listed targets
    // due to OFD locks not being available in the kernel/libc
    // versions used in the CI environment, probably because
    // they run under QEMU.

    #[test]
    #[cfg(all(target_os = "linux", not(target_env = "musl")))]
    #[cfg_attr(target_env = "uclibc", ignore)] // uclibc doesn't support OFD locks, but the test should still compile
    fn test_ofd_write_lock() {
        use nix::sys::stat::fstat;
        use std::mem;

        let tmp = NamedTempFile::new().unwrap();

        let fd = tmp.as_raw_fd();
        let statfs = nix::sys::statfs::fstatfs(tmp.as_file()).unwrap();
        if statfs.filesystem_type() == nix::sys::statfs::OVERLAYFS_SUPER_MAGIC {
            // OverlayFS is a union file system.  It returns one inode value in
            // stat(2), but a different one shows up in /proc/locks.  So we must
            // skip the test.
            skip!("/proc/locks does not work on overlayfs");
        }
        let inode = fstat(fd).expect("fstat failed").st_ino as usize;

        let mut flock: libc::flock = unsafe {
            mem::zeroed() // required for Linux/mips
        };
        flock.l_type = libc::F_WRLCK as libc::c_short;
        flock.l_whence = libc::SEEK_SET as libc::c_short;
        flock.l_start = 0;
        flock.l_len = 0;
        flock.l_pid = 0;
        fcntl(fd, FcntlArg::F_OFD_SETLKW(&flock)).expect("write lock failed");
        assert_eq!(
            Some(("OFDLCK".to_string(), "WRITE".to_string())),
            lock_info(inode)
        );

        flock.l_type = libc::F_UNLCK as libc::c_short;
        fcntl(fd, FcntlArg::F_OFD_SETLKW(&flock)).expect("write unlock failed");
        assert_eq!(None, lock_info(inode));
    }

    #[test]
    #[cfg(all(target_os = "linux", not(target_env = "musl")))]
    #[cfg_attr(target_env = "uclibc", ignore)] // uclibc doesn't support OFD locks, but the test should still compile
    fn test_ofd_read_lock() {
        use nix::sys::stat::fstat;
        use std::mem;

        let tmp = NamedTempFile::new().unwrap();

        let fd = tmp.as_raw_fd();
        let statfs = nix::sys::statfs::fstatfs(tmp.as_file()).unwrap();
        if statfs.filesystem_type() == nix::sys::statfs::OVERLAYFS_SUPER_MAGIC {
            // OverlayFS is a union file system.  It returns one inode value in
            // stat(2), but a different one shows up in /proc/locks.  So we must
            // skip the test.
            skip!("/proc/locks does not work on overlayfs");
        }
        let inode = fstat(fd).expect("fstat failed").st_ino as usize;

        let mut flock: libc::flock = unsafe {
            mem::zeroed() // required for Linux/mips
        };
        flock.l_type = libc::F_RDLCK as libc::c_short;
        flock.l_whence = libc::SEEK_SET as libc::c_short;
        flock.l_start = 0;
        flock.l_len = 0;
        flock.l_pid = 0;
        fcntl(fd, FcntlArg::F_OFD_SETLKW(&flock)).expect("read lock failed");
        assert_eq!(
            Some(("OFDLCK".to_string(), "READ".to_string())),
            lock_info(inode)
        );

        flock.l_type = libc::F_UNLCK as libc::c_short;
        fcntl(fd, FcntlArg::F_OFD_SETLKW(&flock)).expect("read unlock failed");
        assert_eq!(None, lock_info(inode));
    }

    #[cfg(all(target_os = "linux", not(target_env = "musl")))]
    fn lock_info(inode: usize) -> Option<(String, String)> {
        use std::{fs::File, io::BufReader};

        let file = File::open("/proc/locks").expect("open /proc/locks failed");
        let buf = BufReader::new(file);

        for line in buf.lines() {
            let line = line.unwrap();
            let parts: Vec<_> = line.split_whitespace().collect();
            let lock_type = parts[1];
            let lock_access = parts[3];
            let ino_parts: Vec<_> = parts[5].split(':').collect();
            let ino: usize = ino_parts[2].parse().unwrap();
            if ino == inode {
                return Some((lock_type.to_string(), lock_access.to_string()));
            }
        }
        None
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "wasi",
    target_env = "uclibc",
    target_os = "freebsd"
))]
mod test_posix_fadvise {

    use nix::errno::Errno;
    use nix::fcntl::*;
    use nix::unistd::pipe;
    use std::os::unix::io::{AsRawFd, RawFd};
    use tempfile::NamedTempFile;

    #[test]
    fn test_success() {
        let tmp = NamedTempFile::new().unwrap();
        let fd = tmp.as_raw_fd();
        posix_fadvise(fd, 0, 100, PosixFadviseAdvice::POSIX_FADV_WILLNEED)
            .expect("posix_fadvise failed");
    }

    #[test]
    fn test_errno() {
        let (rd, _wr) = pipe().unwrap();
        let res = posix_fadvise(
            rd as RawFd,
            0,
            100,
            PosixFadviseAdvice::POSIX_FADV_WILLNEED,
        );
        assert_eq!(res, Err(Errno::ESPIPE));
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "wasi",
    target_os = "freebsd"
))]
mod test_posix_fallocate {

    use nix::errno::Errno;
    use nix::fcntl::*;
    use nix::unistd::pipe;
    use std::{
        io::Read,
        os::unix::io::{AsRawFd, RawFd},
    };
    use tempfile::NamedTempFile;

    #[test]
    fn success() {
        const LEN: usize = 100;
        let mut tmp = NamedTempFile::new().unwrap();
        let fd = tmp.as_raw_fd();
        let res = posix_fallocate(fd, 0, LEN as libc::off_t);
        match res {
            Ok(_) => {
                let mut data = [1u8; LEN];
                assert_eq!(tmp.read(&mut data).expect("read failure"), LEN);
                assert_eq!(&data[..], &[0u8; LEN][..]);
            }
            Err(Errno::EINVAL) => {
                // POSIX requires posix_fallocate to return EINVAL both for
                // invalid arguments (i.e. len < 0) and if the operation is not
                // supported by the file system.
                // There's no way to tell for sure whether the file system
                // supports posix_fallocate, so we must pass the test if it
                // returns EINVAL.
            }
            _ => res.unwrap(),
        }
    }

    #[test]
    fn errno() {
        let (rd, _wr) = pipe().unwrap();
        let err = posix_fallocate(rd as RawFd, 0, 100).unwrap_err();
        match err {
            Errno::EINVAL | Errno::ENODEV | Errno::ESPIPE | Errno::EBADF => (),
            errno => panic!("unexpected errno {errno}",),
        }
    }
}
