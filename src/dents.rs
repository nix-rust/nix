//! Raw directory iteration using Linux's getdents syscall

use crate::errno::Errno;
use crate::file_type::FileType;
use std::cmp::max;
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::os::unix::io::AsFd;
use std::{mem, slice};

/// A directory iterator implemented with getdents.
///
/// This implementation:
/// - Excludes deleted inodes (with ID 0).
/// - Does not handle growing the buffer. If this functionality is necessary,
///   you'll need to drop the current iterator, resize the buffer, and then
///   re-create the iterator. The iterator is guaranteed to continue where it
///   left off provided the file descriptor isn't changed. See the example in
///   [`RawDir::new`].
#[derive(Debug)]
pub struct RawDir<'buf, Fd: AsFd> {
    fd: Fd,
    buf: &'buf mut [MaybeUninit<u8>],
    initialized: usize,
    offset: usize,
}

impl<'buf, Fd: AsFd> RawDir<'buf, Fd> {
    /// Create a new iterator from the given file descriptor and buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::mem::MaybeUninit;
    /// # use std::os::unix::io::{AsFd, FromRawFd, OwnedFd};
    /// # use nix::dents::RawDir;
    /// # use nix::errno::Errno;
    /// # use nix::fcntl::{OFlag, open, openat};
    /// # use nix::sys::stat::Mode;
    ///
    /// let fd = open(".", OFlag::O_RDONLY | OFlag::O_DIRECTORY, Mode::empty()).unwrap();
    /// let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    ///
    /// let mut buf = [MaybeUninit::uninit(); 2048];
    ///
    /// for entry in RawDir::new(fd, &mut buf) {
    ///     let entry = entry.unwrap();
    ///     dbg!(&entry);
    /// }
    /// ```
    ///
    /// Contrived example that demonstrates reading entries with arbitrarily large file paths:
    ///
    /// ```
    /// # use std::cmp::max;
    /// # use std::mem::MaybeUninit;
    /// # use std::os::unix::io::{AsFd, FromRawFd, OwnedFd};
    /// # use nix::dents::RawDir;
    /// # use nix::errno::Errno;
    /// # use nix::fcntl::{OFlag, open, openat};
    /// # use nix::sys::stat::Mode;
    ///
    /// let fd = open(".", OFlag::O_RDONLY | OFlag::O_DIRECTORY, Mode::empty()).unwrap();
    /// let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    ///
    /// // DO NOT DO THIS. Use `Vec::with_capacity` to at least start the buffer
    /// // off with *some* space.
    /// let mut buf = Vec::new();
    ///
    /// 'read: loop {
    ///     'resize: {
    ///         for entry in RawDir::new(&fd, buf.spare_capacity_mut()) {
    ///             let entry = match entry {
    ///                 Err(Errno::EINVAL) => break 'resize,
    ///                 r => r.unwrap(),
    ///             };
    ///             dbg!(&entry);
    ///         }
    ///         break 'read;
    ///     }
    ///
    ///     let new_capacity = max(buf.capacity() * 2, 1);
    ///     buf.reserve(new_capacity);
    /// }
    /// ```
    ///
    /// Note that this is horribly inefficient as we'll most likely end up doing ~1 syscall per file.
    pub fn new(fd: Fd, buf: &'buf mut [MaybeUninit<u8>]) -> Self {
        Self {
            fd,
            buf,
            initialized: 0,
            offset: 0,
        }
    }
}

/// A raw directory entry, similar to `std::fs::DirEntry`.
///
/// Note that unlike the std version, this may represent the `.` or `..` entries.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct RawDirEntry<'a> {
    pub inode_number: u64,
    pub file_type: FileType,
    pub name: &'a CStr,
}

#[repr(C, packed)]
struct dirent64 {
    d_ino: libc::ino64_t,
    d_off: libc::off64_t,
    d_reclen: libc::c_ushort,
    d_type: libc::c_uchar,
}

impl<'buf, Fd: AsFd> Iterator for RawDir<'buf, Fd> {
    type Item = Result<RawDirEntry<'buf>, Errno>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.offset < self.initialized {
                let dirent_ptr =
                    &self.buf[self.offset] as *const MaybeUninit<u8>;
                // Trust the kernel to use proper alignment
                #[allow(clippy::cast_ptr_alignment)]
                let dirent = unsafe { &*dirent_ptr.cast::<dirent64>() };

                self.offset += dirent.d_reclen as usize;
                if dirent.d_ino == 0 {
                    continue;
                }

                return Some(Ok(RawDirEntry {
                    inode_number: dirent.d_ino,
                    file_type: FileType::from(dirent.d_type),
                    name: unsafe {
                        let name_start =
                            dirent_ptr.add(mem::size_of::<dirent64>());
                        let mut name_end = {
                            // Find the last aligned byte of the file name so we can
                            // start searching for NUL bytes. If we started searching
                            // from the back, we would run into garbage left over from
                            // previous iterations.
                            // TODO use .map_addr() once strict_provenance is stable
                            let addr = max(
                                name_start as usize,
                                dirent_ptr.add(dirent.d_reclen as usize - 1)
                                    as usize
                                    & !(mem::size_of::<usize>() - 1),
                            );
                            addr as *const u8
                        };

                        while *name_end != 0 {
                            name_end = name_end.add(1);
                        }

                        CStr::from_bytes_with_nul_unchecked(
                            slice::from_raw_parts(
                                name_start.cast::<u8>(),
                                // Add 1 for the NUL byte
                                // TODO use .addr() once strict_provenance is stable
                                name_end as usize - name_start as usize + 1,
                            ),
                        )
                    },
                }));
            }
            self.initialized = 0;
            self.offset = 0;

            match unsafe {
                Errno::result(libc::syscall(
                    libc::SYS_getdents64,
                    self.fd.as_fd(),
                    self.buf.as_mut_ptr(),
                    self.buf.len(),
                ))
            } {
                Ok(bytes_read) if bytes_read == 0 => return None,
                Ok(bytes_read) => self.initialized = bytes_read as usize,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
