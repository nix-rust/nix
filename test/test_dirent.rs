use libc;
use nix::dirent::{self, opendir, readdir, seekdir, telldir, DirectoryStream};
use std::path::Path;
use tempdir::TempDir;

fn test_readdir<Open>(open_fn: Open)
    where Open: Fn(&Path) -> DirectoryStream
{
    let tempdir = TempDir::new("nix-test_readdir")
        .unwrap_or_else(|e| panic!("tempdir failed: {}", e));
    let mut dir = open_fn(tempdir.path());
    let first_inode = readdir(&mut dir)
        .unwrap()
        .unwrap()
        .inode();

    let pos = telldir(&mut dir);

    let second_inode = readdir(&mut dir)
        .unwrap()
        .unwrap()
        .inode();
    seekdir(&mut dir, pos);

    let second_inode_again = readdir(&mut dir)
        .unwrap()
        .unwrap()
        .inode();

    assert_ne!(first_inode, second_inode);
    assert_eq!(second_inode, second_inode_again);

    // end of directory
    assert!(readdir(&mut dir).unwrap().is_none());

    unsafe { libc::closedir(dir.into()) };
}

#[test]
fn test_opendir() {
    test_readdir(|path| opendir(path).unwrap());
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[test]
fn test_fdopendir() {
    use std::os::unix::io::IntoRawFd;
    use std::fs::File;
    test_readdir(|path| dirent::fdopendir(File::open(path).unwrap().into_raw_fd()).unwrap());
}
