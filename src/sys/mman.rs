use {Error, Result, NixPath};
use errno::Errno;
use fcntl::OFlag;
use libc::{c_void, size_t, off_t, mode_t};
use sys::stat::Mode;
use std::os::unix::io::RawFd;

pub use self::consts::*;

#[cfg(any(target_os = "linux", target_os = "android"))]
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

    pub type MmapAdvise = c_int;

    pub const MADV_NORMAL     : MmapAdvise  = 0; /* No further special treatment.  */
    pub const MADV_RANDOM     : MmapAdvise  = 1; /* Expect random page references.  */
    pub const MADV_SEQUENTIAL : MmapAdvise  = 2; /* Expect sequential page references.  */
    pub const MADV_WILLNEED   : MmapAdvise  = 3; /* Will need these pages.  */
    pub const MADV_DONTNEED   : MmapAdvise  = 4; /* Don't need these pages.  */
    pub const MADV_REMOVE     : MmapAdvise  = 9; /* Remove these pages and resources.  */
    pub const MADV_DONTFORK   : MmapAdvise  = 10; /* Do not inherit across fork.  */
    pub const MADV_DOFORK     : MmapAdvise  = 11; /* Do inherit across fork.  */
    pub const MADV_MERGEABLE  : MmapAdvise  = 12; /* KSM may merge identical pages.  */
    pub const MADV_UNMERGEABLE: MmapAdvise  = 13; /* KSM may not merge identical pages.  */
    pub const MADV_HUGEPAGE   : MmapAdvise  = 14; /* Worth backing with hugepages.  */
    pub const MADV_NOHUGEPAGE : MmapAdvise  = 15; /* Not worth backing with hugepages.  */
    pub const MADV_DONTDUMP   : MmapAdvise  = 16; /* Explicity exclude from the core dump, overrides the coredump filter bits.  */
    pub const MADV_DODUMP     : MmapAdvise  = 17; /* Clear the MADV_DONTDUMP flag.  */
    pub const MADV_HWPOISON   : MmapAdvise  = 100; /* Poison a page for testing.  */

    pub type MmapSync = c_int;

    pub const MS_ASYNC : MmapSync           = 1;
    pub const MS_SYNC  : MmapSync           = 4;
    pub const MS_INVALIDATE : MmapSync      = 2;

    pub const MAP_FAILED: isize               = -1;
}

#[cfg(any(target_os = "macos",
          target_os = "ios"))]
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

    pub type MmapAdvise = c_int;

    pub const MADV_NORMAL     : MmapAdvise      = 0; /* No further special treatment.  */
    pub const MADV_RANDOM     : MmapAdvise      = 1; /* Expect random page references.  */
    pub const MADV_SEQUENTIAL : MmapAdvise      = 2; /* Expect sequential page references.  */
    pub const MADV_WILLNEED   : MmapAdvise      = 3; /* Will need these pages.  */
    pub const MADV_DONTNEED   : MmapAdvise      = 4; /* Don't need these pages.  */
    pub const MADV_FREE       : MmapAdvise      = 5; /* pages unneeded, discard contents */
    pub const MADV_ZERO_WIRED_PAGES: MmapAdvise = 6; /* zero the wired pages that have not been unwired before the entry is deleted */
    pub const MADV_FREE_REUSABLE : MmapAdvise   = 7; /* pages can be reused (by anyone) */
    pub const MADV_FREE_REUSE : MmapAdvise      = 8; /* caller wants to reuse those pages */
    pub const MADV_CAN_REUSE : MmapAdvise       = 9;

    pub type MmapSync = c_int;

    pub const MS_ASYNC      : MmapSync          = 0x0001; /* [MF|SIO] return immediately */
    pub const MS_INVALIDATE	: MmapSync          = 0x0002; /* [MF|SIO] invalidate all cached data */
    pub const MS_SYNC		: MmapSync          = 0x0010; /* [MF|SIO] msync synchronously */
    pub const MS_KILLPAGES  : MmapSync          = 0x0004; /* invalidate pages, leave mapped */
    pub const MS_DEACTIVATE : MmapSync          = 0x0008; /* deactivate pages, leave mapped */

    pub const MAP_FAILED: isize               = -1;
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd"))]
mod consts {
    use libc::c_int;

    pub type MmapFlag = c_int;

    pub const MAP_SHARED: MmapFlag          = 0x00001;
    pub const MAP_PRIVATE: MmapFlag         = 0x00002;
    pub const MAP_FIXED: MmapFlag           = 0x00010;

    pub const MAP_RENAME: MmapFlag          = 0x00020;
    pub const MAP_NORESERVE: MmapFlag       = 0x00040;
    pub const MAP_HASSEMAPHORE: MmapFlag    = 0x00200;
    pub const MAP_STACK: MmapFlag           = 0x00400;
    #[cfg(target_os = "netbsd")]
    pub const MAP_WIRED: MmapFlag           = 0x00800;
    pub const MAP_NOSYNC: MmapFlag          = 0x00800;
    pub const MAP_FILE: MmapFlag            = 0x00000;
    pub const MAP_ANON: MmapFlag            = 0x01000;

    pub type MmapProt = c_int;

    pub const PROT_READ: MmapProt           = 0x1;
    pub const PROT_WRITE: MmapProt          = 0x2;
    pub const PROT_EXEC: MmapProt           = 0x4;
    pub const PROT_NONE: MmapProt           = 0x0;

    pub type MmapAdvise = c_int;

