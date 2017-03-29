use std::fs::File;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::os::unix::prelude::AsRawFd;

use libc::{S_IFMT, S_IFLNK};

use nix::fcntl;
use nix::sys::stat::*;
use nix::Result;
use tempdir::TempDir;
use tempfile::NamedTempFile;

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
    match stat_result {
        Ok(stats) => {
            assert!(stats.st_dev > 0);      // must be positive integer, exact number machine dependent
            assert!(stats.st_ino > 0);      // inode is positive integer, exact number machine dependent
            assert!(stats.st_mode > 0);     // must be positive integer
            assert!(stats.st_nlink == 1);   // there links created, must be 1
            assert!(valid_uid_gid(stats));  // must be positive integers
            assert!(stats.st_size == 0);    // size is 0 because we did not write anything to the file
            assert!(stats.st_blksize > 0);  // must be positive integer, exact number machine dependent
            assert!(stats.st_blocks <= 16);  // Up to 16 blocks can be allocated for a blank file
        }
        Err(_) => panic!("stat call failed") // if stats system call fails, something is seriously wrong on that machine
    }
}

fn assert_lstat_results(stat_result: Result<FileStat>) {
    match stat_result {
        Ok(stats) => {
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
        Err(_) => panic!("stat call failed") // if stats system call fails, something is seriously wrong on that machine
    }
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
    let tempdir = TempDir::new("nix-test_stat_and_fstat").unwrap();
    let filename = tempdir.path().join("foo.txt");
    File::create(&filename).unwrap();
    let dirfd = fcntl::open(tempdir.path(),
                            fcntl::OFlag::empty(),
                            Mode::empty());

    let result = fstatat(dirfd.unwrap(),
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
    let stat_result = lstat(&filename);
    assert_stat_results(stat_result);

    let lstat_result = lstat(&linkname);
    assert_lstat_results(lstat_result);

    let fstat_result = fstat(link.as_raw_fd());
    assert_stat_results(fstat_result);
}

fn assert_mode(f: &NamedTempFile, mode: u32) {
    assert_eq!(f.metadata().unwrap().permissions().mode(), 
               mode);
}

#[test]
fn test_chmod() {
    let tempfile = NamedTempFile::new().unwrap();
    chmod(tempfile.path(),
          Mode::from_bits(0o755).unwrap()).unwrap();
    assert_mode(&tempfile, 0o755);

    fchmod(tempfile.as_raw_fd(),
          Mode::from_bits(0o644).unwrap()).unwrap();
    assert_mode(&tempfile, 0o644);

    let parent_dir = tempfile.path().parent().unwrap();
    let dirfd = fcntl::open(parent_dir,
                            fcntl::OFlag::empty(),
                            Mode::empty()).unwrap();
    fchmodat(dirfd,
             tempfile.path().file_name().unwrap(),
             Mode::from_bits(0o600).unwrap(),
             fcntl::AtFlags::empty()).unwrap();
    assert_mode(&tempfile, 0o600);
}
