use libc::{self, c_ulong, c_int};
use {Result, NixPath};
use errno::Errno;

libc_bitflags!(
    pub struct MsFlags: c_ulong {
        /// Mount read-only
        MS_RDONLY;
        /// Ignore suid and sgid bits
        MS_NOSUID;
        /// Disallow access to device special files
        MS_NODEV;
        /// Disallow program execution
        MS_NOEXEC;
        /// Writes are synced at once
        MS_SYNCHRONOUS;
        /// Alter flags of a mounted FS
        MS_REMOUNT;
        /// Allow mandatory locks on a FS
        MS_MANDLOCK;
        /// Directory modifications are synchronous
        MS_DIRSYNC;
        /// Do not update access times
        MS_NOATIME;
        /// Do not update directory access times
        MS_NODIRATIME;
        /// Linux 2.4.0 - Bind directory at different place
        MS_BIND;
        MS_MOVE;
        MS_REC;
        MS_SILENT;
        MS_POSIXACL;
        MS_UNBINDABLE;
        MS_PRIVATE;
        MS_SLAVE;
        MS_SHARED;
        MS_RELATIME;
        MS_KERNMOUNT;
        MS_I_VERSION;
        MS_STRICTATIME;
        MS_ACTIVE;
        MS_NOUSER;
        MS_RMT_MASK;
        MS_MGC_VAL;
        MS_MGC_MSK;
    }
);

libc_bitflags!(
    pub struct MntFlags: c_int {
        /// Read only filesystem
        #[cfg(target_os = "macos")]
        MNT_RDONLY;
        /// File system written synchronously
        #[cfg(target_os = "macos")]
        MNT_SYNCHRONOUS;
        /// Can't exec from filesystem
        #[cfg(target_os = "macos")]
        MNT_NOEXEC;
        /// Don't honor setuid bits on fs
        #[cfg(target_os = "macos")]
        MNT_NOSUID;
        /// Don't interpret special files
        #[cfg(target_os = "macos")]
        MNT_NODEV;
        /// Union with underlying filesystem
        #[cfg(target_os = "macos")]
        MNT_UNION;
        /// File system written asynchronously
        #[cfg(target_os = "macos")]
        MNT_ASYNC;
        /// File system supports content protection
        #[cfg(target_os = "macos")]
        MNT_CPROTECT;
        /// File system is exported
        #[cfg(target_os = "macos")]
        MNT_EXPORTED;
        /// File system is quarantined
        #[cfg(target_os = "macos")]
        MNT_QUARANTINE;
        /// Filesystem is stored locally
        #[cfg(target_os = "macos")]
        MNT_LOCAL;
        /// Quotas are enabled on filesystem
        #[cfg(target_os = "macos")]
        MNT_QUOTA;
        /// Identifies the root filesystem
        #[cfg(target_os = "macos")]
        MNT_ROOTFS;
        /// FS supports volfs (deprecated flag in Mac OS X 10.5)
        #[cfg(target_os = "macos")]
        MNT_DOVOLFS;
        /// File system is not appropriate path to user data
        #[cfg(target_os = "macos")]
        MNT_DONTBROWSE;
        /// VFS will ignore ownership information on filesystem objects
        #[cfg(target_os = "macos")]
        MNT_IGNORE_OWNERSHIP;
        /// Filesystem was mounted by automounter
        #[cfg(target_os = "macos")]
        MNT_AUTOMOUNTED;
        /// Filesystem is journaled
        #[cfg(target_os = "macos")]
        MNT_JOURNALED;
        /// Don't allow user extended attributes
        #[cfg(target_os = "macos")]
        MNT_NOUSERXATTR;
        /// Filesystem should defer writes
        #[cfg(target_os = "macos")]
        MNT_DEFWRITE;
        /// MAC support for individual labels
        #[cfg(target_os = "macos")]
        MNT_MULTILABEL;
        /// Disable update of file access time
        #[cfg(target_os = "macos")]
        MNT_NOATIME;
        /// The mount is a snapshot
        #[cfg(target_os = "macos")]
        MNT_SNAPSHOT;
        /// Not a real mount, just an update
        #[cfg(target_os = "macos")]
        MNT_UPDATE;
        /// Don't block unmount if not responding
        #[cfg(target_os = "macos")]
        MNT_NOBLOCK;
        /// Reload filesystem data
        #[cfg(target_os = "macos")]
        MNT_RELOAD;
        /// Force unmount or readonly change
        MNT_FORCE;
        MNT_DETACH;
        MNT_EXPIRE;
    }
);

pub fn mount<P1: ?Sized + NixPath, P2: ?Sized + NixPath, P3: ?Sized + NixPath, P4: ?Sized + NixPath>(
        source: Option<&P1>,
        target: &P2,
        fstype: Option<&P3>,
        flags: MsFlags,
        data: Option<&P4>) -> Result<()> {

    let res =
        source.with_nix_path(|source| {
            target.with_nix_path(|target| {
                fstype.with_nix_path(|fstype| {
                    data.with_nix_path(|data| {
                        unsafe {
                            libc::mount(source.as_ptr(),
                                       target.as_ptr(),
                                       fstype.as_ptr(),
                                       flags.bits,
                                       data.as_ptr() as *const libc::c_void)
                        }
                    })
                })
            })
        })????;

    Errno::result(res).map(drop)
}

pub fn umount<P: ?Sized + NixPath>(target: &P) -> Result<()> {
    let res = target.with_nix_path(|cstr| {
        unsafe { libc::umount(cstr.as_ptr()) }
    })?;

    Errno::result(res).map(drop)
}

pub fn umount2<P: ?Sized + NixPath>(target: &P, flags: MntFlags) -> Result<()> {
    let res = target.with_nix_path(|cstr| {
        unsafe { libc::umount2(cstr.as_ptr(), flags.bits) }
    })?;

    Errno::result(res).map(drop)
}
