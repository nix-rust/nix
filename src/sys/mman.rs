use {Errno, Error, Result, NixPath};
use fcntl::OFlag;
use libc::{self, c_void, size_t, off_t, mode_t};
use sys::stat::Mode;
use std::os::unix::io::RawFd;

pub use self::consts::*;

libc_bitflags!{
    pub flags ProtFlags: libc::c_int {
        PROT_NONE,
        PROT_READ,
        PROT_WRITE,
        PROT_EXEC,
        #[cfg(any(target_os = "linux", target_os = "android"))]
        PROT_GROWSDOWN,
        #[cfg(any(target_os = "linux", target_os = "android"))]
        PROT_GROWSUP,
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod consts {
    use libc::{self, c_int};

    libc_bitflags!{
        pub flags MapFlags: c_int {
            MAP_FILE,
            MAP_SHARED,
            MAP_PRIVATE,
            MAP_FIXED,
            MAP_ANON,
            MAP_ANONYMOUS,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            MAP_32BIT,
            MAP_GROWSDOWN,
            MAP_DENYWRITE,
            MAP_EXECUTABLE,
            MAP_LOCKED,
            MAP_NORESERVE,
            MAP_POPULATE,
            MAP_NONBLOCK,
            MAP_STACK,
            MAP_HUGETLB,
        }
    }

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


    libc_bitflags!{
        pub flags MsFlags: c_int {
            MS_ASYNC,
            MS_INVALIDATE,
            MS_SYNC,
        }
    }

    pub const MAP_FAILED: isize               = -1;
}

#[cfg(any(target_os = "macos",
          target_os = "ios"))]
mod consts {
    use libc::{self, c_int};

    libc_bitflags!{
        pub flags MapFlags: c_int {
            MAP_FILE,
            MAP_SHARED,
            MAP_PRIVATE,
            MAP_FIXED,
            MAP_ANON,
            MAP_NOCACHE,
            MAP_JIT,
        }
    }

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

    libc_bitflags!{
        pub flags MsFlags: c_int {
            MS_ASYNC, /* [MF|SIO] return immediately */
            MS_INVALIDATE, /* [MF|SIO] invalidate all cached data */
            MS_KILLPAGES, /* invalidate pages, leave mapped */
            MS_DEACTIVATE, /* deactivate pages, leave mapped */
            MS_SYNC, /* [MF|SIO] msync synchronously */
        }
    }

    pub const MAP_FAILED: isize               = -1;
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd"))]
mod consts {
    use libc::{self, c_int};

    libc_bitflags!{
        pub flags MapFlags: c_int {
            MAP_FILE,
            MAP_SHARED,
            MAP_PRIVATE,
            MAP_FIXED,
            MAP_RENAME,
            MAP_NORESERVE,
            MAP_HASSEMAPHORE,
            #[cfg(not(any(target_os = "openbsd", target_os = "netbsd")))]
            MAP_STACK,
            #[cfg(target_os = "netbsd")]
            MAP_WIRED,
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            MAP_NOSYNC,
            MAP_ANON,
        }
    }

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

    bitflags!{
        pub struct MsFlags: c_int {
            const MS_ASYNC      = libc::MS_ASYNC; /* [MF|SIO] return immediately */
            const MS_INVALIDATE = libc::MS_INVALIDATE; /* [MF|SIO] invalidate all cached data */
            #[cfg(not(target_os = "dragonfly"))]
            const MS_KILLPAGES  = 0x0004; /* invalidate pages, leave mapped */
            #[cfg(not(target_os = "dragonfly"))]
            const MS_DEACTIVATE = 0x0004; /* deactivate pages, leave mapped */
            const MS_SYNC       = libc::MS_SYNC; /* [MF|SIO] msync synchronously */
        }
    }

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
    Errno::result(ffi::mlock(addr, length)).map(drop)
}

pub unsafe fn munlock(addr: *const c_void, length: size_t) -> Result<()> {
    Errno::result(ffi::munlock(addr, length)).map(drop)
}

/// Calls to mmap are inherently unsafe, so they must be made in an unsafe block. Typically
/// a higher-level abstraction will hide the unsafe interactions with the mmap'd region.
pub unsafe fn mmap(addr: *mut c_void, length: size_t, prot: ProtFlags, flags: MapFlags, fd: RawFd, offset: off_t) -> Result<*mut c_void> {
    let ret = ffi::mmap(addr, length, prot.bits(), flags.bits(), fd, offset);

    if ret as isize == MAP_FAILED  {
        Err(Error::Sys(Errno::last()))
    } else {
        Ok(ret)
    }
}

pub unsafe fn munmap(addr: *mut c_void, len: size_t) -> Result<()> {
    Errno::result(ffi::munmap(addr, len)).map(drop)
}

pub unsafe fn madvise(addr: *const c_void, length: size_t, advise: MmapAdvise) -> Result<()> {
    Errno::result(ffi::madvise(addr, length, advise)).map(drop)
}

pub unsafe fn msync(addr: *const c_void, length: size_t, flags: MsFlags) -> Result<()> {
    Errno::result(ffi::msync(addr, length, flags.bits())).map(drop)
}

pub fn shm_open<P: ?Sized + NixPath>(name: &P, flag: OFlag, mode: Mode) -> Result<RawFd> {
    let ret = try!(name.with_nix_path(|cstr| {
        unsafe {
            ffi::shm_open(cstr.as_ptr(), flag.bits(), mode.bits() as mode_t)
        }
    }));

    Errno::result(ret)
}

pub fn shm_unlink<P: ?Sized + NixPath>(name: &P) -> Result<()> {
    let ret = try!(name.with_nix_path(|cstr| {
        unsafe { ffi::shm_unlink(cstr.as_ptr()) }
    }));

    Errno::result(ret).map(drop)
}
