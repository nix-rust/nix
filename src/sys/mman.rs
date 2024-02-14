//! Memory management declarations.

use crate::errno::Errno;
#[cfg(not(target_os = "android"))]
use crate::NixPath;
use crate::Result;
#[cfg(not(target_os = "android"))]
#[cfg(feature = "fs")]
use crate::{fcntl::OFlag, sys::stat::Mode};
use libc::{
    self, c_int, c_short, c_void, key_t, off_t, semid_ds, seminfo, shmid_ds,
    size_t,
};
use std::ptr::NonNull;
use std::{
    num::NonZeroUsize,
    os::unix::io::{AsFd, AsRawFd},
};

libc_bitflags! {
    /// Desired memory protection of a memory mapping.
    pub struct ProtFlags: c_int {
        /// Pages cannot be accessed.
        PROT_NONE;
        /// Pages can be read.
        PROT_READ;
        /// Pages can be written.
        PROT_WRITE;
        /// Pages can be executed
        PROT_EXEC;
        /// Apply protection up to the end of a mapping that grows upwards.
        #[cfg(linux_android)]
        PROT_GROWSDOWN;
        /// Apply protection down to the beginning of a mapping that grows downwards.
        #[cfg(linux_android)]
        PROT_GROWSUP;
    }
}

libc_bitflags! {
    /// Additional parameters for [`mmap`].
    pub struct MapFlags: c_int {
        /// Compatibility flag. Ignored.
        MAP_FILE;
        /// Share this mapping. Mutually exclusive with `MAP_PRIVATE`.
        MAP_SHARED;
        /// Create a private copy-on-write mapping. Mutually exclusive with `MAP_SHARED`.
        MAP_PRIVATE;
        /// Place the mapping at exactly the address specified in `addr`.
        MAP_FIXED;
        /// Place the mapping at exactly the address specified in `addr`, but never clobber an existing range.
        #[cfg(target_os = "linux")]
        MAP_FIXED_NOREPLACE;
        /// To be used with `MAP_FIXED`, to forbid the system
        /// to select a different address than the one specified.
        #[cfg(target_os = "freebsd")]
        MAP_EXCL;
        /// Synonym for `MAP_ANONYMOUS`.
        MAP_ANON;
        /// The mapping is not backed by any file.
        MAP_ANONYMOUS;
        /// Put the mapping into the first 2GB of the process address space.
        #[cfg(any(all(linux_android,
                      any(target_arch = "x86", target_arch = "x86_64")),
                  all(target_os = "linux", target_env = "musl", any(target_arch = "x86", target_arch = "x86_64")),
                  all(target_os = "freebsd", target_pointer_width = "64")))]
        MAP_32BIT;
        /// Used for stacks; indicates to the kernel that the mapping should extend downward in memory.
        #[cfg(linux_android)]
        MAP_GROWSDOWN;
        /// Compatibility flag. Ignored.
        #[cfg(linux_android)]
        MAP_DENYWRITE;
        /// Compatibility flag. Ignored.
        #[cfg(linux_android)]
        MAP_EXECUTABLE;
        /// Mark the mmaped region to be locked in the same way as `mlock(2)`.
        #[cfg(linux_android)]
        MAP_LOCKED;
        /// Do not reserve swap space for this mapping.
        ///
        /// This was removed in FreeBSD 11 and is unused in DragonFlyBSD.
        #[cfg(not(any(freebsdlike, target_os = "aix", target_os = "hurd")))]
        MAP_NORESERVE;
        /// Populate page tables for a mapping.
        #[cfg(linux_android)]
        MAP_POPULATE;
        /// Only meaningful when used with `MAP_POPULATE`. Don't perform read-ahead.
        #[cfg(linux_android)]
        MAP_NONBLOCK;
        /// Allocate the mapping using "huge pages."
        #[cfg(linux_android)]
        MAP_HUGETLB;
        /// Make use of 64KB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_64KB;
        /// Make use of 512KB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_512KB;
        /// Make use of 1MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_1MB;
        /// Make use of 2MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_2MB;
        /// Make use of 8MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_8MB;
        /// Make use of 16MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_16MB;
        /// Make use of 32MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_32MB;
        /// Make use of 256MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_256MB;
        /// Make use of 512MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_512MB;
        /// Make use of 1GB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_1GB;
        /// Make use of 2GB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_2GB;
        /// Make use of 16GB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_16GB;

        /// Lock the mapped region into memory as with `mlock(2)`.
        #[cfg(target_os = "netbsd")]
        MAP_WIRED;
        /// Causes dirtied data in the specified range to be flushed to disk only when necessary.
        #[cfg(freebsdlike)]
        MAP_NOSYNC;
        /// Rename private pages to a file.
        ///
        /// This was removed in FreeBSD 11 and is unused in DragonFlyBSD.
        #[cfg(netbsdlike)]
        MAP_RENAME;
        /// Region may contain semaphores.
        #[cfg(any(freebsdlike, netbsdlike))]
        MAP_HASSEMAPHORE;
        /// Region grows down, like a stack.
        #[cfg(any(linux_android, freebsdlike, target_os = "openbsd"))]
        MAP_STACK;
        /// Pages in this mapping are not retained in the kernel's memory cache.
        #[cfg(apple_targets)]
        MAP_NOCACHE;
        /// Allows the W/X bit on the page, it's necessary on aarch64 architecture.
        #[cfg(apple_targets)]
        MAP_JIT;
        /// Allows to use large pages, underlying alignment based on size.
        #[cfg(target_os = "freebsd")]
        MAP_ALIGNED_SUPER;
        /// Pages will be discarded in the core dumps.
        #[cfg(target_os = "openbsd")]
        MAP_CONCEAL;
        /// Attempt to place the mapping at exactly the address specified in `addr`.
        /// it's a default behavior on OpenBSD.
        #[cfg(netbsdlike)]
        MAP_TRYFIXED;
    }
}

