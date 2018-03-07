//! POSIX Asynchronous I/O
//!
//! The POSIX AIO interface is used for asynchronous I/O on files and disk-like
//! devices.  It supports [`read`](struct.AioCb.html#method.read),
//! [`write`](struct.AioCb.html#method.write), and
//! [`fsync`](struct.AioCb.html#method.fsync) operations.  Completion
//! notifications can optionally be delivered via
//! [signals](../signal/enum.SigevNotify.html#variant.SigevSignal), via the
//! [`aio_suspend`](fn.aio_suspend.html) function, or via polling.  Some
//! platforms support other completion
//! notifications, such as
//! [kevent](../signal/enum.SigevNotify.html#variant.SigevKevent).
//!
//! Multiple operations may be submitted in a batch with
//! [`lio_listio`](fn.lio_listio.html), though the standard does not guarantee
//! that they will be executed atomically.
//!
//! Outstanding operations may be cancelled with
//! [`cancel`](struct.AioCb.html#method.cancel) or
//! [`aio_cancel_all`](fn.aio_cancel_all.html), though the operating system may
//! not support this for all filesystems and devices.

use {Error, Result};
use bytes::{Bytes, BytesMut};
use errno::Errno;
use std::os::unix::io::RawFd;
use libc::{c_void, off_t, size_t};
use libc;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ptr::{null, null_mut};
use std::thread;
use sys::signal::*;
use sys::time::TimeSpec;

libc_enum! {
    /// Mode for `AioCb::fsync`.  Controls whether only data or both data and
    /// metadata are synced.
    #[repr(i32)]
    pub enum AioFsyncMode {
        /// do it like `fsync`
        O_SYNC,
        /// on supported operating systems only, do it like `fdatasync`
        #[cfg(any(target_os = "ios",
                  target_os = "linux",
                  target_os = "macos",
                  target_os = "netbsd",
                  target_os = "openbsd"))]
        O_DSYNC
    }
}

libc_enum! {
    /// When used with [`lio_listio`](fn.lio_listio.html), determines whether a
    /// given `aiocb` should be used for a read operation, a write operation, or
    /// ignored.  Has no effect for any other aio functions.
    #[repr(i32)]
    pub enum LioOpcode {
        LIO_NOP,
        LIO_WRITE,
        LIO_READ,
    }
}

libc_enum! {
    /// Mode for [`lio_listio`](fn.lio_listio.html)
    #[repr(i32)]
    pub enum LioMode {
        /// Requests that [`lio_listio`](fn.lio_listio.html) block until all
        /// requested operations have been completed
        LIO_WAIT,
        /// Requests that [`lio_listio`](fn.lio_listio.html) return immediately
        LIO_NOWAIT,
    }
}

/// Return values for [`AioCb::cancel`](struct.AioCb.html#method.cancel) and
/// [`aio_cancel_all`](fn.aio_cancel_all.html)
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AioCancelStat {
    /// All outstanding requests were canceled
    AioCanceled = libc::AIO_CANCELED,
    /// Some requests were not canceled.  Their status should be checked with
    /// `AioCb::error`
    AioNotCanceled = libc::AIO_NOTCANCELED,
    /// All of the requests have already finished
    AioAllDone = libc::AIO_ALLDONE,
}

/// Owns (uniquely or shared) a memory buffer to keep it from `Drop`ing while
/// the kernel has a pointer to it.
#[derive(Clone, Debug)]
pub enum Buffer<'a> {
    /// No buffer to own.
    ///
    /// Used for operations like `aio_fsync` that have no data, or for unsafe
    /// operations that work with raw pointers.
    None,
    /// Immutable shared ownership `Bytes` object
    // Must use out-of-line allocation so the address of the data will be
    // stable.  `Bytes` and `BytesMut` sometimes dynamically allocate a buffer,
    // and sometimes inline the data within the struct itself.
    Bytes(Bytes),
    /// Mutable uniquely owned `BytesMut` object
    BytesMut(BytesMut),
    /// Keeps a reference to a slice
    Phantom(PhantomData<&'a mut [u8]>)
}

impl<'a> Buffer<'a> {
    /// Return the inner `Bytes`, if any
    pub fn bytes(&self) -> Option<&Bytes> {
        match *self {
            Buffer::Bytes(ref x) => Some(x),
            _ => None
        }
    }

    /// Return the inner `BytesMut`, if any
    pub fn bytes_mut(&self) -> Option<&BytesMut> {
        match *self {
            Buffer::BytesMut(ref x) => Some(x),
            _ => None
        }
    }

    /// Is this `Buffer` `None`?
    pub fn is_none(&self) -> bool {
        match *self {
            Buffer::None => true,
            _ => false,
        }
    }
}

