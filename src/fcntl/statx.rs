use std::{mem, os::unix::prelude::RawFd};

use libc::{self, c_int, c_uint, mode_t};

use crate::{errno::Errno, NixPath, Result, sys::stat::{SFlag, Mode}, unistd::{Uid, Gid}, sys::time::TimeSpec};

libc_bitflags!(
    /// Configuration options for statx.
    pub struct Flags: c_int {
        /// Allow empty `pathname`, to receive status of the specified file descriptor
        /// itself (not a directory entry) instead.
        AT_EMPTY_PATH;

        /// Do not automount the last component of the path being queried.
        AT_NO_AUTOMOUNT;

        /// Do not follow symlink in the last component of the path being queried.
        AT_SYMLINK_NOFOLLOW;

        /// Do whatever usual, non-`x` stat does. The default.
        AT_STATX_SYNC_AS_STAT;

        /// Force returned attributes to be up to date (synchonized).
        /// May trigger data writeback.
        /// May involve additional roudtrips for networked filesystems.
        AT_STATX_FORCE_SYNC;

        /// Don't synchronize, use cached data if possible. This may lead to the
        /// returned data to be approximate.
        AT_STATX_DONT_SYNC;
    }
);

libc_bitflags!(
    /// Configuration options for statx.
    pub struct Mask: c_uint {
        /// Want stx_mode & S_IFMT
        STATX_TYPE;
        /// Want stx_mode & ~S_IFMT
        STATX_MODE;
        /// Want stx_nlink
        STATX_NLINK;
        /// Want stx_uid
        STATX_UID;
        /// Want stx_gid
        STATX_GID;
        /// Want stx_atime
        STATX_ATIME;
        /// Want stx_mtime
        STATX_MTIME;
        /// Want stx_ctime
        STATX_CTIME;
        /// Want stx_ino
        STATX_INO;
        /// Want stx_size
        STATX_SIZE;
        /// Want stx_blocks
        STATX_BLOCKS;
        /// [All of the above]
        STATX_BASIC_STATS;
        /// Want stx_btime
        STATX_BTIME;
        /// Want stx_mnt_id (since Linux 5.8);
        STATX_MNT_ID;
        /// [All currently available fields]
        STATX_ALL;
    }
);

/// Attempt to retrieve stats of `pathname`. If `pathname` is relative, `dirfd`
/// is used as starting directory for lookups. If `dirfd` is None, current
/// directory is used.
/// `pathname` may be empty string. But instead of specifying empty string
/// literal, you are adviced to use zero-terminated `CStr` to avoid extra allocation 
/// `mask` determines what stats entries should be filled in, but kernel
/// can actually fill more or less than requested.
/// `statx` is not atomic: if attributes are being changed at the time of
/// `statx` is called, the returned attributes set may have items from
/// different moments of time.
pub fn statx<P: ?Sized + NixPath>(
    dirfd: Option<RawFd>,
    pathname: &P,
    flags: Flags,
    mask: Mask,
) -> Result<Statx> {
    let dirfd = dirfd.unwrap_or(libc::AT_FDCWD);

    let mut dst = mem::MaybeUninit::uninit();
    let res = pathname.with_nix_path(|cstr| unsafe {
        libc::statx(
            dirfd,
            cstr.as_ptr(),
            flags.bits() as libc::c_int,
            mask.bits() as libc::c_uint,
            dst.as_mut_ptr(),
        )
    })?;

    Errno::result(res)?;

    Ok(Statx ( unsafe { dst.assume_init() } ))
}

#[derive(Debug,Copy,Clone)]
pub struct Statx (pub libc::statx);

impl From<libc::statx_timestamp> for TimeSpec {
    fn from(x: libc::statx_timestamp) -> Self {
        TimeSpec::from_timespec(libc::timespec {
            tv_sec: x.tv_sec,
            tv_nsec: x.tv_nsec.into(),
        })
    }
}

impl Statx {
    /// Retrieve file type, if it has been returned by kernel
    pub fn filetype(&self) -> Option<SFlag> {
        if Mask::STATX_TYPE.bits() & self.0.stx_mask > 0 {
            Some(SFlag::from_bits_truncate(self.0.stx_mode as mode_t & SFlag::S_IFMT.bits()))
        } else {
            None
        }
    }
    
