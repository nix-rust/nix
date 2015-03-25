use nix::sys::stat::{stat, fstat};

use nix::fcntl::open;
use nix::unistd::{close, unlink};
use nix::fcntl::O_CREAT;
use nix::sys::stat::{FileStat, S_IWUSR};
use nix::Result;

fn assert_stat_results(stat_result: Result<FileStat>) {
    match stat_result {
        Ok(stats) => {
            assert!(stats.st_dev > 0);      // must be positive integer, exact number machine dependent
            assert!(stats.st_ino > 0);      // inode is positive integer, exact number machine dependent
            assert!(stats.st_mode > 0);     // must be positive integer
            assert!(stats.st_nlink == 1);   // there links created, must be 1
            assert!(stats.st_uid > 0);      // must be positive integer
            assert!(stats.st_gid > 0);      // must be positive integer
            assert!(stats.st_rdev == 0);    // no special device
            assert!(stats.st_size == 0);    // size is 0 because we did not write anything to the file
            assert!(stats.st_blksize > 0);  // must be positive integer, exact number machine dependent
            assert!(stats.st_blocks == 0);  // no blocks allocated because file size is 0
        }
        Err(_) => panic!("stat call failed") // if stats system call fails, something is seriously wrong on that machine
    }
}


#[test]
fn test_stat_and_fstat() {
    let filename = b"target/foo.txt".as_slice();
    let fd = open(filename, O_CREAT, S_IWUSR).unwrap();  // create empty file

    let stat_result = stat(filename);
    assert_stat_results(stat_result);

    let fstat_result = fstat(fd);
    assert_stat_results(fstat_result);

    close(fd).unwrap();
    unlink(filename).unwrap();
}