impl MapFlags {
    /// Create `MAP_HUGETLB` with provided size of huge page.
    ///
    /// Under the hood it computes `MAP_HUGETLB | (huge_page_size_log2 << libc::MAP_HUGE_SHIFT`).
    /// `huge_page_size_log2` denotes logarithm of huge page size to use and should be
    /// between 16 and 63 (inclusively).
    ///
    /// ```
    /// # use nix::sys::mman::MapFlags;
    /// let f = MapFlags::map_hugetlb_with_size_log2(30).unwrap();
    /// assert_eq!(f, MapFlags::MAP_HUGETLB | MapFlags::MAP_HUGE_1GB);
    /// ```
    #[cfg(any(linux_android, target_os = "fuchsia"))]
    pub fn map_hugetlb_with_size_log2(
        huge_page_size_log2: u32,
    ) -> Option<Self> {
        if (16..=63).contains(&huge_page_size_log2) {
            let flag = libc::MAP_HUGETLB
                | (huge_page_size_log2 << libc::MAP_HUGE_SHIFT) as i32;
            Some(Self(flag.into()))
        } else {
            None
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "netbsd"))]
libc_bitflags! {
    /// Options for [`mremap`].
    pub struct MRemapFlags: c_int {
        /// Permit the kernel to relocate the mapping to a new virtual address, if necessary.
        #[cfg(target_os = "linux")]
        MREMAP_MAYMOVE;
        /// Place the mapping at exactly the address specified in `new_address`.
        #[cfg(target_os = "linux")]
        MREMAP_FIXED;
        /// Place the mapping at exactly the address specified in `new_address`.
        #[cfg(target_os = "netbsd")]
        MAP_FIXED;
        /// Allows to duplicate the mapping to be able to apply different flags on the copy.
        #[cfg(target_os = "netbsd")]
        MAP_REMAPDUP;
    }
}