/// AIO Control Block.
///
/// The basic structure used by all aio functions.  Each `AioCb` represents one
/// I/O request.
pub struct AioCb<'a> {
    aiocb: libc::aiocb,
    /// Tracks whether the buffer pointed to by `libc::aiocb.aio_buf` is mutable
    mutable: bool,
    /// Could this `AioCb` potentially have any in-kernel state?
    in_progress: bool,
    /// Optionally keeps a reference to the data.
    ///
    /// Used to keep buffers from `Drop`'ing, and may be returned once the
    /// `AioCb` is completed by `into_buffer`.
    buffer: Buffer<'a>
}

impl<'a> AioCb<'a> {
    /// Remove the inner `Buffer` and return it
    ///
    /// It is an error to call this method while the `AioCb` is still in
    /// progress.
    pub fn buffer(&mut self) -> Buffer<'a> {
        assert!(!self.in_progress);
        let mut x = Buffer::None;
        mem::swap(&mut self.buffer, &mut x);
        x
    }

    /// Returns the underlying file descriptor associated with the `AioCb`
    pub fn fd(&self) -> RawFd {
        self.aiocb.aio_fildes
    }

    /// Constructs a new `AioCb` with no associated buffer.
    ///
    /// The resulting `AioCb` structure is suitable for use with `AioCb::fsync`.
    ///
    /// # Parameters
    ///
    /// * `fd`:           File descriptor.  Required for all aio functions.
    /// * `prio`:         If POSIX Prioritized IO is supported, then the
    ///                   operation will be prioritized at the process's
    ///                   priority level minus `prio`.
    /// * `sigev_notify`: Determines how you will be notified of event
    ///                    completion.
    ///
    /// # Examples
    ///
    /// Create an `AioCb` from a raw file descriptor and use it for an
    /// [`fsync`](#method.from_bytes_mut) operation.
    ///
    /// ```
    /// # extern crate tempfile;
    /// # extern crate nix;
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify::SigevNone;
    /// # use std::{thread, time};
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// # fn main() {
    /// let f = tempfile().unwrap();
    /// let mut aiocb = AioCb::from_fd( f.as_raw_fd(), 0, SigevNone);
    /// aiocb.fsync(AioFsyncMode::O_SYNC).expect("aio_fsync failed early");
    /// while (aiocb.error() == Err(Error::from(Errno::EINPROGRESS))) {
    ///     thread::sleep(time::Duration::from_millis(10));
    /// }
    /// aiocb.aio_return().expect("aio_fsync failed late");
    /// # }
    /// ```
    pub fn from_fd(fd: RawFd, prio: libc::c_int,
                    sigev_notify: SigevNotify) -> AioCb<'a> {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = 0;
        a.aio_nbytes = 0;
        a.aio_buf = null_mut();

        AioCb {
            aiocb: a,
            mutable: false,
            in_progress: false,
            buffer: Buffer::None
        }
    }

    /// Constructs a new `AioCb` from a mutable slice.
    ///
    /// The resulting `AioCb` will be suitable for both read and write
    /// operations, but only if the borrow checker can guarantee that the slice
    /// will outlive the `AioCb`.  That will usually be the case if the `AioCb`
    /// is stack-allocated.  If the borrow checker gives you trouble, try using
    /// [`from_bytes_mut`](#method.from_bytes_mut) instead.
    ///
    /// # Parameters
    ///
    /// * `fd`:           File descriptor.  Required for all aio functions.
    /// * `offs`:         File offset
    /// * `buf`:          A memory buffer
    /// * `prio`:         If POSIX Prioritized IO is supported, then the
    ///                   operation will be prioritized at the process's
    ///                   priority level minus `prio`
    /// * `sigev_notify`: Determines how you will be notified of event
    ///                   completion.
    /// * `opcode`:       This field is only used for `lio_listio`.  It
    ///                   determines which operation to use for this individual
    ///                   aiocb
    ///
    /// # Examples
    ///
    /// Create an `AioCb` from a mutable slice and read into it.
    ///
    /// ```
    /// # extern crate tempfile;
    /// # extern crate nix;
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::{thread, time};
    /// # use std::io::Write;
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// # fn main() {
    /// const INITIAL: &[u8] = b"abcdef123456";
    /// const LEN: usize = 4;
    /// let mut rbuf = vec![0; LEN];
    /// let mut f = tempfile().unwrap();
    /// f.write_all(INITIAL).unwrap();
    /// {
    ///     let mut aiocb = AioCb::from_mut_slice( f.as_raw_fd(),
    ///         2,   //offset
    ///         &mut rbuf,
    ///         0,   //priority
    ///         SigevNotify::SigevNone,
    ///         LioOpcode::LIO_NOP);
    ///     aiocb.read().unwrap();
    ///     while (aiocb.error() == Err(Error::from(Errno::EINPROGRESS))) {
    ///         thread::sleep(time::Duration::from_millis(10));
    ///     }
    ///     assert_eq!(aiocb.aio_return().unwrap() as usize, LEN);
    /// }
    /// assert_eq!(rbuf, b"cdef");
    /// # }
    /// ```
    pub fn from_mut_slice(fd: RawFd, offs: off_t, buf: &'a mut [u8],
                          prio: libc::c_int, sigev_notify: SigevNotify,
                          opcode: LioOpcode) -> AioCb<'a> {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = offs;
        a.aio_nbytes = buf.len() as size_t;
        a.aio_buf = buf.as_ptr() as *mut c_void;
        a.aio_lio_opcode = opcode as libc::c_int;

        AioCb {
            aiocb: a,
            mutable: true,
            in_progress: false,
            buffer: Buffer::Phantom(PhantomData),
        }
    }

    /// Constructs a new `AioCb` from a `Bytes` object.
    ///
    /// Unlike `from_slice`, this method returns a structure suitable for
    /// placement on the heap.  It may be used for write operations, but not
    /// read operations.
    ///
    /// # Parameters
    ///
    /// * `fd`:           File descriptor.  Required for all aio functions.
    /// * `offs`:         File offset
    /// * `buf`:          A shared memory buffer
    /// * `prio`:         If POSIX Prioritized IO is supported, then the
    ///                   operation will be prioritized at the process's
    ///                   priority level minus `prio`
    /// * `sigev_notify`: Determines how you will be notified of event
    ///                   completion.
    /// * `opcode`:       This field is only used for `lio_listio`.  It
    ///                   determines which operation to use for this individual
    ///                   aiocb
    ///
    /// # Examples
    ///
    /// Create an `AioCb` from a `Bytes` object and use it for writing.
    ///
    /// ```
    /// # extern crate bytes;
    /// # extern crate tempfile;
    /// # extern crate nix;
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use bytes::Bytes;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::{thread, time};
    /// # use std::io::Write;
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// # fn main() {
    /// let wbuf = Bytes::from(&b"CDEF"[..]);
    /// let mut f = tempfile().unwrap();
    /// let mut aiocb = AioCb::from_bytes( f.as_raw_fd(),
    ///     2,   //offset
    ///     wbuf.clone(),
    ///     0,   //priority
    ///     SigevNotify::SigevNone,
    ///     LioOpcode::LIO_NOP);
    /// aiocb.write().unwrap();
    /// while (aiocb.error() == Err(Error::from(Errno::EINPROGRESS))) {
    ///     thread::sleep(time::Duration::from_millis(10));
    /// }
    /// assert_eq!(aiocb.aio_return().unwrap() as usize, wbuf.len());
    /// # }
    /// ```
    pub fn from_bytes(fd: RawFd, offs: off_t, buf: Bytes,
                      prio: libc::c_int, sigev_notify: SigevNotify,
                      opcode: LioOpcode) -> AioCb<'a> {
        // Small BytesMuts are stored inline.  Inline storage is a no-no,
        // because we store a pointer to the buffer in the AioCb before
        // returning the Buffer by move.  If the buffer is too small, reallocate
        // it to force out-of-line storage
        // TODO: Add an is_inline() method to BytesMut, and a way to explicitly
        // force out-of-line allocation.
        let buf2 = if buf.len() < 64 {
            // Reallocate to force out-of-line allocation
            let mut ool = Bytes::with_capacity(64);
            ool.extend_from_slice(buf.deref());
            ool
        } else {
            buf
        };
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = offs;
        a.aio_nbytes = buf2.len() as size_t;
        a.aio_buf = buf2.as_ptr() as *mut c_void;
        a.aio_lio_opcode = opcode as libc::c_int;

        AioCb {
            aiocb: a,
            mutable: false,
            in_progress: false,
            buffer: Buffer::Bytes(buf2),
        }
    }

    /// Constructs a new `AioCb` from a `BytesMut` object.
    ///
    /// Unlike `from_mut_slice`, this method returns a structure suitable for
    /// placement on the heap.  It may be used for both reads and writes.
    ///
    /// # Parameters
    ///
    /// * `fd`:           File descriptor.  Required for all aio functions.
    /// * `offs`:         File offset
    /// * `buf`:          An owned memory buffer
    /// * `prio`:         If POSIX Prioritized IO is supported, then the
    ///                   operation will be prioritized at the process's
    ///                   priority level minus `prio`
    /// * `sigev_notify`: Determines how you will be notified of event
    ///                   completion.
    /// * `opcode`:       This field is only used for `lio_listio`.  It
    ///                   determines which operation to use for this individual
    ///                   aiocb
    ///
    /// # Examples
    ///
    /// Create an `AioCb` from a `BytesMut` and use it for reading.  In this
    /// example the `AioCb` is stack-allocated, so we could've used
    /// `from_mut_slice` instead.
    ///
    /// ```
    /// # extern crate bytes;
    /// # extern crate tempfile;
    /// # extern crate nix;
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use bytes::BytesMut;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::{thread, time};
    /// # use std::io::Write;
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// # fn main() {
    /// const INITIAL: &[u8] = b"abcdef123456";
    /// const LEN: usize = 4;
    /// let rbuf = BytesMut::from(vec![0; LEN]);
    /// let mut f = tempfile().unwrap();
    /// f.write_all(INITIAL).unwrap();
    /// let mut aiocb = AioCb::from_bytes_mut( f.as_raw_fd(),
    ///     2,   //offset
    ///     rbuf,
    ///     0,   //priority
    ///     SigevNotify::SigevNone,
    ///     LioOpcode::LIO_NOP);
    /// aiocb.read().unwrap();
    /// while (aiocb.error() == Err(Error::from(Errno::EINPROGRESS))) {
    ///     thread::sleep(time::Duration::from_millis(10));
    /// }
    /// assert_eq!(aiocb.aio_return().unwrap() as usize, LEN);
    /// let buffer = aiocb.into_buffer();
    /// const EXPECT: &[u8] = b"cdef";
    /// assert_eq!(buffer.bytes_mut().unwrap(), EXPECT);
    /// # }
    /// ```
    pub fn from_bytes_mut(fd: RawFd, offs: off_t, buf: BytesMut,
                          prio: libc::c_int, sigev_notify: SigevNotify,
                          opcode: LioOpcode) -> AioCb<'a> {
        let mut buf2 = if buf.len() < 64 {
            // Reallocate to force out-of-line allocation
            let mut ool = BytesMut::with_capacity(64);
            ool.extend_from_slice(buf.deref());
            ool
        } else {
            buf
        };
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = offs;
        a.aio_nbytes = buf2.len() as size_t;
        a.aio_buf = buf2.as_mut_ptr() as *mut c_void;
        a.aio_lio_opcode = opcode as libc::c_int;

        AioCb {
            aiocb: a,
            mutable: true,
            in_progress: false,
            buffer: Buffer::BytesMut(buf2),
        }
    }

    /// Constructs a new `AioCb` from a mutable raw pointer
    ///
    /// Unlike `from_mut_slice`, this method returns a structure suitable for
    /// placement on the heap.  It may be used for both reads and writes.  Due
    /// to its unsafety, this method is not recommended.  It is most useful when
    /// heap allocation is required but for some reason the data cannot be
    /// converted to a `BytesMut`.
    ///
    /// # Parameters
    ///
    /// * `fd`:           File descriptor.  Required for all aio functions.
    /// * `offs`:         File offset
    /// * `buf`:          Pointer to the memory buffer
    /// * `len`:          Length of the buffer pointed to by `buf`
    /// * `prio`:         If POSIX Prioritized IO is supported, then the
    ///                   operation will be prioritized at the process's
    ///                   priority level minus `prio`
    /// * `sigev_notify`: Determines how you will be notified of event
    ///                   completion.
    /// * `opcode`:       This field is only used for `lio_listio`.  It
    ///                   determines which operation to use for this individual
    ///                   aiocb
    ///
    /// # Safety
    ///
    /// The caller must ensure that the storage pointed to by `buf` outlives the
    /// `AioCb`.  The lifetime checker can't help here.
    pub unsafe fn from_mut_ptr(fd: RawFd, offs: off_t,
                           buf: *mut c_void, len: usize,
                           prio: libc::c_int, sigev_notify: SigevNotify,
                           opcode: LioOpcode) -> AioCb<'a> {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = offs;
        a.aio_nbytes = len;
        a.aio_buf = buf;
        a.aio_lio_opcode = opcode as libc::c_int;

        AioCb {
            aiocb: a,
            mutable: true,
            in_progress: false,
            buffer: Buffer::None
        }
    }

    /// Constructs a new `AioCb` from a raw pointer.
    ///
    /// Unlike `from_slice`, this method returns a structure suitable for
    /// placement on the heap.  Due to its unsafety, this method is not
    /// recommended.  It is most useful when heap allocation is required but for
    /// some reason the data cannot be converted to a `Bytes`.
    ///
    /// # Parameters
    ///
    /// * `fd`:           File descriptor.  Required for all aio functions.
    /// * `offs`:         File offset
    /// * `buf`:          Pointer to the memory buffer
    /// * `len`:          Length of the buffer pointed to by `buf`
    /// * `prio`:         If POSIX Prioritized IO is supported, then the
    ///                   operation will be prioritized at the process's
    ///                   priority level minus `prio`
    /// * `sigev_notify`: Determines how you will be notified of event
    ///                   completion.
    /// * `opcode`:       This field is only used for `lio_listio`.  It
    ///                   determines which operation to use for this individual
    ///                   aiocb
    ///
    /// # Safety
    ///
    /// The caller must ensure that the storage pointed to by `buf` outlives the
    /// `AioCb`.  The lifetime checker can't help here.
    pub unsafe fn from_ptr(fd: RawFd, offs: off_t,
                           buf: *const c_void, len: usize,
                           prio: libc::c_int, sigev_notify: SigevNotify,
                           opcode: LioOpcode) -> AioCb<'a> {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = offs;
        a.aio_nbytes = len;
        // casting a const ptr to a mutable ptr here is ok, because we set the
        // AioCb's mutable field to false
        a.aio_buf = buf as *mut c_void;
        a.aio_lio_opcode = opcode as libc::c_int;

        AioCb {
            aiocb: a,
            mutable: false,
            in_progress: false,
            buffer: Buffer::None
        }
    }

    /// Like `from_mut_slice`, but works on constant slices rather than
    /// mutable slices.
    ///
    /// An `AioCb` created this way cannot be used with `read`, and its
    /// `LioOpcode` cannot be set to `LIO_READ`.  This method is useful when
    /// writing a const buffer with `AioCb::write`, since `from_mut_slice` can't
    /// work with const buffers.
    ///
    /// # Examples
    ///
    /// Construct an `AioCb` from a slice and use it for writing.
    ///
    /// ```
    /// # extern crate tempfile;
    /// # extern crate nix;
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::{thread, time};
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// # fn main() {
    /// const WBUF: &[u8] = b"abcdef123456";
    /// let mut f = tempfile().unwrap();
    /// let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
    ///     2,   //offset
    ///     WBUF,
    ///     0,   //priority
    ///     SigevNotify::SigevNone,
    ///     LioOpcode::LIO_NOP);
    /// aiocb.write().unwrap();
    /// while (aiocb.error() == Err(Error::from(Errno::EINPROGRESS))) {
    ///     thread::sleep(time::Duration::from_millis(10));
    /// }
    /// assert_eq!(aiocb.aio_return().unwrap() as usize, WBUF.len());
    /// # }
    /// ```
    // Note: another solution to the problem of writing const buffers would be
    // to genericize AioCb for both &mut [u8] and &[u8] buffers.  AioCb::read
    // could take the former and AioCb::write could take the latter.  However,
    // then lio_listio wouldn't work, because that function needs a slice of
    // AioCb, and they must all be of the same type.
    pub fn from_slice(fd: RawFd, offs: off_t, buf: &'a [u8],
                      prio: libc::c_int, sigev_notify: SigevNotify,
                      opcode: LioOpcode) -> AioCb {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = offs;
        a.aio_nbytes = buf.len() as size_t;
        // casting an immutable buffer to a mutable pointer looks unsafe,
        // but technically its only unsafe to dereference it, not to create
        // it.
        a.aio_buf = buf.as_ptr() as *mut c_void;
        assert!(opcode != LioOpcode::LIO_READ, "Can't read into an immutable buffer");
        a.aio_lio_opcode = opcode as libc::c_int;

        AioCb {
            aiocb: a,
            mutable: false,
            in_progress: false,
            buffer: Buffer::None,
        }
    }

    /// Consumes the `aiocb` and returns its inner `Buffer`, if any.
    ///
    /// This method is especially useful when reading into a `BytesMut`, because
    /// that type does not support shared ownership.
    pub fn into_buffer(mut self) -> Buffer<'static> {
        let buf = self.buffer();
        match buf {
            Buffer::BytesMut(x) => Buffer::BytesMut(x),
            Buffer::Bytes(x) => Buffer::Bytes(x),
            _ => Buffer::None
        }
    }

    fn common_init(fd: RawFd, prio: libc::c_int,
                   sigev_notify: SigevNotify) -> libc::aiocb {
        // Use mem::zeroed instead of explicitly zeroing each field, because the
        // number and name of reserved fields is OS-dependent.  On some OSes,
        // some reserved fields are used the kernel for state, and must be
        // explicitly zeroed when allocated.
        let mut a = unsafe { mem::zeroed::<libc::aiocb>()};
        a.aio_fildes = fd;
        a.aio_reqprio = prio;
        a.aio_sigevent = SigEvent::new(sigev_notify).sigevent();
        a
    }

    /// Update the notification settings for an existing `aiocb`
    pub fn set_sigev_notify(&mut self, sigev_notify: SigevNotify) {
        self.aiocb.aio_sigevent = SigEvent::new(sigev_notify).sigevent();
    }

    /// Cancels an outstanding AIO request.
    ///
    /// The operating system is not required to implement cancellation for all
    /// file and device types.  Even if it does, there is no guarantee that the
    /// operation has not already completed.  So the caller must check the
    /// result and handle operations that were not canceled or that have already
    /// completed.
    ///
    /// # Examples
    ///
    /// Cancel an outstanding aio operation.  Note that we must still call
    /// `aio_return` to free resources, even though we don't care about the
    /// result.
    ///
    /// ```
    /// # extern crate bytes;
    /// # extern crate tempfile;
    /// # extern crate nix;
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use bytes::Bytes;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::{thread, time};
    /// # use std::io::Write;
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// # fn main() {
    /// let wbuf = Bytes::from(&b"CDEF"[..]);
    /// let mut f = tempfile().unwrap();
    /// let mut aiocb = AioCb::from_bytes( f.as_raw_fd(),
    ///     2,   //offset
    ///     wbuf.clone(),
    ///     0,   //priority
    ///     SigevNotify::SigevNone,
    ///     LioOpcode::LIO_NOP);
    /// aiocb.write().unwrap();
    /// let cs = aiocb.cancel().unwrap();
    /// if cs == AioCancelStat::AioNotCanceled {
    ///     while (aiocb.error() == Err(Error::from(Errno::EINPROGRESS))) {
    ///         thread::sleep(time::Duration::from_millis(10));
    ///     }
    /// }
    /// // Must call `aio_return`, but ignore the result
    /// let _ = aiocb.aio_return();
    /// # }
    /// ```
    ///
    /// # References
    ///
    /// [aio_cancel](http://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_cancel.html)
    pub fn cancel(&mut self) -> Result<AioCancelStat> {
        match unsafe { libc::aio_cancel(self.aiocb.aio_fildes, &mut self.aiocb) } {
            libc::AIO_CANCELED => Ok(AioCancelStat::AioCanceled),
            libc::AIO_NOTCANCELED => Ok(AioCancelStat::AioNotCanceled),
            libc::AIO_ALLDONE => Ok(AioCancelStat::AioAllDone),
            -1 => Err(Error::last()),
            _ => panic!("unknown aio_cancel return value")
        }
    }

    /// Retrieve error status of an asynchronous operation.
    ///
    /// If the request has not yet completed, returns `EINPROGRESS`.  Otherwise,
    /// returns `Ok` or any other error.
    ///
    /// # Examples
    ///
    /// Issue an aio operation and use `error` to poll for completion.  Polling
    /// is an alternative to `aio_suspend`, used by most of the other examples.
    ///
    /// ```
    /// # extern crate tempfile;
    /// # extern crate nix;
    /// # use nix::errno::Errno;
    /// # use nix::Error;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::{thread, time};
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// # fn main() {
    /// const WBUF: &[u8] = b"abcdef123456";
    /// let mut f = tempfile().unwrap();
    /// let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
    ///     2,   //offset
    ///     WBUF,
    ///     0,   //priority
    ///     SigevNotify::SigevNone,
    ///     LioOpcode::LIO_NOP);
    /// aiocb.write().unwrap();
    /// while (aiocb.error() == Err(Error::from(Errno::EINPROGRESS))) {
    ///     thread::sleep(time::Duration::from_millis(10));
    /// }
    /// assert_eq!(aiocb.aio_return().unwrap() as usize, WBUF.len());
    /// # }
    /// ```
    ///
    /// # References
    ///
    /// [aio_error](http://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_error.html)
    pub fn error(&mut self) -> Result<()> {
        match unsafe { libc::aio_error(&mut self.aiocb as *mut libc::aiocb) } {
            0 => Ok(()),
            num if num > 0 => Err(Error::from_errno(Errno::from_i32(num))),
            -1 => Err(Error::last()),
            num => panic!("unknown aio_error return value {:?}", num)
        }
    }

    /// An asynchronous version of `fsync(2)`.
    ///
    /// # References
    ///
    /// [aio_fsync](http://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_fsync.html)
    pub fn fsync(&mut self, mode: AioFsyncMode) -> Result<()> {
        let p: *mut libc::aiocb = &mut self.aiocb;
        Errno::result(unsafe {
                libc::aio_fsync(mode as libc::c_int, p)
        }).map(|_| {
            self.in_progress = true;
        })
    }

    /// Returns the `aiocb`'s `LioOpcode` field
    ///
    /// If the value cannot be represented as an `LioOpcode`, returns `None`
    /// instead.
    pub fn lio_opcode(&self) -> Option<LioOpcode> {
        match self.aiocb.aio_lio_opcode {
            libc::LIO_READ => Some(LioOpcode::LIO_READ),
            libc::LIO_WRITE => Some(LioOpcode::LIO_WRITE),
            libc::LIO_NOP => Some(LioOpcode::LIO_NOP),
            _ => None
        }
    }

    /// Returns the requested length of the aio operation in bytes
    ///
    /// This method returns the *requested* length of the operation.  To get the
    /// number of bytes actually read or written by a completed operation, use
    /// `aio_return` instead.
    pub fn nbytes(&self) -> usize {
        self.aiocb.aio_nbytes
    }

    /// Returns the file offset stored in the `AioCb`
    pub fn offset(&self) -> off_t {
        self.aiocb.aio_offset
    }

    /// Returns the priority of the `AioCb`
    pub fn priority(&self) -> libc::c_int {
        self.aiocb.aio_reqprio
    }

    /// Asynchronously reads from a file descriptor into a buffer
    ///
    /// # References
    ///
    /// [aio_read](http://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_read.html)
    pub fn read(&mut self) -> Result<()> {
        assert!(self.mutable, "Can't read into an immutable buffer");
        let p: *mut libc::aiocb = &mut self.aiocb;
        Errno::result(unsafe {
            libc::aio_read(p)
        }).map(|_| {
            self.in_progress = true;
        })
    }

    /// Returns the `SigEvent` stored in the `AioCb`
    pub fn sigevent(&self) -> SigEvent {
        SigEvent::from(&self.aiocb.aio_sigevent)
    }

    /// Retrieve return status of an asynchronous operation.
    ///
    /// Should only be called once for each `AioCb`, after `AioCb::error`
    /// indicates that it has completed.  The result is the same as for the
    /// synchronous `read(2)`, `write(2)`, of `fsync(2)` functions.
    ///
    /// # References
    ///
    /// [aio_return](http://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_return.html)
    // Note: this should be just `return`, but that's a reserved word
    pub fn aio_return(&mut self) -> Result<isize> {
        let p: *mut libc::aiocb = &mut self.aiocb;
        self.in_progress = false;
        Errno::result(unsafe { libc::aio_return(p) })
    }

    /// Asynchronously writes from a buffer to a file descriptor
    ///
    /// # References
    ///
    /// [aio_write](http://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_write.html)
    pub fn write(&mut self) -> Result<()> {
        let p: *mut libc::aiocb = &mut self.aiocb;
        Errno::result(unsafe {
            libc::aio_write(p)
        }).map(|_| {
            self.in_progress = true;
        })
    }

}