    /// Retrieve file mode, if it has been returned by kernel.
    pub fn mode(&self) -> Option<Mode> {
        if Mask::STATX_MODE.bits() & self.0.stx_mask > 0 {
            Some(Mode::from_bits_truncate(self.0.stx_mode as mode_t))
        } else {
            None
        }
    }

    /// Retrieve number of hard links, if it has been returned by kernel.
    pub fn nlinks(&self) -> Option<u32> {
        if Mask::STATX_NLINK.bits() & self.0.stx_mask > 0 {
            Some(self.0.stx_nlink)
        } else {
            None
        }
    }

    /// Retrieve uid (owner, user ID), if it has been returned by kernel.
    pub fn uid(&self) -> Option<Uid> {
        if Mask::STATX_UID.bits() & self.0.stx_mask > 0 {
            Some(Uid::from_raw(self.0.stx_uid))
        } else {
            None
        }
    }

    /// Retrieve gid (group ID), if it has been returned by kernel.
    pub fn gid(&self) -> Option<Gid> {
        if Mask::STATX_GID.bits() & self.0.stx_mask > 0 {
            Some(Gid::from_raw(self.0.stx_uid))
        } else {
            None
        }
    }

    /// Retrieve file access time, if it has been returned by kernel
    pub fn atime(&self) -> Option<TimeSpec> {
        if Mask::STATX_ATIME.bits() & self.0.stx_mask > 0 {
            Some(self.0.stx_atime.into())
        } else {
            None
        }
    }

    /// Retrieve file modification time, if it has been returned by kernel
    pub fn mtime(&self) -> Option<TimeSpec> {
        if Mask::STATX_MTIME.bits() & self.0.stx_mask > 0 {
            Some(self.0.stx_mtime.into())
        } else {
            None
        }
    }

    /// Retrieve file attribute change time, if it has been returned by kernel
    pub fn ctime(&self) -> Option<TimeSpec> {
        if Mask::STATX_CTIME.bits() & self.0.stx_mask > 0 {
            Some(self.0.stx_ctime.into())
        } else {
            None
        }
    }

    /// Retrieve file birth (creation) time, if it has been returned by kernel
    pub fn btime(&self) -> Option<TimeSpec> {
        if Mask::STATX_BTIME.bits() & self.0.stx_mask > 0 {
            Some(self.0.stx_btime.into())
        } else {
            None
        }
    }

    /// Retrieve inode number, if it has been returned by kernel
    pub fn ino(&self) -> Option<u64> {
        if Mask::STATX_INO.bits() & self.0.stx_mask > 0 {
            Some(self.0.stx_ino)
        } else {
            None
        }
    }

    /// Retrieve file size, in bytes, if it has been returned by kernel
    pub fn size(&self) -> Option<u64> {
        if Mask::STATX_SIZE.bits() & self.0.stx_mask > 0 {
            Some(self.0.stx_size)
        } else {
            None
        }
    }

    /// Retrieve file size as a number of blocks, if it has been returned by kernel
    pub fn blocks(&self) -> Option<u64> {
        if Mask::STATX_BLOCKS.bits() & self.0.stx_mask > 0 {
            Some(self.0.stx_blocks)
        } else {
            None
        }
    }

    /// Retrieve size of the block (used in `blocks` function) in bytes
    pub fn blksize(&self) -> Option<u32> {
        // I'm not sure what exact mask bit should be used to check presence of block size.
        // Actual Linux kernel seems return most of STATX_BASIC_STATS
        // in one go, regarless of which things were asked for.
        if Mask::STATX_BASIC_STATS.bits() & self.0.stx_mask == Mask::STATX_BASIC_STATS.bits()  {
            Some(self.0.stx_blksize)
        } else {
            None
        }
    }

    /// Retrieve mount ID, if it has been returned by kernel
    pub fn mnt_id(&self) -> Option<u64> {
        if Mask::STATX_MNT_ID.bits() & self.0.stx_mask > 0 {
            Some(self.0.stx_mnt_id)
        } else {
            None
        }
    }