libc_enum! {
    /// Usage information for a range of memory to allow for performance optimizations by the kernel.
    ///
    /// Used by [`madvise`].
    #[repr(i32)]
    #[non_exhaustive]
    pub enum MmapAdvise {
        /// No further special treatment. This is the default.
        MADV_NORMAL,
        /// Expect random page references.
        MADV_RANDOM,
        /// Expect sequential page references.
        MADV_SEQUENTIAL,
        /// Expect access in the near future.
        MADV_WILLNEED,
        /// Do not expect access in the near future.
        MADV_DONTNEED,
        /// Free up a given range of pages and its associated backing store.
        #[cfg(linux_android)]
        MADV_REMOVE,
        /// Do not make pages in this range available to the child after a `fork(2)`.
        #[cfg(linux_android)]
        MADV_DONTFORK,
        /// Undo the effect of `MADV_DONTFORK`.
        #[cfg(linux_android)]
        MADV_DOFORK,
        /// Poison the given pages.
        ///
        /// Subsequent references to those pages are treated like hardware memory corruption.
        #[cfg(linux_android)]
        MADV_HWPOISON,
        /// Enable Kernel Samepage Merging (KSM) for the given pages.
        #[cfg(linux_android)]
        MADV_MERGEABLE,
        /// Undo the effect of `MADV_MERGEABLE`
        #[cfg(linux_android)]
        MADV_UNMERGEABLE,
        /// Preserve the memory of each page but offline the original page.
        #[cfg(any(target_os = "android",
            all(target_os = "linux", any(
                target_arch = "aarch64",
                target_arch = "arm",
                target_arch = "powerpc",
                target_arch = "powerpc64",
                target_arch = "s390x",
                target_arch = "x86",
                target_arch = "x86_64",
                target_arch = "sparc64"))))]
        MADV_SOFT_OFFLINE,
        /// Enable Transparent Huge Pages (THP) for pages in the given range.
        #[cfg(linux_android)]
        MADV_HUGEPAGE,
        /// Undo the effect of `MADV_HUGEPAGE`.
        #[cfg(linux_android)]
        MADV_NOHUGEPAGE,
        /// Exclude the given range from a core dump.
        #[cfg(linux_android)]
        MADV_DONTDUMP,
        /// Undo the effect of an earlier `MADV_DONTDUMP`.
        #[cfg(linux_android)]
        MADV_DODUMP,
        /// Specify that the application no longer needs the pages in the given range.
        #[cfg(not(any(target_os = "aix", target_os = "hurd")))]
        MADV_FREE,
        /// Request that the system not flush the current range to disk unless it needs to.
        #[cfg(freebsdlike)]
        MADV_NOSYNC,
        /// Undoes the effects of `MADV_NOSYNC` for any future pages dirtied within the given range.
        #[cfg(freebsdlike)]
        MADV_AUTOSYNC,
        /// Region is not included in a core file.
        #[cfg(freebsdlike)]
        MADV_NOCORE,
        /// Include region in a core file
        #[cfg(freebsdlike)]
        MADV_CORE,
        /// This process should not be killed when swap space is exhausted.
        #[cfg(any(target_os = "freebsd"))]
        MADV_PROTECT,
        /// Invalidate the hardware page table for the given region.
        #[cfg(target_os = "dragonfly")]
        MADV_INVAL,
        /// Set the offset of the page directory page to `value` for the virtual page table.
        #[cfg(target_os = "dragonfly")]
        MADV_SETMAP,
        /// Indicates that the application will not need the data in the given range.
        #[cfg(apple_targets)]
        MADV_ZERO_WIRED_PAGES,
        /// Pages can be reused (by anyone).
        #[cfg(apple_targets)]
        MADV_FREE_REUSABLE,
        /// Caller wants to reuse those pages.
        #[cfg(apple_targets)]
        MADV_FREE_REUSE,
        // Darwin doesn't document this flag's behavior.
        #[cfg(apple_targets)]
        #[allow(missing_docs)]
        MADV_CAN_REUSE,
    }
}

libc_bitflags! {
    /// Configuration flags for [`msync`].
    pub struct MsFlags: c_int {
        /// Schedule an update but return immediately.
        MS_ASYNC;
        /// Invalidate all cached data.
        MS_INVALIDATE;
        /// Invalidate pages, but leave them mapped.
        #[cfg(apple_targets)]
        MS_KILLPAGES;
        /// Deactivate pages, but leave them mapped.
        #[cfg(apple_targets)]
        MS_DEACTIVATE;
        /// Perform an update and wait for it to complete.
        MS_SYNC;
    }
}

#[cfg(not(target_os = "haiku"))]
libc_bitflags! {
    /// Flags for [`mlockall`].
    pub struct MlockAllFlags: c_int {
        /// Lock pages that are currently mapped into the address space of the process.
        MCL_CURRENT;
        /// Lock pages which will become mapped into the address space of the process in the future.
        MCL_FUTURE;
    }
}

/// Locks all memory pages that contain part of the address range with `length`
/// bytes starting at `addr`.
///
/// Locked pages never move to the swap area.
///
/// # Safety
///
/// `addr` must meet all the requirements described in the [`mlock(2)`] man page.
///
/// [`mlock(2)`]: https://man7.org/linux/man-pages/man2/mlock.2.html
pub unsafe fn mlock(addr: NonNull<c_void>, length: size_t) -> Result<()> {
    unsafe { Errno::result(libc::mlock(addr.as_ptr(), length)).map(drop) }
}

/// Unlocks all memory pages that contain part of the address range with
/// `length` bytes starting at `addr`.
///
/// # Safety
///
/// `addr` must meet all the requirements described in the [`munlock(2)`] man
/// page.
///
/// [`munlock(2)`]: https://man7.org/linux/man-pages/man2/munlock.2.html
pub unsafe fn munlock(addr: NonNull<c_void>, length: size_t) -> Result<()> {
    unsafe { Errno::result(libc::munlock(addr.as_ptr(), length)).map(drop) }
}

