use crate::{Errno, NixPath, Result};
use libc::c_int;

libc_bitflags!(
    /// Used with [`Nmount::nmount`].
    pub struct MntFlags: c_int {
        /// All I/O to the file system should be done asynchronously.
        MNT_ASYNC;
        /// Force a read-write mount even if the file system appears to be
        /// unclean.
        MNT_FORCE;
        /// MAC support for objects.
        MNT_MULTILABEL;
        /// Disable read clustering.
        #[cfg(freebsdlike)]
        MNT_NOCLUSTERR;
        /// Disable write clustering.
        #[cfg(freebsdlike)]
        MNT_NOCLUSTERW;
        /// Do not update access times.
        MNT_NOATIME;
        /// Disallow program execution.
        MNT_NOEXEC;
        /// Do not honor setuid or setgid bits on files when executing them.
        MNT_NOSUID;
        /// Mount read-only.
        MNT_RDONLY;
        /// Causes the vfs subsystem to update its data structures pertaining to
        /// the specified already mounted file system.
        MNT_RELOAD;
        /// Create a snapshot of the file system.
        ///
        MNT_SNAPSHOT;
        /// All I/O to the file system should be done synchronously.
        MNT_SYNCHRONOUS;
        /// Union with underlying fs.
        MNT_UNION;
        /// Indicates that the mount command is being applied to an already
        /// mounted file system.
        MNT_UPDATE;
    }
);


libc_bitflags!(
    /// Used with [`mount`].
    pub struct MsFlags: c_int {
        MS_ASYNC;
        MS_INVALIDATE;
        MS_SYNC;
        MS_KILLPAGES;
        MS_DEACTIVATE;
    }
);


/// Mount a file system.
///
/// # Arguments
/// - `source`  -   Specifies the file system.  e.g. `/dev/sd0`.
/// - `target` -    Specifies the destination.  e.g. `/mnt`.
/// - `flags` -     Optional flags controlling the mount.
/// - `data` -      Optional file system specific data.
///
pub fn mount<
    P1: ?Sized + NixPath,
    P2: ?Sized + NixPath,
    P3: ?Sized + NixPath,
    P4: ?Sized + NixPath,
>(
    source: Option<&P1>,
    target: &P2,
    fstype: Option<&P3>,
    flags: MsFlags,
    data: Option<&P4>,
) -> Result<()> {
    fn with_opt_nix_path<P, T, F>(p: Option<&P>, f: F) -> Result<T>
    where
        P: ?Sized + NixPath,
        F: FnOnce(*const libc::c_char) -> T,
    {
        match p {
            Some(path) => path.with_nix_path(|p_str| f(p_str.as_ptr())),
            None => Ok(f(std::ptr::null())),
        }
    }

    let res = with_opt_nix_path(source, |s| {
        target.with_nix_path(|t| {
            with_opt_nix_path(fstype, |_| {
                with_opt_nix_path(data, |d| unsafe {
                    libc::mount(
                        s,
                        t.as_ptr(),
                        flags.bits(),
                        d as *mut libc::c_void,
                    )
                })
            })
        })
    })????;

    Errno::result(res).map(drop)
}

/// Unmount the file system mounted at `target`.
pub fn unmount<P>(mountpoint: &P, flags: MntFlags) -> Result<()>
where
    P: ?Sized + NixPath,
{
    let res = mountpoint.with_nix_path(|cstr| unsafe {
        libc::unmount(cstr.as_ptr(), flags.bits())
    })?;

    Errno::result(res).map(drop)
}