    pub const MADV_NORMAL     : MmapAdvise      = 0; /* No further special treatment.  */
    pub const MADV_RANDOM     : MmapAdvise      = 1; /* Expect random page references.  */
    pub const MADV_SEQUENTIAL : MmapAdvise      = 2; /* Expect sequential page references.  */
    pub const MADV_WILLNEED   : MmapAdvise      = 3; /* Will need these pages.  */
    pub const MADV_DONTNEED   : MmapAdvise      = 4; /* Don't need these pages.  */
    pub const MADV_FREE       : MmapAdvise      = 5; /* pages unneeded, discard contents */
    pub const MADV_NOSYNC     : MmapAdvise      = 6; /* try to avoid flushes to physical media*/
    pub const MADV_AUTOSYNC   : MmapAdvise      = 7; /* refert to default flushing strategy */
    pub const MADV_NOCORE     : MmapAdvise      = 8; /* do not include these pages in a core file */
    pub const MADV_CORE       : MmapAdvise      = 9; /* revert to including pages in a core file */
    #[cfg(not(target_os = "dragonfly"))]
    pub const MADV_PROTECT    : MmapAdvise      = 10; /* protect process from pageout kill */
    #[cfg(target_os = "dragonfly")]
    pub const MADV_INVAL      : MmapAdvise      = 10; /* virt page tables have changed, inval pmap */
    #[cfg(target_os = "dragonfly")]
    pub const MADV_SETMAP     : MmapAdvise      = 11; /* set page table directory page for map */

    pub type MmapSync = c_int;

    pub const MS_ASYNC      : MmapSync          = 0x0001; /* [MF|SIO] return immediately */
    pub const MS_INVALIDATE : MmapSync          = 0x0002; /* [MF|SIO] invalidate all cached data */
    #[cfg(not(target_os = "dragonfly"))]
    pub const MS_SYNC       : MmapSync          = 0x0010; /* [MF|SIO] msync synchronously */
    #[cfg(target_os = "dragonfly")]
    pub const MS_SYNC       : MmapSync          = 0x0000; /* [MF|SIO] msync synchronously */
    #[cfg(not(target_os = "dragonfly"))]
    pub const MS_KILLPAGES  : MmapSync          = 0x0004; /* invalidate pages, leave mapped */
    #[cfg(not(target_os = "dragonfly"))]
    pub const MS_DEACTIVATE : MmapSync          = 0x0008; /* deactivate pages, leave mapped */

    pub const MAP_FAILED: isize                 = -1;
}
mod ffi {
    use libc::{c_void, size_t, c_int, c_char, mode_t};

    pub use libc::{mmap, munmap};

    #[allow(improper_ctypes)]
    extern {
        pub fn shm_open(name: *const c_char, oflag: c_int, mode: mode_t) -> c_int;
        pub fn shm_unlink(name: *const c_char) -> c_int;
        pub fn mlock(addr: *const c_void, len: size_t) -> c_int;
        pub fn munlock(addr: *const c_void, len: size_t) -> c_int;
        pub fn madvise (addr: *const c_void, len: size_t, advice: c_int) -> c_int;
        pub fn msync (addr: *const c_void, len: size_t, flags: c_int) -> c_int;
    }
}

pub unsafe fn mlock(addr: *const c_void, length: size_t) -> Result<()> {
    match ffi::mlock(addr, length) {
        0 => Ok(()),
        _ => Err(Error::Sys(Errno::last()))
    }
}

pub fn munlock(addr: *const c_void, length: size_t) -> Result<()> {
    match unsafe { ffi::munlock(addr, length) } {
        0 => Ok(()),
        _ => Err(Error::Sys(Errno::last()))
    }
}

/// Calls to mmap are inherently unsafe, so they must be made in an unsafe block. Typically
/// a higher-level abstraction will hide the unsafe interactions with the mmap'd region.
pub fn mmap(addr: *mut c_void, length: size_t, prot: MmapProt, flags: MmapFlag, fd: RawFd, offset: off_t) -> Result<*mut c_void> {
    let ret = unsafe { ffi::mmap(addr, length, prot, flags, fd, offset) };

    if ret as isize == MAP_FAILED  {
        Err(Error::Sys(Errno::last()))
    } else {
        Ok(ret)
    }
}

pub fn munmap(addr: *mut c_void, len: size_t) -> Result<()> {
    match unsafe { ffi::munmap(addr, len) } {
        0 => Ok(()),
        _ => Err(Error::Sys(Errno::last()))
    }
}

pub fn madvise(addr: *const c_void, length: size_t, advise: MmapAdvise) -> Result<()> {
    match unsafe { ffi::madvise(addr, length, advise) } {
        0 => Ok(()),
        _ => Err(Error::Sys(Errno::last()))
    }
}

pub fn msync(addr: *const c_void, length: size_t, flags: MmapSync) -> Result<()> {
    match unsafe { ffi::msync(addr, length, flags) } {
        0 => Ok(()),
        _ => Err(Error::Sys(Errno::last()))
    }
}

pub fn shm_open<P: ?Sized + NixPath>(name: &P, flag: OFlag, mode: Mode) -> Result<RawFd> {
    let ret = try!(name.with_nix_path(|cstr| {
        unsafe {
            ffi::shm_open(cstr.as_ptr(), flag.bits(), mode.bits() as mode_t)
        }
    }));

    if ret < 0 {
        Err(Error::Sys(Errno::last()))
    } else {
        Ok(ret)
    }
}

pub fn shm_unlink<P: ?Sized + NixPath>(name: &P) -> Result<()> {
    let ret = try!(name.with_nix_path(|cstr| {
        unsafe { ffi::shm_unlink(cstr.as_ptr()) }
    }));

    if ret < 0 {
        Err(Error::Sys(Errno::last()))
    } else {
        Ok(())
    }
}