/// Locks all memory pages mapped into this process' address space.
///
/// Locked pages never move to the swap area. For more information, see [`mlockall(2)`].
///
/// [`mlockall(2)`]: https://man7.org/linux/man-pages/man2/mlockall.2.html
#[cfg(not(target_os = "haiku"))]
pub fn mlockall(flags: MlockAllFlags) -> Result<()> {
    unsafe { Errno::result(libc::mlockall(flags.bits())) }.map(drop)
}

/// Unlocks all memory pages mapped into this process' address space.
///
/// For more information, see [`munlockall(2)`].
///
/// [`munlockall(2)`]: https://man7.org/linux/man-pages/man2/munlockall.2.html
#[cfg(not(target_os = "haiku"))]
pub fn munlockall() -> Result<()> {
    unsafe { Errno::result(libc::munlockall()) }.map(drop)
}

/// Allocate memory, or map files or devices into memory
///
/// For anonymous mappings (`MAP_ANON`/`MAP_ANONYMOUS`), see [mmap_anonymous].
///
/// # Safety
///
/// See the [`mmap(2)`] man page for detailed requirements.
///
/// [`mmap(2)`]: https://man7.org/linux/man-pages/man2/mmap.2.html
pub unsafe fn mmap<F: AsFd>(
    addr: Option<NonZeroUsize>,
    length: NonZeroUsize,
    prot: ProtFlags,
    flags: MapFlags,
    f: F,
    offset: off_t,
) -> Result<NonNull<c_void>> {
    let ptr = addr.map_or(std::ptr::null_mut(), |a| a.get() as *mut c_void);

    let fd = f.as_fd().as_raw_fd();
    let ret = unsafe {
        libc::mmap(ptr, length.into(), prot.bits(), flags.bits(), fd, offset)
    };

    if ret == libc::MAP_FAILED {
        Err(Errno::last())
    } else {
        // SAFETY: `libc::mmap` returns a valid non-null pointer or `libc::MAP_FAILED`, thus `ret`
        // will be non-null here.
        Ok(unsafe { NonNull::new_unchecked(ret) })
    }
}

/// Create an anonymous memory mapping.
///
/// This function is a wrapper around [`mmap`]:
/// `mmap(ptr, len, prot, MAP_ANONYMOUS | flags, -1, 0)`.
///
/// # Safety
///
/// See the [`mmap(2)`] man page for detailed requirements.
///
/// [`mmap(2)`]: https://man7.org/linux/man-pages/man2/mmap.2.html
pub unsafe fn mmap_anonymous(
    addr: Option<NonZeroUsize>,
    length: NonZeroUsize,
    prot: ProtFlags,
    flags: MapFlags,
) -> Result<NonNull<c_void>> {
    let ptr = addr.map_or(std::ptr::null_mut(), |a| a.get() as *mut c_void);

    let flags = MapFlags::MAP_ANONYMOUS | flags;
    let ret = unsafe {
        libc::mmap(ptr, length.into(), prot.bits(), flags.bits(), -1, 0)
    };

    if ret == libc::MAP_FAILED {
        Err(Errno::last())
    } else {
        // SAFETY: `libc::mmap` returns a valid non-null pointer or `libc::MAP_FAILED`, thus `ret`
        // will be non-null here.
        Ok(unsafe { NonNull::new_unchecked(ret) })
    }
}

/// Expands (or shrinks) an existing memory mapping, potentially moving it at
/// the same time.
///
/// # Safety
///
/// See the `mremap(2)` [man page](https://man7.org/linux/man-pages/man2/mremap.2.html) for
/// detailed requirements.
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
pub unsafe fn mremap(
    addr: NonNull<c_void>,
    old_size: size_t,
    new_size: size_t,
    flags: MRemapFlags,
    new_address: Option<NonNull<c_void>>,
) -> Result<NonNull<c_void>> {
    #[cfg(target_os = "linux")]
    let ret = unsafe {
        libc::mremap(
            addr.as_ptr(),
            old_size,
            new_size,
            flags.bits(),
            new_address
                .map(NonNull::as_ptr)
                .unwrap_or(std::ptr::null_mut()),
        )
    };
    #[cfg(target_os = "netbsd")]
    let ret = unsafe {
        libc::mremap(
            addr.as_ptr(),
            old_size,
            new_address
                .map(NonNull::as_ptr)
                .unwrap_or(std::ptr::null_mut()),
            new_size,
            flags.bits(),
        )
    };

    if ret == libc::MAP_FAILED {
        Err(Errno::last())
    } else {
        // SAFETY: `libc::mremap` returns a valid non-null pointer or `libc::MAP_FAILED`, thus `ret`
        // will be non-null here.
        Ok(unsafe { NonNull::new_unchecked(ret) })
    }
}