/// Cancels outstanding AIO requests for a given file descriptor.
///
/// # Examples
///
/// Issue an aio operation, then cancel all outstanding operations on that file
/// descriptor.
///
/// ```
/// # extern crate bytes;
/// # extern crate tempfile;
/// # extern crate nix;
/// # use nix::errno::Errno;
/// # use nix::Error;
/// # use bytes::Bytes;
/// # use nix::sys::aio::*;
/// # use nix::sys::signal::SigevNotify;
/// # use std::{thread, time};
/// # use std::io::Write;
/// # use std::os::unix::io::AsRawFd;
/// # use tempfile::tempfile;
/// # fn main() {
/// let wbuf = Bytes::from(&b"CDEF"[..]);
/// let mut f = tempfile().unwrap();
/// let mut aiocb = AioCb::from_bytes( f.as_raw_fd(),
///     2,   //offset
///     wbuf.clone(),
///     0,   //priority
///     SigevNotify::SigevNone,
///     LioOpcode::LIO_NOP);
/// aiocb.write().unwrap();
/// let cs = aio_cancel_all(f.as_raw_fd()).unwrap();
/// if cs == AioCancelStat::AioNotCanceled {
///     while (aiocb.error() == Err(Error::from(Errno::EINPROGRESS))) {
///         thread::sleep(time::Duration::from_millis(10));
///     }
/// }
/// // Must call `aio_return`, but ignore the result
/// let _ = aiocb.aio_return();
/// # }
/// ```
///
/// # References
///
/// [`aio_cancel`](http://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_cancel.html)
pub fn aio_cancel_all(fd: RawFd) -> Result<AioCancelStat> {
    match unsafe { libc::aio_cancel(fd, null_mut()) } {
        libc::AIO_CANCELED => Ok(AioCancelStat::AioCanceled),
        libc::AIO_NOTCANCELED => Ok(AioCancelStat::AioNotCanceled),
        libc::AIO_ALLDONE => Ok(AioCancelStat::AioAllDone),
        -1 => Err(Error::last()),
        _ => panic!("unknown aio_cancel return value")
    }
}

