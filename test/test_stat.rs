use std::fs;
use std::str;

use libc::consts::os::posix88;

use nix::sys::stat::{stat, fstat, lstat};

use nix::fcntl::open;
use nix::unistd::{close, unlink};
use nix::fcntl::{O_CREAT, O_RDONLY};
use nix::sys::stat::{FileStat, S_IWUSR, S_IRUSR};
use nix::Result;

fn assert_stat_results(stat_result: Result<FileStat>) {
    match stat_result {
        Ok(stats) => {
            assert!(stats.st_dev > 0);      // must be positive integer, exact number machine dependent
            assert!(stats.st_ino > 0);      // inode is positive integer, exact number machine dependent
            assert!(stats.st_mode > 0);     // must be positive integer
            assert!(stats.st_nlink == 1);   // there links created, must be 1
            // uid could be 0 for the `root` user. This quite possible when
            // the tests are being run on a rooted Android device.
            assert!(stats.st_uid >= 0);      // must be positive integer
            assert!(stats.st_gid >= 0);      // must be positive integer
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
            assert!((stats.st_mode as usize) & (posix88::S_IFMT as usize)
                    == posix88::S_IFLNK as usize); // should be a link
            assert!(stats.st_nlink == 1);   // there links created, must be 1
            // uid could be 0 for the `root` user. This quite possible when
            // the tests are being run on a rooted Android device.
            assert!(stats.st_uid >= 0);      // must be positive integer
            assert!(stats.st_gid >= 0);      // must be positive integer
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
    let filename = b"target/foo.txt".as_ref();
    let fd = open(filename, O_CREAT, S_IWUSR).unwrap();  // create empty file

    let stat_result = stat(filename);
    assert_stat_results(stat_result);

    let fstat_result = fstat(fd);
    assert_stat_results(fstat_result);

    close(fd).unwrap();
    unlink(filename).unwrap();
}

#[test]
fn test_stat_fstat_lstat() {
    let filename = b"target/bar.txt".as_ref();
    let linkname = b"target/barlink".as_ref();

    open(filename, O_CREAT, S_IWUSR | S_IRUSR).unwrap();  // create empty file
    fs::soft_link("bar.txt", str::from_utf8(linkname).unwrap()).unwrap();
    let fd = open(linkname, O_RDONLY, S_IRUSR).unwrap();

    // should be the same result as calling stat,
    // since it's a regular file
    let stat_result = lstat(filename);
    assert_stat_results(stat_result);

    let lstat_result = lstat(linkname);
    assert_lstat_results(lstat_result);

    let fstat_result = fstat(fd);
    assert_stat_results(fstat_result);

    close(fd).unwrap();
    unlink(linkname).unwrap();
    unlink(filename).unwrap();
}