/// remove a mapping
///
/// # Safety
///
/// `addr` must meet all the requirements described in the [`munmap(2)`] man
/// page.
///
/// [`munmap(2)`]: https://man7.org/linux/man-pages/man2/munmap.2.html
pub unsafe fn munmap(addr: NonNull<c_void>, len: size_t) -> Result<()> {
    unsafe { Errno::result(libc::munmap(addr.as_ptr(), len)).map(drop) }
}

/// give advice about use of memory
///
/// # Safety
///
/// See the [`madvise(2)`] man page.  Take special care when using
/// [`MmapAdvise::MADV_FREE`].
///
/// [`madvise(2)`]: https://man7.org/linux/man-pages/man2/madvise.2.html
#[allow(rustdoc::broken_intra_doc_links)] // For Hurd as `MADV_FREE` is not available on it
pub unsafe fn madvise(
    addr: NonNull<c_void>,
    length: size_t,
    advise: MmapAdvise,
) -> Result<()> {
    unsafe {
        Errno::result(libc::madvise(addr.as_ptr(), length, advise as i32))
            .map(drop)
    }
}

/// Set protection of memory mapping.
///
/// See [`mprotect(3)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/mprotect.html) for
/// details.
///
/// # Safety
///
/// Calls to `mprotect` are inherently unsafe, as changes to memory protections can lead to
/// SIGSEGVs.
///
/// ```
/// # use nix::libc::size_t;
/// # use nix::sys::mman::{mmap_anonymous, mprotect, MapFlags, ProtFlags};
/// # use std::ptr;
/// # use std::os::unix::io::BorrowedFd;
/// const ONE_K: size_t = 1024;
/// let one_k_non_zero = std::num::NonZeroUsize::new(ONE_K).unwrap();
/// let mut slice: &mut [u8] = unsafe {
///     let mem = mmap_anonymous(None, one_k_non_zero, ProtFlags::PROT_NONE, MapFlags::MAP_PRIVATE)
///         .unwrap();
///     mprotect(mem, ONE_K, ProtFlags::PROT_READ | ProtFlags::PROT_WRITE).unwrap();
///     std::slice::from_raw_parts_mut(mem.as_ptr().cast(), ONE_K)
/// };
/// assert_eq!(slice[0], 0x00);
/// slice[0] = 0xFF;
/// assert_eq!(slice[0], 0xFF);
/// ```
pub unsafe fn mprotect(
    addr: NonNull<c_void>,
    length: size_t,
    prot: ProtFlags,
) -> Result<()> {
    unsafe {
        Errno::result(libc::mprotect(addr.as_ptr(), length, prot.bits()))
            .map(drop)
    }
}

/// synchronize a mapped region
///
/// # Safety
///
/// `addr` must meet all the requirements described in the [`msync(2)`] man
/// page.
///
/// [`msync(2)`]: https://man7.org/linux/man-pages/man2/msync.2.html
pub unsafe fn msync(
    addr: NonNull<c_void>,
    length: size_t,
    flags: MsFlags,
) -> Result<()> {
    unsafe {
        Errno::result(libc::msync(addr.as_ptr(), length, flags.bits()))
            .map(drop)
    }
}

#[cfg(not(target_os = "android"))]
feature! {
#![feature = "fs"]
/// Creates and opens a new, or opens an existing, POSIX shared memory object.
///
/// For more information, see [`shm_open(3)`].
///
/// [`shm_open(3)`]: https://man7.org/linux/man-pages/man3/shm_open.3.html
pub fn shm_open<P>(
    name: &P,
    flag: OFlag,
    mode: Mode
    ) -> Result<std::os::unix::io::OwnedFd>
    where P: ?Sized + NixPath
{
    use std::os::unix::io::{FromRawFd, OwnedFd};

    let ret = name.with_nix_path(|cstr| {
        #[cfg(apple_targets)]
        unsafe {
            libc::shm_open(cstr.as_ptr(), flag.bits(), mode.bits() as libc::c_uint)
        }
        #[cfg(not(apple_targets))]
        unsafe {
            libc::shm_open(cstr.as_ptr(), flag.bits(), mode.bits() as libc::mode_t)
        }
    })?;

    match ret {
        -1 => Err(Errno::last()),
        fd => Ok(unsafe{ OwnedFd::from_raw_fd(fd) })
    }
}
}

