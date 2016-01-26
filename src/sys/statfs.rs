use {Errno, Result, NixPath};
use std::os::unix::io::AsRawFd;

pub mod vfs {
    #[cfg(target_pointer_width = "32")]
    pub mod hwdep {
        use libc::{c_uint};
        pub type FsType = c_uint;
        pub type BlockSize = c_uint;
        pub type NameLen = c_uint;
        pub type FragmentSize = c_uint;
        pub type SwordType = c_uint;
    }

    #[cfg(target_pointer_width = "64")]
    pub mod hwdep {
        use libc::{c_long};
        pub type FsType = c_long;
        pub type BlockSize = c_long;
        pub type NameLen = c_long;
        pub type FragmentSize = c_long;
        pub type SwordType = c_long;
    }

    use sys::statfs::vfs::hwdep::*;

    #[repr(C)]
    #[derive(Debug,Copy,Clone)]
    pub struct Statfs {
        pub f_type:  FsType,
        pub f_bsize: BlockSize,
        pub f_blocks: u64,
        pub f_bfree: u64,
        pub f_bavail: u64,
        pub f_files: u64,
        pub f_ffree: u64,
        pub f_fsid: u64,
        pub f_namelen: NameLen,
        pub f_frsize: FragmentSize,
        pub f_spare: [SwordType; 5],
    }

    pub const ADFS_SUPER_MAGIC : FsType =      0xadf5;
    pub const AFFS_SUPER_MAGIC : FsType =      0xADFF;
    pub const BEFS_SUPER_MAGIC : FsType =      0x42465331;
    pub const BFS_MAGIC : FsType =             0x1BADFACE;
    pub const CIFS_MAGIC_NUMBER : FsType =     0xFF534D42;
    pub const CODA_SUPER_MAGIC : FsType =      0x73757245;
    pub const COH_SUPER_MAGIC : FsType =       0x012FF7B7;
    pub const CRAMFS_MAGIC : FsType =          0x28cd3d45;
    pub const DEVFS_SUPER_MAGIC : FsType =     0x1373;
    pub const EFS_SUPER_MAGIC : FsType =       0x00414A53;
    pub const EXT_SUPER_MAGIC : FsType =       0x137D;
    pub const EXT2_OLD_SUPER_MAGIC : FsType =  0xEF51;
    pub const EXT2_SUPER_MAGIC : FsType =      0xEF53;
    pub const EXT3_SUPER_MAGIC : FsType =      0xEF53;
    pub const EXT4_SUPER_MAGIC : FsType =      0xEF53;
    pub const HFS_SUPER_MAGIC : FsType =       0x4244;
    pub const HPFS_SUPER_MAGIC : FsType =      0xF995E849;
    pub const HUGETLBFS_MAGIC : FsType =       0x958458f6;
    pub const ISOFS_SUPER_MAGIC : FsType =     0x9660;
    pub const JFFS2_SUPER_MAGIC : FsType =     0x72b6;
    pub const JFS_SUPER_MAGIC : FsType =       0x3153464a;
    pub const MINIX_SUPER_MAGIC : FsType =     0x137F; /* orig. minix */
    pub const MINIX_SUPER_MAGIC2 : FsType =    0x138F; /* 30 char minix */
    pub const MINIX2_SUPER_MAGIC : FsType =    0x2468; /* minix V2 */
    pub const MINIX2_SUPER_MAGIC2 : FsType =   0x2478; /* minix V2, 30 char names */
    pub const MSDOS_SUPER_MAGIC : FsType =     0x4d44;
    pub const NCP_SUPER_MAGIC : FsType =       0x564c;
    pub const NFS_SUPER_MAGIC : FsType =       0x6969;
    pub const NTFS_SB_MAGIC : FsType =         0x5346544e;
    pub const OPENPROM_SUPER_MAGIC : FsType =  0x9fa1;
    pub const PROC_SUPER_MAGIC : FsType =      0x9fa0;
    pub const QNX4_SUPER_MAGIC : FsType =      0x002f;
    pub const REISERFS_SUPER_MAGIC : FsType =  0x52654973;
    pub const ROMFS_MAGIC : FsType =           0x7275;
    pub const SMB_SUPER_MAGIC : FsType =       0x517B;
    pub const SYSV2_SUPER_MAGIC : FsType =     0x012FF7B6;
    pub const SYSV4_SUPER_MAGIC : FsType =     0x012FF7B5;
    pub const TMPFS_MAGIC : FsType =           0x01021994;
    pub const UDF_SUPER_MAGIC : FsType =       0x15013346;
    pub const UFS_MAGIC : FsType =             0x00011954;
    pub const USBDEVICE_SUPER_MAGIC : FsType = 0x9fa2;
    pub const VXFS_SUPER_MAGIC : FsType =      0xa501FCF5;
    pub const XENIX_SUPER_MAGIC : FsType =     0x012FF7B4;
    pub const XFS_SUPER_MAGIC : FsType =       0x58465342;
    pub const _XIAFS_SUPER_MAGIC : FsType =    0x012FD16D;
}

mod ffi {
    use libc::{c_int,c_char};
    use sys::statfs::vfs;

    extern {
        pub fn statfs(path: * const c_char, buf: *mut vfs::Statfs) -> c_int;
        pub fn fstatfs(fd: c_int, buf: *mut vfs::Statfs) -> c_int;
    }
}

pub fn statfs<P: ?Sized + NixPath>(path: &P, stat: &mut vfs::Statfs) -> Result<()> {
    unsafe {
        Errno::clear();
        let res = try!(
            path.with_nix_path(|path| ffi::statfs(path.as_ptr(), stat))
        );

        Errno::result(res).map(drop)
    }
}

pub fn fstatfs<T: AsRawFd>(fd: &T, stat: &mut vfs::Statfs) -> Result<()> {
    unsafe {
        Errno::clear();
        Errno::result(ffi::fstatfs(fd.as_raw_fd(), stat)).map(drop)
    }
}
