//! List directory contents

use crate::errno::Errno;
use crate::fcntl::{self, OFlag};
use crate::sys;
use crate::{NixPath, Result};
use cfg_if::cfg_if;
use std::ffi;
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
use std::ptr;

#[cfg(target_os = "linux")]
use libc::{dirent64 as dirent, readdir64_r as readdir_r};

#[cfg(not(target_os = "linux"))]
use libc::{dirent, readdir_r};

/// An open directory.
///
/// This is a lower-level interface than [`std::fs::ReadDir`]. Notable differences:
///    * can be opened from a file descriptor (as returned by [`openat`][openat],
///      perhaps before knowing if the path represents a file or directory).
///    * implements [`AsFd`][AsFd], so it can be passed to [`fstat`][fstat],
///      [`openat`][openat], etc. The file descriptor continues to be owned by the
///      `Dir`, so callers must not keep a `RawFd` after the `Dir` is dropped.
///    * can be iterated through multiple times without closing and reopening the file
///      descriptor. Each iteration rewinds when finished.
///    * returns entries for `.` (current directory) and `..` (parent directory).
///    * returns entries' names as a [`CStr`][cstr] (no allocation or conversion beyond whatever libc
///      does).
///
/// [AsFd]: std::os::fd::AsFd
/// [fstat]: crate::sys::stat::fstat
/// [openat]: crate::fcntl::openat
/// [cstr]: std::ffi::CStr
///
/// # Examples
///
/// Traverse the current directory, and print entries' names:
///
/// ```
/// use nix::dir::Dir;
/// use nix::fcntl::OFlag;
/// use nix::sys::stat::Mode;
///
/// let mut cwd = Dir::open(".", OFlag::O_RDONLY | OFlag::O_CLOEXEC, Mode::empty()).unwrap();
/// for res_entry in cwd.iter() {
///     let entry = res_entry.unwrap();
///     println!("File name: {}", entry.file_name().to_string_lossy());
/// }
/// ```
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Dir(ptr::NonNull<libc::DIR>);

impl Dir {
    /// Opens the given path as with `fcntl::open`.
    pub fn open<P: ?Sized + NixPath>(
        path: &P,
        oflag: OFlag,
        mode: sys::stat::Mode,
    ) -> Result<Self> {
        let fd = fcntl::open(path, oflag, mode)?;
        Dir::from_fd(fd)
    }

    /// Opens the given path as with `fcntl::openat`.
    pub fn openat<Fd: std::os::fd::AsFd, P: ?Sized + NixPath>(
        dirfd: Fd,
        path: &P,
        oflag: OFlag,
        mode: sys::stat::Mode,
    ) -> Result<Self> {
        let fd = fcntl::openat(dirfd, path, oflag, mode)?;
        Dir::from_fd(fd)
    }

    /// Converts from a descriptor-based object, closing the descriptor on success or failure.
    ///
    /// # Safety
    ///
    /// It is only safe if `fd` is an owned file descriptor.
    #[inline]
    #[deprecated(
        since = "0.30.0",
        note = "Deprecate this since it is not I/O-safe, use from_fd instead."
    )]
    pub unsafe fn from<F: IntoRawFd>(fd: F) -> Result<Self> {
        use std::os::fd::FromRawFd;
        use std::os::fd::OwnedFd;

        // SAFETY:
        //
        // This is indeed unsafe is `fd` it not an owned fd.
        let owned_fd = unsafe { OwnedFd::from_raw_fd(fd.into_raw_fd()) };
        Dir::from_fd(owned_fd)
    }

    /// Converts from a file descriptor, closing it on failure.
    ///
    /// # Examples
    ///
    /// `ENOTDIR` would be returned if `fd` does not refer to a directory:
    ///
    /// ```should_panic
    /// use std::os::fd::OwnedFd;
    /// use nix::dir::Dir;
    ///
    /// let temp_file = tempfile::tempfile().unwrap();
    /// let temp_file_fd: OwnedFd = temp_file.into();
    /// let never = Dir::from_fd(temp_file_fd).unwrap();
    /// ```
    #[doc(alias("fdopendir"))]
    pub fn from_fd(fd: std::os::fd::OwnedFd) -> Result<Self> {
        // take the ownership as the constructed `Dir` is now the owner
        let raw_fd = fd.into_raw_fd();
        let d = ptr::NonNull::new(unsafe { libc::fdopendir(raw_fd) })
            .ok_or(Errno::last())?;
        Ok(Dir(d))
    }

    /// Returns an iterator of `Result<Entry>` which rewinds when finished.
    pub fn iter(&mut self) -> Iter<'_> {
        Iter(self)
    }
}