/// Performs the converse of [`shm_open`], removing an object previously created.
///
/// For more information, see [`shm_unlink(3)`].
///
/// [`shm_unlink(3)`]: https://man7.org/linux/man-pages/man3/shm_unlink.3.html
#[cfg(not(target_os = "android"))]
pub fn shm_unlink<P: ?Sized + NixPath>(name: &P) -> Result<()> {
    let ret =
        name.with_nix_path(|cstr| unsafe { libc::shm_unlink(cstr.as_ptr()) })?;

    Errno::result(ret).map(drop)
}

#[derive(Debug, Default, Clone, Copy)]
/// Type used to transform a raw number to an octal permission, while performing a clamp to u9
///
/// # Example
///
/// ```
/// # use nix::errno::Errno;
/// # use nix::sys::mman::Permissions;
///
/// # fn main() -> Result<(), Errno> {
/// assert_eq!(Permissions::new(511)?.get_permission(), &(0o0777 as u16));
/// assert_eq!(Permissions::new(512).expect_err("512 is bigger than what u9 can store"), Errno::E2BIG);
/// # Ok(())
/// # }
/// ```
pub struct Permissions {
    permission: u16,
}

impl Permissions {
    /// Create a new Permissions object
    ///
    /// Clamp to a u9 size, return Errno::E2BIG if it fails
    ///
    pub fn new(octal: u16) -> Result<Self> {
        if octal >= 2_u16.pow(9) {
            return Err(Errno::E2BIG);
        }
        Ok(Permissions { permission: octal })
    }

    /// Getter for permission
    ///
    pub fn get_permission(&self) -> &u16 {
        &self.permission
    }

    /// Using the current stored permission, do a bitor operation on the
    /// bitflags enums given
    ///
    pub fn to_octal<T: bitflags::Flags<Bits = i32>>(
        &self,
        vec_flags: Vec<T>,
    ) -> c_int {
        let mut flags: c_int = T::empty().bits();
        for flag in vec_flags {
            flags |= flag.bits();
        }
        flags |= self.permission as i32;
        flags
    }
}

libc_bitflags! {
    /// Valid flags for the third parameter of the function [`shmget`]
    pub struct ShmgetFlag: c_int
    {
        /// A new shared memory segment is created if key has this value.
        IPC_PRIVATE;
        /// Create a new segment.
        /// If this flag is not used, then shmget() will find the segment
        /// associated with key and check to see if the user has permission
        /// to access the segment.
        IPC_CREAT;
        /// This flag is used with IPC_CREAT to ensure that this call creates
        /// the segment.  If the segment already exists, the call fails.
        IPC_EXCL;
        /// Allocate the segment using "huge" pages.  See the Linux kernel
        /// source file Documentation/admin-guide/mm/hugetlbpage.rst for
        /// further information.
        #[cfg(any(target_os = "linux"))]
        SHM_HUGETLB;
        // TODO: Does not exist in libc/linux, but should? Maybe open an issue in their repo
        // SHM_HUGE_2MB;
        // TODO: Same for this one
        // SHM_HUGE_1GB;
        /// This flag serves the same purpose as the mmap(2) MAP_NORESERVE flag.
        /// Do not reserve swap space for this segment. When swap space is
        /// reserved, one has the guarantee that it is possible to modify the
        /// segment. When swap space is not reserved one might get SIGSEGV upon
        /// a write if no physical memory is available. See also the discussion
        /// of the file /proc/sys/vm/overcommit_memory in proc(5).
        #[cfg(any(target_os = "linux"))]
        SHM_NORESERVE;
    }
}
/// Creates and returns a new, or returns an existing, System V shared memory
/// segment identifier.
///
/// For more information, see [`shmget(2)`].
///
/// [`shmget(2)`]: https://man7.org/linux/man-pages/man2/shmget.2.html
pub fn shmget(
    key: key_t,
    size: size_t,
    shmflg: Vec<ShmgetFlag>,
    permission: Permissions,
) -> Result<i32> {
    let flags = permission.to_octal(shmflg);
    Errno::result(unsafe { libc::shmget(key, size, flags) })
}

libc_bitflags! {
    /// Valid flags for the third parameter of the function [`semget`]
    pub struct SemgetFlag: c_int
    {
        /// A new shared memory segment is created if key has this value
        IPC_PRIVATE;
        /// Create a new segment.
        /// If this flag is not used, then shmget() will find the segment
        /// associated with key and check to see if the user has permission
        /// to access the segment.
        IPC_CREAT;
        /// This flag is used with IPC_CREAT to ensure that this call creates
        /// the segment. If the segment already exists, the call fails.
        IPC_EXCL;
    }
}
/// Creates and return a new, or returns an existing, System V shared memory
/// semaphore identifier.
///
/// For more information, see [`semget(2)`].
///
/// [`semget(2)`]: https://man7.org/linux/man-pages/man2/semget.2.html
pub fn semget(
    key: key_t,
    size: i32,
    semflg: Vec<SemgetFlag>,
    permission: Permissions,
) -> Result<i32> {
    let flags = permission.to_octal(semflg);
    Errno::result(unsafe { libc::semget(key, size, flags) })
}