/// Suspends the calling process until at least one of the specified `AioCb`s
/// has completed, a signal is delivered, or the timeout has passed.
///
/// If `timeout` is `None`, `aio_suspend` will block indefinitely.
///
/// # Examples
///
/// Use `aio_suspend` to block until an aio operation completes.
///
// Disable doctest due to a known bug in FreeBSD's 32-bit emulation.  The fix
// will be included in release 11.2.
// FIXME reenable the doc test when the CI machine gets upgraded to that release.
// https://svnweb.freebsd.org/base?view=revision&revision=325018
/// ```no_run
/// # extern crate tempfile;
/// # extern crate nix;
/// # use nix::sys::aio::*;
/// # use nix::sys::signal::SigevNotify;
/// # use std::os::unix::io::AsRawFd;
/// # use tempfile::tempfile;
/// # fn main() {
/// const WBUF: &[u8] = b"abcdef123456";
/// let mut f = tempfile().unwrap();
/// let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
///     2,   //offset
///     WBUF,
///     0,   //priority
///     SigevNotify::SigevNone,
///     LioOpcode::LIO_NOP);
/// aiocb.write().unwrap();
/// aio_suspend(&[&aiocb], None).expect("aio_suspend failed");
/// assert_eq!(aiocb.aio_return().unwrap() as usize, WBUF.len());
/// # }
/// ```
/// # References
///
/// [`aio_suspend`](http://pubs.opengroup.org/onlinepubs/9699919799/functions/aio_suspend.html)
pub fn aio_suspend(list: &[&AioCb], timeout: Option<TimeSpec>) -> Result<()> {
    let plist = list as *const [&AioCb] as *const [*const libc::aiocb];
    let p = plist as *const *const libc::aiocb;
    let timep = match timeout {
        None    => null::<libc::timespec>(),
        Some(x) => x.as_ref() as *const libc::timespec
    };
    Errno::result(unsafe {
        libc::aio_suspend(p, list.len() as i32, timep)
    }).map(drop)
}

