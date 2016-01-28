//! FFI for statvfs functions
//!
//! See the `vfs::Statvfs` struct for some rusty wrappers

use {Errno, Result, NixPath};
use std::os::unix::io::AsRawFd;

pub mod vfs {
    //! Structs related to the `statvfs` and `fstatvfs` functions
    //!
    //! The `Statvfs` struct has some wrappers methods around the `statvfs` and
    //! `fstatvfs` calls.

    use libc::{c_ulong,c_int};
    use std::os::unix::io::AsRawFd;
    use {Result, NixPath};

    use super::{statvfs, fstatvfs};

    bitflags!(
        /// Mount Flags
        #[repr(C)]
        #[derive(Default)]
        flags FsFlags: c_ulong {
            /// Read Only
            const RDONLY = 1,
            /// Do not allow the set-uid bits to have an effect
            const NOSUID = 2,
            /// Do not interpret character or block-special devices
            const NODEV  = 4,
            /// Do not allow execution of binaries on the filesystem
            const NOEXEC = 8,
            /// All IO should be done synchronously
            const SYNCHRONOUS  = 16,
            /// Allow mandatory locks on the filesystem
            const MANDLOCK = 64,
            const WRITE = 128,
            const APPEND = 256,
            const IMMUTABLE = 512,
            /// Do not update access times on files
            const NOATIME = 1024,
            /// Do not update access times on files
            const NODIRATIME = 2048,
            /// Update access time relative to modify/change time
            const RELATIME = 4096,
        }
    );

    /// The posix statvfs struct
    ///
    /// http://linux.die.net/man/2/statvfs
    #[repr(C)]
    #[derive(Debug,Copy,Clone)]
    pub struct Statvfs {
        /// Filesystem block size. This is the value that will lead to
        /// most efficient use of the filesystem
        pub f_bsize: c_ulong,
        /// Fragment Size -- actual minimum unit of allocation on this
        /// filesystem
        pub f_frsize: c_ulong,
        /// Total number of blocks on the filesystem
        pub f_blocks: u64,
        /// Number of unused blocks on the filesystem, including those
        /// reserved for root
        pub f_bfree: u64,
        /// Number of blocks available to non-root users
        pub f_bavail: u64,
        /// Total number of inodes available on the filesystem
        pub f_files: u64,
        /// Number of inodes available on the filesystem
        pub f_ffree: u64,
        /// Number of inodes available to non-root users
        pub f_favail: u64,
        /// File System ID
        pub f_fsid: c_ulong,
        /// Mount Flags
        pub f_flag: FsFlags,
        /// Maximum filename length
        pub f_namemax: c_ulong,
        /// Reserved extra space, OS-dependent
        f_spare: [c_int; 6],
    }

    impl Statvfs {
        /// Create a new `Statvfs` object and fill it with information about
        /// the mount that contains `path`
        pub fn for_path<P: ?Sized + NixPath>(path: &P) -> Result<Statvfs> {
            let mut stat = Statvfs::default();
            let res = statvfs(path, &mut stat);
            res.map(|_| stat)
        }

        /// Replace information in this struct with information about `path`
        pub fn update_with_path<P: ?Sized + NixPath>(&mut self, path: &P) -> Result<()> {
            statvfs(path, self)
        }

        /// Create a new `Statvfs` object and fill it with information from fd
        pub fn for_fd<T: AsRawFd>(fd: &T) -> Result<Statvfs> {
            let mut stat = Statvfs::default();
            let res = fstatvfs(fd, &mut stat);
            res.map(|_| stat)
        }

        /// Replace information in this struct with information about `fd`
        pub fn update_with_fd<T: AsRawFd>(&mut self, fd: &T) -> Result<()> {
            fstatvfs(fd, self)
        }
    }

    impl Default for Statvfs {
        /// Create a statvfs object initialized to all zeros
        fn default() -> Self {
            Statvfs {
                f_bsize: 0,
                f_frsize: 0,
                f_blocks: 0,
                f_bfree: 0,
                f_bavail: 0,
                f_files: 0,
                f_ffree: 0,
                f_favail: 0,
                f_fsid: 0,
                f_flag: FsFlags::default(),
                f_namemax: 0,
                f_spare: [0, 0, 0, 0, 0, 0],
            }
        }
    }
}

mod ffi {
    use libc::{c_char, c_int};
    use sys::statvfs::vfs;

    extern {
        pub fn statvfs(path: * const c_char, buf: *mut vfs::Statvfs) -> c_int;
        pub fn fstatvfs(fd: c_int, buf: *mut vfs::Statvfs) -> c_int;
    }
}

/// Fill an existing `Statvfs` object with information about the `path`
pub fn statvfs<P: ?Sized + NixPath>(path: &P, stat: &mut vfs::Statvfs) -> Result<()> {
    unsafe {
        Errno::clear();
        let res = try!(
            path.with_nix_path(|path| ffi::statvfs(path.as_ptr(), stat))
        );

        Errno::result(res).map(drop)
    }
}

/// Fill an existing `Statvfs` object with information about `fd`
pub fn fstatvfs<T: AsRawFd>(fd: &T, stat: &mut vfs::Statvfs) -> Result<()> {
    unsafe {
        Errno::clear();
        Errno::result(ffi::fstatvfs(fd.as_raw_fd(), stat)).map(drop)
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use sys::statvfs::*;

    #[test]
    fn statvfs_call() {
        let mut stat = vfs::Statvfs::default();
        statvfs("/".as_bytes(), &mut stat).unwrap()
    }

    #[test]
    fn fstatvfs_call() {
        let mut stat = vfs::Statvfs::default();
        let root = File::open("/").unwrap();
        fstatvfs(&root, &mut stat).unwrap()
    }
}
