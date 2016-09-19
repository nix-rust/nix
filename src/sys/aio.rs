use {Error, Errno, Result};
use std::os::unix::io::RawFd;
use libc::{c_void, off_t, size_t};
use libc;
use std::mem;
use std::ptr::{null, null_mut};
use sys::signal::*;
use sys::time::TimeSpec;

/// Mode for `aio_fsync`.  Controls whether only data or both data and metadata
/// are synced.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AioFsyncMode {
    /// do it like `fsync`
    O_SYNC = libc::O_SYNC,
    /// on supported operating systems only, do it like `fdatasync`
    #[cfg(any(target_os = "openbsd", target_os = "bitrig",
              target_os = "netbsd", target_os = "macos", target_os = "ios",
              target_os = "linux"))]
    O_DSYNC = libc::O_DSYNC
}

/// When used with `lio_listio`, determines whether a given `aiocb` should be
/// used for a read operation, a write operation, or ignored.  Has no effect for
/// any other aio functions.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LioOpcode {
    LIO_NOP = libc::LIO_NOP,
    LIO_WRITE = libc::LIO_WRITE,
    LIO_READ = libc::LIO_READ
}

/// Mode for `lio_listio`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LioMode {
    /// Requests that `lio_listio` block until all requested operations have
    /// been completed
    LIO_WAIT = libc::LIO_WAIT,
    /// Requests that `lio_listio` return immediately
    LIO_NOWAIT = libc::LIO_NOWAIT,
}

/// Return values for `aio_cancel`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AioCancelStat {
    /// All outstanding requests were canceled
    AioCanceled = libc::AIO_CANCELED,
    /// Some requests were not canceled.  Their status should be checked with
    /// `aio_error`
    AioNotCanceled = libc::AIO_NOTCANCELED,
    /// All of the requests have already finished
    AioAllDone = libc::AIO_ALLDONE,
}

/// The basic structure used by all aio functions.  Each `aiocb` represents one
/// I/O request.
#[repr(C)]
pub struct AioCb {
    aiocb: libc::aiocb
}

impl AioCb {
    /// Constructs a new `AioCb` with no associated buffer.
    ///
    /// The resulting `AioCb` structure is suitable for use with `aio_fsync`.
    /// * `fd`  File descriptor.  Required for all aio functions.
    /// * `prio` If POSIX Prioritized IO is supported, then the operation will
    /// be prioritized at the process's priority level minus `prio`
    /// * `sigev_notify` Determines how you will be notified of event
    /// completion.
    pub fn from_fd(fd: RawFd, prio: ::c_int,
                    sigev_notify: SigevNotify) -> AioCb {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = 0;
        a.aio_nbytes = 0;
        a.aio_buf = null_mut();

        let aiocb = AioCb { aiocb: a};
        aiocb
    }

    /// Constructs a new `AioCb`.
    ///
    /// * `fd`  File descriptor.  Required for all aio functions.
    /// * `offs` File offset
    /// * `buf` A memory buffer
    /// * `prio` If POSIX Prioritized IO is supported, then the operation will
    /// be prioritized at the process's priority level minus `prio`
    /// * `sigev_notify` Determines how you will be notified of event
    /// completion.
    /// * `opcode` This field is only used for `lio_listio`.  It determines
    /// which operation to use for this individual aiocb
    pub fn from_mut_slice(fd: RawFd, offs: off_t, buf: &mut [u8],
                          prio: ::c_int, sigev_notify: SigevNotify,
                          opcode: LioOpcode) -> AioCb {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = offs;
        a.aio_nbytes = buf.len() as size_t;
        a.aio_buf = buf.as_ptr() as *mut c_void;
        a.aio_lio_opcode = opcode as ::c_int;

        let aiocb = AioCb { aiocb: a};
        aiocb
    }

    /// Like `from_mut_slice`, but works on constant slices rather than
    /// mutable slices.
    ///
    /// This is technically unsafe, but in practice it's fine
    /// to use with any aio functions except `aio_read` and `lio_listio` (with
    /// `opcode` set to `LIO_READ`).  This method is useful when writing a const
    /// buffer with `aio_write`, since from_mut_slice can't work with const
    /// buffers.
    // Note: another solution to the problem of writing const buffers would be
    // to genericize AioCb for both &mut [u8] and &[u8] buffers.  aio_read could
    // take the former and aio_write could take the latter.  However, then
    // lio_listio wouldn't work, because that function needs a slice of AioCb,
    // and they must all be the same type.  We're basically stuck with using an
    // unsafe function, since aio (as designed in C) is an unsafe API.
    pub unsafe fn from_slice(fd: RawFd, offs: off_t, buf: &[u8],
                             prio: ::c_int, sigev_notify: SigevNotify,
                             opcode: LioOpcode) -> AioCb {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = offs;
        a.aio_nbytes = buf.len() as size_t;
        a.aio_buf = buf.as_ptr() as *mut c_void;
        a.aio_lio_opcode = opcode as ::c_int;

        let aiocb = AioCb { aiocb: a};
        aiocb
    }

