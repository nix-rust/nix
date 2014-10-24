use errno::{SysResult, SysError};
use std::io::FilePermission;
use fcntl::{Fd, OFlag};
use libc::{c_void, size_t, off_t, mode_t};

pub use self::consts::*;

#[cfg(target_os = "linux")]
mod consts {
    use libc::c_int;

    pub type MmapFlag = c_int;

    pub const MAP_SHARED: MmapFlag          = 0x00001;
    pub const MAP_PRIVATE: MmapFlag         = 0x00002;
    pub const MAP_FIXED: MmapFlag           = 0x00010;

    pub const MAP_FILE: MmapFlag            = 0x00000;
    pub const MAP_ANONYMOUS: MmapFlag       = 0x00020;
    pub const MAP_ANON: MmapFlag            = MAP_ANONYMOUS;
    pub const MAP_32BIT: MmapFlag           = 0x00040;

    pub const MAP_GROWSDOWN: MmapFlag       = 0x00100;
    pub const MAP_DENYWRITE: MmapFlag       = 0x00800;
    pub const MAP_EXECUTABLE: MmapFlag      = 0x01000;
    pub const MAP_LOCKED: MmapFlag          = 0x02000;
    pub const MAP_NORESERVE: MmapFlag       = 0x04000;
    pub const MAP_POPULATE: MmapFlag        = 0x08000;
    pub const MAP_NONBLOCK: MmapFlag        = 0x10000;
    pub const MAP_STACK: MmapFlag           = 0x20000;
    pub const MAP_HUGETLB: MmapFlag         = 0x40000;

    pub type MmapProt = c_int;

    pub const PROT_READ: MmapProt           = 0x1;
    pub const PROT_WRITE: MmapProt          = 0x2;
    pub const PROT_EXEC: MmapProt           = 0x4;
    pub const PROT_NONE: MmapProt           = 0x0;
    pub const PROT_GROWSDOWN: MmapProt      = 0x01000000;
    pub const PROT_GROWSUP: MmapProt        = 0x02000000;

    pub const MAP_FAILED: int               = -1;
}

#[cfg(target_os = "macos")]
mod consts {
    use libc::c_int;

    pub type MmapFlag = c_int;

    pub const MAP_SHARED: MmapFlag          = 0x00001;
    pub const MAP_PRIVATE: MmapFlag         = 0x00002;
    pub const MAP_FIXED: MmapFlag           = 0x00010;

    pub const MAP_NOCACHE: MmapFlag         = 0x00400;
    pub const MAP_JIT: MmapFlag             = 0x00800;

    pub type MmapProt = c_int;

    pub const PROT_READ: MmapProt           = 0x1;
    pub const PROT_WRITE: MmapProt          = 0x2;
    pub const PROT_EXEC: MmapProt           = 0x4;
    pub const PROT_NONE: MmapProt           = 0x0;

    pub const MAP_FAILED: int               = -1;
}

mod ffi {
    use libc::{c_void, size_t, c_int, c_char, mode_t};

    pub use libc::{mmap, munmap};


    extern {
        pub fn shm_open(name: *const c_char, oflag: c_int, mode: mode_t) -> c_int;
        pub fn shm_unlink(name: *const c_char) -> c_int;
        pub fn mlock(addr: *const c_void, len: size_t) -> c_int;
        pub fn munlock(addr: *const c_void, len: size_t) -> c_int;
    }
}

pub unsafe fn mlock(addr: *const c_void, length: size_t) -> SysResult<()> {
    match ffi::mlock(addr, length) {
        0 => Ok(()),
        _ => Err(SysError::last())
    }
}

pub fn munlock(addr: *const c_void, length: size_t) -> SysResult<()> {
    match unsafe { ffi::munlock(addr, length) } {
        0 => Ok(()),
        _ => Err(SysError::last())
    }
}

/// Calls to mmap are inherently unsafe, so they must be made in an unsafe block. Typically
/// a higher-level abstraction will hide the unsafe interactions with the mmap'd region.
pub fn mmap(addr: *mut c_void, length: size_t, prot: MmapProt, flags: MmapFlag, fd: Fd, offset: off_t) -> SysResult<*mut c_void> {
    let ret = unsafe { ffi::mmap(addr, length, prot, flags, fd, offset) };

    if ret as int == MAP_FAILED  {
        Err(SysError::last())
    } else {
        Ok(ret)
    }
}

pub fn munmap(addr: *mut c_void, len: size_t) -> SysResult<()> {
    match unsafe { ffi::munmap(addr, len) } {
        0 => Ok(()),
        _ => Err(SysError::last())
    }
}

pub fn shm_open(name: &String, flag: OFlag, mode: FilePermission) -> SysResult<Fd> {
    let ret = unsafe { ffi::shm_open(name.to_c_str().as_ptr(), flag.bits(), mode.bits() as mode_t) };

    if ret < 0 {
        Err(SysError::last())
    } else {
        Ok(ret)
    }
}

pub fn shm_unlink(name: &String) -> SysResult<()> {
    let ret = unsafe { ffi::shm_unlink(name.to_c_str().as_ptr()) };

    if ret < 0 {
        Err(SysError::last())
    } else {
        Ok(())
    }
}