    /// Retrieve device major and minor numbers (first and second elements of the
    /// tuple respectively) of the filesystem where the file resides at, if it has been returned by kernel.
    pub fn dev_major_minor(&self) -> Option<(u32, u32)> {
        // I'm not sure what exact mask bit should be used to check presence of this information.
        // Actual Linux kernel seems return most of STATX_BASIC_STATS in one go,
        // regarless of which things were asked for.
        if Mask::STATX_BASIC_STATS.bits() & self.0.stx_mask == Mask::STATX_BASIC_STATS.bits() {
            Some((self.0.stx_dev_major, self.0.stx_dev_minor))
        } else {
            None
        }
    }

    /// Retrieve pointed-to device major and minor numbers (first and second
    /// elements of the tuple respectively) of this character or block device
    /// file, if it has been returned by kernel.
    /// Note that this function does not check for the file type.
    /// It would return `Some` even for regular files.
    pub fn rdev_major_minor(&self) -> Option<(u32, u32)> {
        // I'm not sure what exact mask bit should be used to check presence of this information.
        // Actual Linux kernel seems return most of STATX_BASIC_STATS
        // in one go, regarless of which things were asked for.
        if Mask::STATX_BASIC_STATS.bits() & self.0.stx_mask == Mask::STATX_BASIC_STATS.bits() {
            Some((self.0.stx_rdev_major, self.0.stx_rdev_minor))
        } else {
            None
        }
    }

    /// Determine if the file is compressed. None means kernel does not
    /// indicate this attrbiute is supported by the filesystem
    pub fn compressed(&self) -> Option<bool> {
        if self.0.stx_attributes_mask & libc::STATX_ATTR_COMPRESSED as u64 > 0 {
            Some(self.0.stx_attributes & libc::STATX_ATTR_COMPRESSED as u64 > 0)
        } else {
            None
        }
    }

    /// Determine if the file is immutable. None means kernel does not indicate this
    /// attrbiute is supported by the filesystem
    pub fn immutable(&self) -> Option<bool> {
        if self.0.stx_attributes_mask & libc::STATX_ATTR_IMMUTABLE as u64 > 0 {
            Some(self.0.stx_attributes & libc::STATX_ATTR_IMMUTABLE as u64 > 0)
        } else {
            None
        }
    }
    
    /// Determine if the file is append-only. None means kernel does not indicate
    /// this attrbiute is supported by the filesystem
    pub fn append_only(&self) -> Option<bool> {
        if self.0.stx_attributes_mask & libc::STATX_ATTR_APPEND as u64 > 0 {
            Some(self.0.stx_attributes & libc::STATX_ATTR_APPEND as u64 > 0)
        } else {
            None
        }
    }

    /// Determine if the file is not a candidate for a backup. None means kernel
    /// does not indicate this attrbiute is supported by the filesystem
    pub fn nodump(&self) -> Option<bool> {
        if self.0.stx_attributes_mask & libc::STATX_ATTR_NODUMP as u64 > 0 {
            Some(self.0.stx_attributes & libc::STATX_ATTR_NODUMP as u64 > 0)
        } else {
            None
        }
    }

    /// Determine if the file requires a key to be encrypted(?).
    /// None means kernel does not indicate this attrbiute is supported by the filesystem
    pub fn encrypted(&self) -> Option<bool> {
        if self.0.stx_attributes_mask & libc::STATX_ATTR_ENCRYPTED as u64 > 0 {
            Some(self.0.stx_attributes & libc::STATX_ATTR_ENCRYPTED as u64 > 0)
        } else {
            None
        }
    }

    /*
    /// Determine if the file has fs-verify enabled. None means kernel does not
    /// indicate this attrbiute is supported by the filesystem
    pub fn verify_enabled(&self) -> Option<bool> {
        if self.0.stx_attributes_mask & libc::STATX_ATTR_VERITY as u64 > 0 {
            Some(self.0.stx_attributes & libc::STATX_ATTR_VERITY as u64 > 0)
        } else {
            None
        }
    }

    /// Determine if the file is in CPU direct access state. None means kernel
    /// does not indicate this attrbiute is supported by the filesystem.
    pub fn dax(&self) -> Option<bool> {
        if self.0.stx_attributes_mask & libc::STATX_ATTR_DAX as u64 > 0 {
            Some(self.0.stx_attributes & libc::STATX_ATTR_DAX as u64 > 0)
        } else {
            None
        }
    }
    */
}