    fn common_init(fd: RawFd, prio: ::c_int,
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
}

/// Cancels outstanding AIO requests.  If `aiocb` is `None`, then all requests
/// for `fd` will be cancelled.  Otherwise, only the given `AioCb` will be
/// cancelled.
pub fn aio_cancel(fd: RawFd, aiocb: Option<&mut AioCb>) -> Result<AioCancelStat> {
    let p: *mut libc::aiocb = match aiocb {
        None => null_mut(),
        Some(x) => &mut x.aiocb
    };
    match unsafe { libc::aio_cancel(fd, p) } {
        libc::AIO_CANCELED => Ok(AioCancelStat::AioCanceled),
        libc::AIO_NOTCANCELED => Ok(AioCancelStat::AioNotCanceled),
        libc::AIO_ALLDONE => Ok(AioCancelStat::AioAllDone),
        -1 => Err(Error::last()),
        _ => panic!("unknown aio_cancel return value")
    }
}

/// Retrieve error status of an asynchronous operation.  If the request has not
/// yet completed, returns `EINPROGRESS`.  Otherwise, returns `Ok` or any other
/// error.
pub fn aio_error(aiocb: &mut AioCb) -> Result<()> {
    let p: *mut libc::aiocb = &mut aiocb.aiocb;
    match unsafe { libc::aio_error(p) } {
        0 => Ok(()),
        num if num > 0 => Err(Error::from_errno(Errno::from_i32(num))),
        -1 => Err(Error::last()),
        num => panic!("unknown aio_error return value {:?}", num)
    }
}

/// An asynchronous version of `fsync`.
pub fn aio_fsync(mode: AioFsyncMode, aiocb: &mut AioCb) -> Result<()> {
    let p: *mut libc::aiocb = &mut aiocb.aiocb;
    Errno::result(unsafe { libc::aio_fsync(mode as ::c_int, p) }).map(drop)
}

/// Asynchously reads from a file descriptor into a buffer
pub fn aio_read(aiocb: &mut AioCb) -> Result<()> {
    let p: *mut libc::aiocb = &mut aiocb.aiocb;
    Errno::result(unsafe { libc::aio_read(p) }).map(drop)
}

/// Retrieve return status of an asynchronous operation.  Should only be called
/// once for each `AioCb`, after `aio_error` indicates that it has completed.
/// The result the same as for `read`, `write`, of `fsync`.
pub fn aio_return(aiocb: &mut AioCb) -> Result<isize> {
    let p: *mut libc::aiocb = &mut aiocb.aiocb;
    Errno::result(unsafe { libc::aio_return(p) })
}

/// Suspends the calling process until at least one of the specified `AioCb`s
/// has completed, a signal is delivered, or the timeout has passed.  If
/// `timeout` is `None`, `aio_suspend` will block indefinitely.
pub fn aio_suspend(list: &[&AioCb], timeout: Option<TimeSpec>) -> Result<()> {
    // We must use transmute because Rust doesn't understand that a pointer to a
    // Struct is the same as a pointer to its first element.
    let plist = unsafe {
        mem::transmute::<&[&AioCb], *const [*const libc::aiocb]>(list)
    };
    let p = plist as *const *const libc::aiocb;
    let timep = match timeout {
        None    => null::<libc::timespec>(),
        Some(x) => x.as_ref() as *const libc::timespec
    };
    Errno::result(unsafe {
        libc::aio_suspend(p, list.len() as i32, timep)
    }).map(drop)
}

/// Asynchronously writes from a buffer to a file descriptor
pub fn aio_write(aiocb: &mut AioCb) -> Result<()> {
    let p: *mut libc::aiocb = &mut aiocb.aiocb;
    Errno::result(unsafe { libc::aio_write(p) }).map(drop)
}

/// Submits multiple asynchronous I/O requests with a single system call.  The
/// order in which the requests are carried out is not specified.
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub fn lio_listio(mode: LioMode, list: &[&mut AioCb],
                  sigev_notify: SigevNotify) -> Result<()> {
    let sigev = SigEvent::new(sigev_notify);
    let sigevp = &mut sigev.sigevent() as *mut libc::sigevent;
    // We must use transmute because Rust doesn't understand that a pointer to a
    // Struct is the same as a pointer to its first element.
    let plist = unsafe {
        mem::transmute::<&[&mut AioCb], *const [*mut libc::aiocb]>(list)
    };
    let p = plist as *const *mut libc::aiocb;
    Errno::result(unsafe {
        libc::lio_listio(mode as i32, p, list.len() as i32, sigevp)
    }).map(drop)
}
