#![cfg(target_os = "linux")]

use std::ptr;
use std::path::Path;
use libc::{c_ulong, c_int, c_void};
use errno::{SysResult, from_ffi};

bitflags!(
    flags MsFlags: c_ulong {
        static MS_RDONLY      = 1 << 0,  // Mount read-only
        static MS_NOSUID      = 1 << 1,  // Ignore suid and sgid bits
        static MS_NODEV       = 1 << 2,  // Disallow access to device special files
        static MS_NOEXEC      = 1 << 3,  // Disallow program execution
        static MS_SYNCHRONOUS = 1 << 4,  // Writes are synced at once
        static MS_REMOUNT     = 1 << 5,  // Alter flags of a mounted FS
        static MS_MANDLOCK    = 1 << 6,  // Allow mandatory locks on a FS
        static MS_DIRSYNC     = 1 << 7,  // Directory modifications are synchronous
        static MS_NOATIME     = 1 << 10, // Do not update access times
        static MS_NODIRATIME  = 1 << 11, // Do not update directory access times
        static MS_BIND        = 1 << 12, // Linux 2.4.0 - Bind directory at different place
        static MS_MOVE        = 1 << 13,
        static MS_REC         = 1 << 14,
        static MS_VERBOSE     = 1 << 15, // Deprecated
        static MS_SILENT      = 1 << 15,
        static MS_POSIXACL    = 1 << 16,
        static MS_UNBINDABLE  = 1 << 17,
        static MS_PRIVATE     = 1 << 18,
        static MS_SLAVE       = 1 << 19,
        static MS_SHARED      = 1 << 20,
        static MS_RELATIME    = 1 << 21,
        static MS_KERNMOUNT   = 1 << 22,
        static MS_I_VERSION   = 1 << 23,
        static MS_STRICTATIME = 1 << 24,
        static MS_NOSEC       = 1 << 28,
        static MS_BORN        = 1 << 29,
        static MS_ACTIVE      = 1 << 30,
        static MS_NOUSER      = 1 << 31,
        static MS_RMT_MASK    = MS_RDONLY.bits
                              | MS_SYNCHRONOUS.bits
                              | MS_MANDLOCK.bits
                              | MS_I_VERSION.bits,
        static MS_MGC_VAL     = 0xC0ED0000,
        static MS_MGC_MSK     = 0xffff0000
    }
)

bitflags!(
    flags MntFlags: c_int {
        static MNT_FORCE   = 1 << 0,
        static MNT_DETATCH = 1 << 1,
        static MNT_EXPIRE  = 1 << 2
    }
)

mod ffi {
    use libc::{c_char, c_int, c_void, c_ulong};

    extern {
        pub fn mount(
                source: *const c_char,
                target: *const c_char,
                fstype: *const c_char,
                flags: c_ulong,
                data: *const c_void) -> c_int;

        pub fn umount(target: *const c_char) -> c_int;

        pub fn umount2(target: *const c_char, flags: c_int) -> c_int;
    }
}

pub fn mount(
        source: Option<&Path>,
        target: &Path,
        fstype: Option<&str>,
        flags: MsFlags,
        data: Option<&str>) -> SysResult<()> {

    let source = source.map(|s| s.to_c_str());
    let target = target.to_c_str();
    let fstype = fstype.map(|s| s.to_c_str());
    let data = data.map(|s| s.to_c_str());

    let res = unsafe {
        ffi::mount(
            source.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null()),
            target.as_ptr(),
            fstype.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null()),
            flags.bits,
            data.map(|s| s.as_ptr() as *const c_void).unwrap_or(ptr::null()))
    };

    from_ffi(res)
}

pub fn umount(target: &Path) -> SysResult<()> {
    let target = target.to_c_str();

    let res = unsafe { ffi::umount(target.as_ptr()) };

    from_ffi(res)
}

pub fn umount2(target: &Path, flags: MntFlags) -> SysResult<()> {
    let target = target.to_c_str();

    let res = unsafe { ffi::umount2(target.as_ptr(), flags.bits) };

    from_ffi(res)
}
