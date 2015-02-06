#![allow(unstable)]

extern crate nix;

#[cfg(test)]
mod test {
    use nix::sys::stat::{stat, fstat};


    #[test]
    fn test_stat() {

        use std::old_io::{File, Open, ReadWrite};
        use std::old_io::fs::{unlink};
        use std::old_path::posix::Path as OldPath; // temporary until new File type is available that takes std::path::Path
        use std::path::Path as NewPath;

        let filename = "target/foo.txt";
        let old_io_path = OldPath::new(filename);  // for creating File

        let test_file = File::create(&old_io_path);


        let stat_result = stat(NewPath::new(filename));

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

        unlink(&old_io_path);
    }
}
