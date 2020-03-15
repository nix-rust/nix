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
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_RDONLY;
        /// File system written synchronously
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_SYNCHRONOUS;
        /// Can't exec from filesystem
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_NOEXEC;
        /// Don't honor setuid bits on fs
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_NOSUID;
        /// Don't interpret special files
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_NODEV;
        /// Union with underlying filesystem
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_UNION;
        /// File system written asynchronously
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_ASYNC;
        /// File system supports content protection
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_CPROTECT;
        /// File system is exported
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_EXPORTED;
        /// File system is quarantined
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_QUARANTINE;
        /// Filesystem is stored locally
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_LOCAL;
        /// Quotas are enabled on filesystem
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_QUOTA;
        /// Identifies the root filesystem
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_ROOTFS;
        /// FS supports volfs (deprecated flag in Mac OS X 10.5)
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_DOVOLFS;
        /// File system is not appropriate path to user data
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_DONTBROWSE;
        /// VFS will ignore ownership information on filesystem objects
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_IGNORE_OWNERSHIP;
        /// Filesystem was mounted by automounter
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_AUTOMOUNTED;
        /// Filesystem is journaled
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_JOURNALED;
        /// Don't allow user extended attributes
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_NOUSERXATTR;
        /// Filesystem should defer writes
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_DEFWRITE;
        /// MAC support for individual labels
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_MULTILABEL;
        /// Disable update of file access time
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_NOATIME;
        /// The mount is a snapshot
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_SNAPSHOT;
        /// Not a real mount, just an update
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_UPDATE;
        /// Don't block unmount if not responding
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
        MNT_NOBLOCK;
        /// Reload filesystem data
        #[cfg(any(
            target_os = "macos",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "bitrig",
            target_os = "netbsd"
        ))]
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