libc_bitflags! {
    /// Valid flags for the third parameter of the function [`shmat`]
    pub struct ShmatFlag: c_int
    {
        /// Allow the contents of the segment to be executed. The caller must
        /// have execute permission on the segment.
        #[cfg(any(target_os = "linux"))]
        SHM_EXEC;
        #[cfg(any(target_os = "linux"))]
        /// This flag specifies that the mapping of the segment should replace
        /// any existing mapping in the range starting at shmaddr and
        /// continuing for the size of the segment.
        /// (Normally, an EINVAL error would result if a mapping already exists
        /// in this address range.)
        /// In this case, shmaddr must not be NULL.
        SHM_REMAP;
        /// Attach the segment for read-only access. The process must have read
        /// permission for the segment. If this flag is not specified, the
        /// segment is attached for read and write access, and the process must
        /// have read and write permission for the segment.
        /// There is no notion of a write-only shared memory segment.
        SHM_RDONLY;
        /// TODO: I have no clue at what this does
        SHM_RND;
    }
}
/// Attaches the System V shared memory segment identified by `shmid` to the
/// address space of the calling process.
///
/// For more information, see [`shmat(2)`].
///
/// # Safety
///
/// `shmid` should be a valid shared memory identifier and
/// `shmaddr` must meet the requirements described in the [`shmat(2)`] man page.
///
/// [`shmat(2)`]: https://man7.org/linux/man-pages/man2/shmat.2.html
pub fn shmat(
    shmid: c_int,
    shmaddr: *const c_void,
    shmflg: Vec<ShmatFlag>,
    permission: Permissions,
) -> Result<*mut c_void> {
    let flags = permission.to_octal(shmflg);
    Errno::result(unsafe { libc::shmat(shmid, shmaddr, flags) })
}

/// Performs the reverse of [`shmat`], detaching the shared memory segment at
/// the given address from the address space of the calling process.
///
/// For more information, see [`shmdt(2)`].
///
/// # Safety
///
/// `shmaddr` must meet the requirements described in the [`shmdt(2)`] man page.
///
/// [`shmdt(2)`]: https://man7.org/linux/man-pages/man2/shmdt.2.html
pub fn shmdt(shmaddr: *const c_void) -> Result<()> {
    Errno::result(unsafe { libc::shmdt(shmaddr) }).map(drop)
}

libc_bitflags! {
    /// Valid flags for the second parameter of the function [`shmctl`]
    pub struct ShmctlFlag: c_int {
        #[cfg(any(target_os = "linux"))]
        /// Returns the index of the highest used entry in the kernel's internal
        /// array recording information about all shared memory segment
        IPC_INFO;
        /// Write the values of some members of the shmid_ds structure pointed
        /// to by buf to the kernel data structure associated with this shared
        /// memory segment, updating also its shm_ctime member.
        ///
        /// The following fields are updated: shm_perm.uid,
        /// shm_perm.gid, and (the least significant 9 bits of)
        /// shm_perm.mode.
        ///
        /// The effective UID of the calling process must match the owner
        /// (shm_perm.uid) or creator (shm_perm.cuid) of the shared memory
        /// segment, or the caller must be privileged.
        IPC_SET;
        /// Copy information from the kernel data structure associated with
        /// shmid into the shmid_ds structure pointed to by buf.
        /// The caller must have read permission on the shared memory segment.
        IPC_STAT;
        /// Mark the segment to be destroyed. The segment will actually be
        /// destroyed only after the last process detaches it
        /// (i.e., when the shm_nattch member of the associated structure
        /// shmid_ds is zero).
        /// The caller must be the owner or creator of the segment,
        /// or be privileged. The buf argument is ignored.
        ///
        /// If a segment has been marked for destruction, then the
        /// (nonstandard) SHM_DEST flag of the shm_perm.mode field in the
        /// associated data structure retrieved by IPC_STAT will be set.
        ///
        /// The caller must ensure that a segment is eventually destroyed;
        /// otherwise its pages that were faulted in will remain in memory
        /// or swap.
        ///
        /// See also the description of /proc/sys/kernel/shm_rmid_forced
        /// in proc(5).
        IPC_RMID;
        // not available in libc/linux, but should be?
        // #[cfg(any(target_os = "linux"))]
        // SHM_INFO;
        // #[cfg(any(target_os = "linux"))]
        // SHM_STAT;
        // #[cfg(any(target_os = "linux"))]
        // SHM_STAT_ANY;
        #[cfg(any(target_os = "linux"))]
        /// Prevent swapping of the shared memory segment. The caller must
        /// fault in any pages that are required to be present after locking is
        /// enabled.
        /// If a segment has been locked, then the (nonstandard) SHM_LOCKED
        /// flag of the shm_perm.mode field in the associated data structure
        /// retrieved by IPC_STAT will be set.
        SHM_LOCK;
        #[cfg(any(target_os = "linux"))]
        /// Unlock the segment, allowing it to be swapped out.
        SHM_UNLOCK;
    }
}
/// Performs control operation specified by `cmd` on the System V shared
/// memory segment given by `shmid`.
///
/// For more information, see [`shmctl(2)`].
///
/// # Safety
///
/// All arguments should be valid and meet the requirements described in the [`shmctl(2)`] man page.
///
/// [`shmctl(2)`]: https://man7.org/linux/man-pages/man2/shmctl.2.html
pub fn shmctl(
    shmid: c_int,
    cmd: ShmctlFlag,
    buf: *mut shmid_ds,
    permission: Permissions,
) -> Result<c_int> {
    let command = permission.to_octal(vec![cmd]);
    Errno::result(unsafe { libc::shmctl(shmid, command, buf) })
}