// `Dir` is not `Sync`. With the current implementation, it could be, but according to
// https://www.gnu.org/software/libc/manual/html_node/Reading_002fClosing-Directory.html,
// future versions of POSIX are likely to obsolete `readdir_r` and specify that it's unsafe to
// call `readdir` simultaneously from multiple threads.
//
// `Dir` is safe to pass from one thread to another, as it's not reference-counted.
unsafe impl Send for Dir {}

impl std::os::fd::AsFd for Dir {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        let raw_fd = self.as_raw_fd();

        // SAFETY:
        //
        // `raw_fd` should be open and valid for the lifetime of the returned
        // `BorrowedFd` as it is extracted from `&self`.
        unsafe { std::os::fd::BorrowedFd::borrow_raw(raw_fd) }
    }
}

impl AsRawFd for Dir {
    fn as_raw_fd(&self) -> RawFd {
        unsafe { libc::dirfd(self.0.as_ptr()) }
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let e = Errno::result(unsafe { libc::closedir(self.0.as_ptr()) });
        if !std::thread::panicking() && e == Err(Errno::EBADF) {
            panic!("Closing an invalid file descriptor!");
        };
    }
}

// The pass by mut is technically needless only because the inner NonNull is
// Copy.  But philosophically we're mutating the Dir, so we pass by mut.
#[allow(clippy::needless_pass_by_ref_mut)]
fn next(dir: &mut Dir) -> Option<Result<Entry>> {
    unsafe {
        // Note: POSIX specifies that portable applications should dynamically allocate a
        // buffer with room for a `d_name` field of size `pathconf(..., _PC_NAME_MAX)` plus 1
        // for the NUL byte. It doesn't look like the std library does this; it just uses
        // fixed-sized buffers (and libc's dirent seems to be sized so this is appropriate).
        // Probably fine here too then.
        let mut ent = std::mem::MaybeUninit::<dirent>::uninit();
        let mut result = ptr::null_mut();
        if let Err(e) = Errno::result(readdir_r(
            dir.0.as_ptr(),
            ent.as_mut_ptr(),
            &mut result,
        )) {
            return Some(Err(e));
        }
        if result.is_null() {
            return None;
        }
        assert_eq!(result, ent.as_mut_ptr());
        Some(Ok(Entry(ent.assume_init())))
    }
}

/// Return type of [`Dir::iter`].
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Iter<'d>(&'d mut Dir);

impl Iterator for Iter<'_> {
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        next(self.0)
    }
}

impl Drop for Iter<'_> {
    fn drop(&mut self) {
        unsafe { libc::rewinddir((self.0).0.as_ptr()) }
    }
}

/// The return type of [Dir::into_iter]
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct OwningIter(Dir);

impl Iterator for OwningIter {
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        next(&mut self.0)
    }
}

/// The file descriptor continues to be owned by the `OwningIter`,
/// so callers must not keep a `RawFd` after the `OwningIter` is dropped.
impl AsRawFd for OwningIter {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoIterator for Dir {
    type Item = Result<Entry>;
    type IntoIter = OwningIter;

    /// Creates a owning iterator, that is, one that takes ownership of the
    /// `Dir`. The `Dir` cannot be used after calling this.  This can be useful
    /// when you have a function that both creates a `Dir` instance and returns
    /// an `Iterator`.
    ///
    /// Example:
    ///
    /// ```
    /// use nix::{dir::Dir, fcntl::OFlag, sys::stat::Mode};
    /// use std::{iter::Iterator, string::String};
    ///
    /// fn ls_upper(dirname: &str) -> impl Iterator<Item=String> {
    ///     let d = Dir::open(dirname, OFlag::O_DIRECTORY, Mode::S_IXUSR).unwrap();
    ///     d.into_iter().map(|x| x.unwrap().file_name().as_ref().to_string_lossy().to_ascii_uppercase())
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        OwningIter(self)
    }
}

