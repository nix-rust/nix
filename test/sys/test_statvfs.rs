use nix::sys::statvfs::*;
use std::fs::File;

#[test]
#[cfg_attr(miri, ignore)]
fn statvfs_call() {
    statvfs(&b"/"[..]).unwrap();
}

#[test]
#[cfg_attr(miri, ignore)]
fn fstatvfs_call() {
    let root = File::open("/").unwrap();
    fstatvfs(&root).unwrap();
}
