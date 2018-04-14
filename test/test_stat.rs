use std::fs::File;
use std::os::unix::fs::symlink;
use std::os::unix::prelude::AsRawFd;

use libc::{S_IFMT, S_IFLNK};

use nix::fcntl;
use nix::sys::stat::{self, fchmod, fchmodat, fstat, lstat, stat};
use nix::sys::stat::{FileStat, Mode, FchmodatFlags};
use nix::unistd::chdir;
use nix::Result;
use tempdir::TempDir;

#[allow(unused_comparisons)]
// uid and gid are signed on Windows, but not on other platforms. This function
// allows warning free compiles on all platforms, and can be removed when
// expression-level #[allow] is available.
fn valid_uid_gid(stat: FileStat) -> bool {
    // uid could be 0 for the `root` user. This quite possible when
    // the tests are being run on a rooted Android device.
    stat.st_uid >= 0 && stat.st_gid >= 0
}

fn assert_stat_results(stat_result: Result<FileStat>) {
    let stats = stat_result.expect("stat call failed");
    assert!(stats.st_dev > 0);      // must be positive integer, exact number machine dependent
    assert!(stats.st_ino > 0);      // inode is positive integer, exact number machine dependent
    assert!(stats.st_mode > 0);     // must be positive integer
    assert!(stats.st_nlink == 1);   // there links created, must be 1
    assert!(valid_uid_gid(stats));  // must be positive integers
    assert!(stats.st_size == 0);    // size is 0 because we did not write anything to the file
    assert!(stats.st_blksize > 0);  // must be positive integer, exact number machine dependent
    assert!(stats.st_blocks <= 16);  // Up to 16 blocks can be allocated for a blank file
}

fn assert_lstat_results(stat_result: Result<FileStat>) {
    let stats = stat_result.expect("stat call failed");
    assert!(stats.st_dev > 0);      // must be positive integer, exact number machine dependent
    assert!(stats.st_ino > 0);      // inode is positive integer, exact number machine dependent
    assert!(stats.st_mode > 0);     // must be positive integer

    // st_mode is c_uint (u32 on Android) while S_IFMT is mode_t
    // (u16 on Android), and that will be a compile error.
    // On other platforms they are the same (either both are u16 or u32).
    assert!((stats.st_mode as usize) & (S_IFMT as usize) == S_IFLNK as usize); // should be a link
    assert!(stats.st_nlink == 1);   // there links created, must be 1
    assert!(valid_uid_gid(stats));  // must be positive integers
    assert!(stats.st_size > 0);    // size is > 0 because it points to another file
    assert!(stats.st_blksize > 0);  // must be positive integer, exact number machine dependent

    // st_blocks depends on whether the machine's file system uses fast
    // or slow symlinks, so just make sure it's not negative
    // (Android's st_blocks is ulonglong which is always non-negative.)
    assert!(stats.st_blocks >= 0);
}

#[test]
fn test_stat_and_fstat() {
    let tempdir = TempDir::new("nix-test_stat_and_fstat").unwrap();
    let filename = tempdir.path().join("foo.txt");
    let file = File::create(&filename).unwrap();

    let stat_result = stat(&filename);
    assert_stat_results(stat_result);

    let fstat_result = fstat(file.as_raw_fd());
    assert_stat_results(fstat_result);
}

#[test]
fn test_fstatat() {
    let tempdir = TempDir::new("nix-test_fstatat").unwrap();
    let filename = tempdir.path().join("foo.txt");
    File::create(&filename).unwrap();
    let dirfd = fcntl::open(tempdir.path(),
                            fcntl::OFlag::empty(),
                            stat::Mode::empty());

    let result = stat::fstatat(dirfd.unwrap(),
                               &filename,
                               fcntl::AtFlags::empty());
    assert_stat_results(result);
}

#[test]
fn test_stat_fstat_lstat() {
    let tempdir = TempDir::new("nix-test_stat_fstat_lstat").unwrap();
    let filename = tempdir.path().join("bar.txt");
    let linkname = tempdir.path().join("barlink");

    File::create(&filename).unwrap();
    symlink("bar.txt", &linkname).unwrap();
    let link = File::open(&linkname).unwrap();

    // should be the same result as calling stat,
    // since it's a regular file
    let stat_result = stat(&filename);
    assert_stat_results(stat_result);

    let lstat_result = lstat(&linkname);
    assert_lstat_results(lstat_result);

    let fstat_result = fstat(link.as_raw_fd());
    assert_stat_results(fstat_result);
}

#[test]
fn test_fchmod() {
    let tempdir = TempDir::new("nix-test_fchmod").unwrap();
    let filename = tempdir.path().join("foo.txt");
    let file = File::create(&filename).unwrap();

    let mut mode1 = Mode::empty();
    mode1.insert(Mode::S_IRUSR);
    mode1.insert(Mode::S_IWUSR);
    fchmod(file.as_raw_fd(), mode1).unwrap();

    let file_stat1 = stat(&filename).unwrap();
    assert_eq!(file_stat1.st_mode & 0o7777, mode1.bits());

    let mut mode2 = Mode::empty();
    mode2.insert(Mode::S_IROTH);
    fchmod(file.as_raw_fd(), mode2).unwrap();

    let file_stat2 = stat(&filename).unwrap();
    assert_eq!(file_stat2.st_mode & 0o7777, mode2.bits());
}

#[test]
fn test_fchmodat() {
    let tempdir = TempDir::new("nix-test_fchmodat").unwrap();
    let filename = "foo.txt";
    let fullpath = tempdir.path().join(filename);
    File::create(&fullpath).unwrap();

    let dirfd = fcntl::open(tempdir.path(), fcntl::OFlag::empty(), stat::Mode::empty()).unwrap();

    let mut mode1 = Mode::empty();
    mode1.insert(Mode::S_IRUSR);
    mode1.insert(Mode::S_IWUSR);
    fchmodat(Some(dirfd), filename, mode1, FchmodatFlags::FollowSymlink).unwrap();

    let file_stat1 = stat(&fullpath).unwrap();
    assert_eq!(file_stat1.st_mode & 0o7777, mode1.bits());

    chdir(tempdir.path()).unwrap();

    let mut mode2 = Mode::empty();
    mode2.insert(Mode::S_IROTH);
    fchmodat(None, filename, mode2, FchmodatFlags::FollowSymlink).unwrap();

    let file_stat2 = stat(&fullpath).unwrap();
    assert_eq!(file_stat2.st_mode & 0o7777, mode2.bits());
}