/// A directory entry, similar to `std::fs::DirEntry`.
///
/// Note that unlike the std version, this may represent the `.` or `..` entries.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct Entry(dirent);

/// Type of file referenced by a directory entry
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    /// FIFO (Named pipe)
    Fifo,
    /// Character device
    CharacterDevice,
    /// Directory
    Directory,
    /// Block device
    BlockDevice,
    /// Regular file
    File,
    /// Symbolic link
    Symlink,
    /// Unix-domain socket
    Socket,
}

impl Entry {
    /// Returns the inode number (`d_ino`) of the underlying `dirent`.
    #[allow(clippy::useless_conversion)] // Not useless on all OSes
    // The cast is not unnecessary on all platforms.
    #[allow(clippy::unnecessary_cast)]
    pub fn ino(&self) -> u64 {
        cfg_if! {
            if #[cfg(any(target_os = "aix",
                         target_os = "emscripten",
                         target_os = "fuchsia",
                         target_os = "haiku",
                         target_os = "hurd",
                         target_os = "cygwin",
                         solarish,
                         linux_android,
                         apple_targets))] {
                self.0.d_ino as u64
            } else {
                u64::from(self.0.d_fileno)
            }
        }
    }



    /// Returns the bare file name of this directory entry without any other leading path component.
    /// 
     #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    ///PLEASE REMOVE THESE COMMENTS (OBVIOUSLY WHEN DONE))
    /// This utilises a constant-time constant function strlen implementation that's faster than `libc::strlen`` (std library internal implementation)
    /// The function used is described at the bottom of the file. Benchmarks at 
    pub  const fn file_name(&self) -> &ffi::CStr {
        let str_length = unsafe { dirent_const_time_strlen(&self.0)+1 };
        unsafe{
            //we need to transmute here because we're trying to match the original type of this implementation to not break it
            std::mem::transmute(&*std::ptr::slice_from_raw_parts(self.0.d_name.as_ptr() as  *const u8, str_length))


        //unsafe { ffi::CStr::from_ptr(self.0.d_name.as_ptr()) }
    }
}

    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    pub fn file_name(&self) -> &ffi::CStr {
        unsafe { ffi::CStr::from_ptr(self.0.d_name.as_ptr()) }
    }

    /// Returns the type of this directory entry, if known.
    ///
    /// See platform `readdir(3)` or `dirent(5)` manpage for when the file type is known;
    /// notably, some Linux filesystems don't implement this. The caller should use `stat` or
    /// `fstat` if this returns `None`.
    pub fn file_type(&self) -> Option<Type> {
        #[cfg(not(any(solarish, target_os = "aix", target_os = "haiku")))]
        match self.0.d_type {
            libc::DT_FIFO => Some(Type::Fifo),
            libc::DT_CHR => Some(Type::CharacterDevice),
            libc::DT_DIR => Some(Type::Directory),
            libc::DT_BLK => Some(Type::BlockDevice),
            libc::DT_REG => Some(Type::File),
            libc::DT_LNK => Some(Type::Symlink),
            libc::DT_SOCK => Some(Type::Socket),
            /* libc::DT_UNKNOWN | */ _ => None,
        }

        // illumos, Solaris, and Haiku systems do not have the d_type member at all:
        #[cfg(any(solarish, target_os = "aix", target_os = "haiku"))]
        None
    }
}