#[derive(Debug)]
/// Called as the fourth parameter of the function [`semctl`]
/// 
pub enum Semun {
    /// Value for SETVAL
    val(c_int),
    /// Buffer for IPC_STAT, IPC_SET
    buf(*mut semid_ds),
    /// Array for GETALL, SETALL
    array(*mut c_short),
    /// Buffer for IPC_INFO
    #[cfg(any(target_os = "linux"))]
    __buf(*mut seminfo),
}
libc_bitflags! {
    /// Valid flags for the third parameter of the function [`shmctl`]
    pub struct SemctlCmd: c_int {
        /// Copy information from the kernel data structure associated with
        /// shmid into the shmid_ds structure pointed to by buf.
        /// The caller must have read permission on the shared memory segment.
        IPC_STAT;
        /// Write the values of some members of the semid_ds structure pointed
        /// to by arg.buf to the kernel data structure associated with this
        /// semaphore set, updating also its sem_ctime member.
        /// 
        /// The following members of the structure are updated:
        /// sem_perm.uid, sem_perm.gid, and (the least significant 9 bits of)
        /// sem_perm.mode.
        /// 
        /// The effective UID of the calling process must match the owner
        /// (sem_perm.uid) or creator (sem_perm.cuid) of the semaphore set,
        /// or the caller must be privileged. The argument semnum is ignored.
        IPC_SET;
        /// Immediately remove the semaphore set, awakening all processes
        /// blocked in semop(2) calls on the set
        /// (with an error return and errno set to EIDRM).
        /// The effective user ID of the calling process must match the creator
        /// or owner of the semaphore set, or the caller must be privileged.
        /// The argument semnum is ignored.
        IPC_RMID;
        #[cfg(any(target_os = "linux"))]
        /// Return information about system-wide semaphore limits and
        /// parameters in the structure pointed to by arg.__buf. This structure
        /// is of type [`seminfo`].
        IPC_INFO;
        // TODO: None of the one following are defined in libc
        // #[cfg(any(target_os = "linux"))]
        // SEM_INFO;
        // #[cfg(any(target_os = "linux"))]
        // SEM_STAT;
        // #[cfg(any(target_os = "linux"))]
        // SEM_STAT_ANY;
        // GETALL;
        // GETNCNT;
        // GETPID;
        // GETVAL;
        // GETZCNT;
        // SETALL;
        // SETVAL;
    }
}
/// Performs control operation specified by `cmd` on the System V shared
/// semaphore segment given by `semid`.
///
/// For more information, see [`semctl(2)`].
///
/// #
///
/// [`semctl(2)`]: https://man7.org/linux/man-pages/man2/semctl.2.html
pub fn semctl(
    semid: c_int,
    semnum: c_int,
    cmd: SemctlCmd,
    permission: Permissions,
    semun: Option<Semun>,
) -> Result<c_int> {
    let command = permission.to_octal(vec![cmd]);
    if semun.is_none() {
        return Errno::result(unsafe { libc::semctl(semid, semnum, command) });
    }
    Errno::result(unsafe { libc::semctl(semid, semnum, command, semun) })
}
