extern crate nix;

#[cfg(test)]
mod test {
    use nix::sys::stat::stat;


    #[test]
    fn test_stat() {

        use nix::fcntl::open;
        use nix::unistd::{close, unlink};
        use nix::fcntl::O_CREAT;
        use nix::sys::stat::S_IWUSR;

        let filename = b"target/foo.txt";
        let fd_res = open(filename, O_CREAT, S_IWUSR);
        assert!(fd_res.is_ok()); // creating a file in "target" should always work
        let close_res = close(fd_res.ok().unwrap()); // close right here. We use the file only for the stat test
        assert!(close_res.is_ok());
        let stat_result = stat(filename);

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

        let unlink_res = unlink(filename);
        assert!(unlink_res.is_ok());

    }
}