#[inline]
#[allow(clippy::ptr_as_ptr)] //safe to do this as u8 is aligned to 8 bytes
#[allow(clippy::cast_lossless)] //shutup
/// Const-fn strlen for dirent's `d_name` field using bit tricks, no SIMD, no overreads!!!
/// O(1) complexity, no branching, and no loops.
///
/// This function can't really be used in a const manner, I just took the win where I could! ( I thought it was cool too...)
/// It's probably the most efficient way to calculate the length
/// It calculates the length of the `d_name` field in a `libc::dirent64` structure without branching on the presence of null bytes.
/// It needs to be used on  a VALID `libc::dirent64` pointer, and it assumes that the `d_name` field is null-terminated.
/// Refererence<  https://graphics.stanford.edu/~seander/bithacks.html#HasZeroByte>    
///                        
/// Reference <https://github.com/Soveu/find/blob/master/src/dirent.rs>          
///
///
/// Main idea:
/// - We read the last 8 bytes of the `d_name` field, which is guaranteed to be null-terminated by the kernel.
/// This is based on the observation that d_name is always null-terminated by the kernel,
///  so we only need to scan at most 255 bytes. However, since we read the last 8 bytes and apply bit tricks,
/// we can locate the null terminator with a single 64-bit read and mask, assuming alignment and endianness.
///                    
/// Combining all these tricks, i made this beautiful thing!
/// # SAFETY
/// This function is `unsafe` because it involves dereferencing raw pointers, pointer arithmetic, structure offsets...just everything.
/// The caller must uphold the following invariants:
/// - The `dirent` pointer must point to a valid `libc::dirent64` structure
///  `SWAR` (SIMD Within A Register) is used to find the first null byte in the `d_name` field of a `libc::dirent64` structure.
pub const unsafe fn dirent_const_time_strlen(dirent: *const libc::dirent64) -> usize {
    const DIRENT_HEADER_START: usize = std::mem::offset_of!(libc::dirent64, d_name) + 1; //we're going backwards(to the start of d_name) so we add 1 to the offset
    let reclen = unsafe { (*dirent).d_reclen as usize}; 
    //find the record-length
    // THIS WILL ONLY WORK ON LITTLE-ENDIAN ARCHITECTURES, I CANT BE BOTHERED TO FIGURE THAT OUT, qemu isnt fun
    // Calculate the  start of the d_name field
    let last_word = unsafe { *((dirent as *const u8).add(reclen - 8) as *const u64) };
    // Special case: When processing the 3rd u64 word (index 2), we need to mask
    // the non-name bytes (d_type and padding) to avoid false null detection.
    // The 0x00FF_FFFF mask preserves only the 3 bytes where the name could start.
    // Branchless masking: avoids branching by using a mask that is either 0 or 0x00FF_FFFF
    unsafe{std::hint::assert_unchecked(reclen % 8 ==0 && reclen >=24 )}; //tell the compiler is a multiple of 8 and within bounds
    //this is safe because the kernel guarantees the above.
    //............................//special case short name check
    let mask = 0x00FF_FFFFu64 * ((reclen ==24) as u64); // (multiply by 0 or 1)
    // The mask is applied to the last word to isolate the relevant bytes.
    // The last word is masked to isolate the relevant bytes,
    //we're bit manipulating the last word (a byte/u64) to find the first null byte
    //this boils to a complexity of strlen over 8 bytes, which we then accomplish with a bit trick
    // The mask is applied to the last word to isolate the relevant bytes.
    // The last word is masked to isolate the relevant bytes, and then we find the first zero byte.
    let candidate_pos = last_word | mask;
    // The resulting value (`candidate_pos`) has:
    // - Original name bytes preserved
    // - Non-name bytes forced to 0xFF (guaranteed non-zero)
    // - Maintains the exact position of any null bytes in the name
    let zero_bit = candidate_pos.wrapping_sub(0x0101_0101_0101_0101)// 0x0101_0101_0101_0101 -> underflows the high bit if a byte is zero
        & !candidate_pos//ensures only bytes that were zero retain the underflowed high bit.
        & 0x8080_8080_8080_8080; //  0x8080_8080_8080_8080 -->This masks out the high bit of each byte, so we can find the first zero byte
    // The trailing zeros of the zero_bit gives us the position of the first zero byte.
    // We divide by 8 to convert the bit position to a byte position..
    // We subtract 7 to get the correct offset in the d_name field.
    //>> 3 converts from bit position to byte index (divides by 8)
    

    reclen  - DIRENT_HEADER_START - (7 - (zero_bit.trailing_zeros() >> 3) as usize)
}
