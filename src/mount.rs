use libc::{c_ulong, c_int};
use libc;
use {Errno, Result, NixPath};

bitflags!(
    pub struct MsFlags: c_ulong {
        const MS_RDONLY      = libc::MS_RDONLY;      // Mount read-only
        const MS_NOSUID      = libc::MS_NOSUID;      // Ignore suid and sgid bits
        const MS_NODEV       = libc::MS_NODEV;       // Disallow access to device special files
        const MS_NOEXEC      = libc::MS_NOEXEC;      // Disallow program execution
        const MS_SYNCHRONOUS = libc::MS_SYNCHRONOUS; // Writes are synced at once
        const MS_REMOUNT     = libc::MS_REMOUNT;     // Alter flags of a mounted FS
        const MS_MANDLOCK    = libc::MS_MANDLOCK;    // Allow mandatory locks on a FS
        const MS_DIRSYNC     = libc::MS_DIRSYNC;     // Directory modifications are synchronous
        const MS_NOATIME     = libc::MS_NOATIME;     // Do not update access times
        const MS_NODIRATIME  = libc::MS_NODIRATIME;  // Do not update directory access times
        const MS_BIND        = libc::MS_BIND;        // Linux 2.4.0 - Bind directory at different place
        const MS_MOVE        = libc::MS_MOVE;
        const MS_REC         = libc::MS_REC;
        const MS_VERBOSE     = 1 << 15;              // Deprecated
        const MS_SILENT      = libc::MS_SILENT;
        const MS_POSIXACL    = libc::MS_POSIXACL;
        const MS_UNBINDABLE  = libc::MS_UNBINDABLE;
        const MS_PRIVATE     = libc::MS_PRIVATE;
        const MS_SLAVE       = libc::MS_SLAVE;
        const MS_SHARED      = libc::MS_SHARED;
        const MS_RELATIME    = libc::MS_RELATIME;
        const MS_KERNMOUNT   = libc::MS_KERNMOUNT;
        const MS_I_VERSION   = libc::MS_I_VERSION;
        const MS_STRICTATIME = libc::MS_STRICTATIME;
        const MS_NOSEC       = 1 << 28;
        const MS_BORN        = 1 << 29;
        const MS_ACTIVE      = libc::MS_ACTIVE;
        const MS_NOUSER      = libc::MS_NOUSER;
        const MS_RMT_MASK    = libc::MS_RMT_MASK;
        const MS_MGC_VAL     = libc::MS_MGC_VAL;
        const MS_MGC_MSK     = libc::MS_MGC_MSK;
    }
);

libc_bitflags!(
    pub flags MntFlags: c_int {
        MNT_FORCE,
        MNT_DETACH,
        MNT_EXPIRE,
    }
);

pub fn mount<P1: ?Sized + NixPath, P2: ?Sized + NixPath, P3: ?Sized + NixPath, P4: ?Sized + NixPath>(
        source: Option<&P1>,
        target: &P2,
        fstype: Option<&P3>,
        flags: MsFlags,
        data: Option<&P4>) -> Result<()> {
    use libc;

    let res = try!(try!(try!(try!(
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
        })))));

    Errno::result(res).map(drop)
}

pub fn umount<P: ?Sized + NixPath>(target: &P) -> Result<()> {
    let res = try!(target.with_nix_path(|cstr| {
        unsafe { libc::umount(cstr.as_ptr()) }
    }));

    Errno::result(res).map(drop)
}

pub fn umount2<P: ?Sized + NixPath>(target: &P, flags: MntFlags) -> Result<()> {
    let res = try!(target.with_nix_path(|cstr| {
        unsafe { libc::umount2(cstr.as_ptr(), flags.bits) }
    }));

    Errno::result(res).map(drop)
}