impl<'a> Debug for AioCb<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("AioCb")
            .field("aio_fildes", &self.aiocb.aio_fildes)
            .field("aio_offset", &self.aiocb.aio_offset)
            .field("aio_buf", &self.aiocb.aio_buf)
            .field("aio_nbytes", &self.aiocb.aio_nbytes)
            .field("aio_lio_opcode", &self.aiocb.aio_lio_opcode)
            .field("aio_reqprio", &self.aiocb.aio_reqprio)
            .field("aio_sigevent", &SigEvent::from(&self.aiocb.aio_sigevent))
            .field("mutable", &self.mutable)
            .field("in_progress", &self.in_progress)
            .finish()
    }
}

impl<'a> Drop for AioCb<'a> {
    /// If the `AioCb` has no remaining state in the kernel, just drop it.
    /// Otherwise, dropping constitutes a resource leak, which is an error
    fn drop(&mut self) {
        assert!(thread::panicking() || !self.in_progress,
                "Dropped an in-progress AioCb");
    }
}

/// LIO Control Block.
///
/// The basic structure used to issue multiple AIO operations simultaneously.
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub struct LioCb<'a> {
    /// A collection of [`AioCb`]s.  All of these will be issued simultaneously
    /// by the [`listio`] method.
    ///
    /// [`AioCb`]: struct.AioCb.html
    /// [`listio`]: #method.listio
    pub aiocbs: Vec<AioCb<'a>>,

    /// The actual list passed to `libc::lio_listio`.
    ///
    /// It must live for as long as any of the operations are still being
    /// processesed, because the aio subsystem uses its address as a unique
    /// identifier.
    list: Vec<*mut libc::aiocb>
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
impl<'a> LioCb<'a> {
    /// Initialize an empty `LioCb`
    pub fn with_capacity(capacity: usize) -> LioCb<'a> {
        LioCb {
            aiocbs: Vec::with_capacity(capacity),
            list: Vec::with_capacity(capacity)
        }
    }

    /// Submits multiple asynchronous I/O requests with a single system call.
    ///
    /// They are not guaranteed to complete atomically, and the order in which
    /// the requests are carried out is not specified.  Reads, writes, and
    /// fsyncs may be freely mixed.
    ///
    /// This function is useful for reducing the context-switch overhead of
    /// submitting many AIO operations.  It can also be used with
    /// `LioMode::LIO_WAIT` to block on the result of several independent
    /// operations.  Used that way, it is often useful in programs that
    /// otherwise make little use of AIO.
    ///
    /// # Examples
    ///
    /// Use `listio` to submit an aio operation and wait for its completion.  In
    /// this case, there is no need to use `aio_suspend` to wait or
    /// `AioCb#error` to poll.
    ///
    /// ```
    /// # extern crate tempfile;
    /// # extern crate nix;
    /// # use nix::sys::aio::*;
    /// # use nix::sys::signal::SigevNotify;
    /// # use std::os::unix::io::AsRawFd;
    /// # use tempfile::tempfile;
    /// # fn main() {
    /// const WBUF: &[u8] = b"abcdef123456";
    /// let mut f = tempfile().unwrap();
    /// let mut liocb = LioCb::with_capacity(1);
    /// liocb.aiocbs.push(AioCb::from_slice( f.as_raw_fd(),
    ///     2,   //offset
    ///     WBUF,
    ///     0,   //priority
    ///     SigevNotify::SigevNone,
    ///     LioOpcode::LIO_WRITE));
    /// liocb.listio(LioMode::LIO_WAIT,
    ///              SigevNotify::SigevNone).unwrap();
    /// assert_eq!(liocb.aiocbs[0].aio_return().unwrap() as usize, WBUF.len());
    /// # }
    /// ```
    ///
    /// # References
    ///
    /// [`lio_listio`](http://pubs.opengroup.org/onlinepubs/9699919799/functions/lio_listio.html)
    pub fn listio(&mut self, mode: LioMode,
                  sigev_notify: SigevNotify) -> Result<()> {
        let sigev = SigEvent::new(sigev_notify);
        let sigevp = &mut sigev.sigevent() as *mut libc::sigevent;
        self.list.clear();
        for a in self.aiocbs.iter_mut() {
            self.list.push(a as *mut AioCb<'a>
                             as *mut libc::aiocb);
        }
        let p = self.list.as_ptr();
        Errno::result(unsafe {
            libc::lio_listio(mode as i32, p, self.list.len() as i32, sigevp)
        }).map(|_| ())
    }
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
impl<'a> Debug for LioCb<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("LioCb")
            .field("aiocbs", &self.aiocbs)
            .finish()
    }
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
impl<'a> From<Vec<AioCb<'a>>> for LioCb<'a> {
    fn from(src: Vec<AioCb<'a>>) -> LioCb<'a> {
        LioCb {
            list: Vec::with_capacity(src.capacity()),
            aiocbs: src,
        }
    }
}
