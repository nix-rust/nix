//! Monitoring API for filesystem events.
//!
//! Fanotify is a Linux-only API to monitor filesystems events.
//!
//! Additional capabilities compared to the `inotify` API include the ability to
//! monitor all of the objects in a mounted filesystem, the ability to make
//! access permission decisions, and the possibility to read or modify files
//! before access by other applications.
//!
//! For more documentation, please read
//! [fanotify(7)](https://man7.org/linux/man-pages/man7/fanotify.7.html).

use crate::errno::Errno;
use crate::fcntl::OFlag;
use crate::unistd::{close, read, write};
use crate::{NixPath, Result};
use std::marker::PhantomData;
use std::mem::{size_of, MaybeUninit};
use std::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::ptr;

libc_bitflags! {
    /// Mask for defining which events shall be listened with [`Fanotify::mark()`]
    /// and for querying notifications.
    pub struct MaskFlags: u64 {
        /// File was accessed.
        FAN_ACCESS;
        /// File was modified.
        FAN_MODIFY;
        /// Metadata has changed. Since Linux 5.1.
        FAN_ATTRIB;
        /// Writtable file was closed.
        FAN_CLOSE_WRITE;
        /// Unwrittable file was closed.
        FAN_CLOSE_NOWRITE;
        /// File was opened.
        FAN_OPEN;
        /// File was moved from X. Since Linux 5.1.
        FAN_MOVED_FROM;
        /// File was moved to Y. Since Linux 5.1.
        FAN_MOVED_TO;
        /// Subfile was created. Since Linux 5.1.
        FAN_CREATE;
        /// Subfile was deleted. Since Linux 5.1.
        FAN_DELETE;
        /// Self was deleted. Since Linux 5.1.
        FAN_DELETE_SELF;
        /// Self was moved. Since Linux 5.1.
        FAN_MOVE_SELF;
        /// File was opened for execution. Since Linux 5.0.
        FAN_OPEN_EXEC;

        /// Event queue overflowed.
        FAN_Q_OVERFLOW;
        /// Filesystem error. Since Linux 5.16.
        FAN_FS_ERROR;

        /// Permission to open file was requested.
        FAN_OPEN_PERM;
        /// Permission to access file was requested.
        FAN_ACCESS_PERM;
        /// Permission to open file for execution was requested. Since Linux
        /// 5.0.
        FAN_OPEN_EXEC_PERM;

        /// Interested in child events.
        FAN_EVENT_ON_CHILD;

        /// File was renamed. Since Linux 5.17.
        FAN_RENAME;

        /// Event occurred against dir.
        FAN_ONDIR;

        /// Combination of `FAN_CLOSE_WRITE` and `FAN_CLOSE_NOWRITE`.
        FAN_CLOSE;
        /// Combination of `FAN_MOVED_FROM` and `FAN_MOVED_TO`.
        FAN_MOVE;
    }
}

libc_bitflags! {
    /// Configuration options for [`Fanotify::init()`].
    pub struct InitFlags: libc::c_uint {
        /// Close-on-exec flag set on the file descriptor.
        FAN_CLOEXEC;
        /// Nonblocking flag set on the file descriptor.
        FAN_NONBLOCK;

        /// Receipt of events notifications.
        FAN_CLASS_NOTIF;
        /// Receipt of events for permission decisions, after they contain final
        /// data.
        FAN_CLASS_CONTENT;
        /// Receipt of events for permission decisions, before they contain
        /// final data.
        FAN_CLASS_PRE_CONTENT;

        /// Remove the limit on the number of events in the event queue.
        ///
        /// Prior to Linux kernel 5.13, this limit was hardcoded to 16384. After
        /// 5.13, one can change it via file `/proc/sys/fs/fanotify/max_queued_events`.
        ///
        /// See `fanotify(7)` for details about this limit. Use of this flag
        /// requires the `CAP_SYS_ADMIN` capability.
        FAN_UNLIMITED_QUEUE;
        /// Remove the limit on the number of fanotify marks per user.
        ///
        /// Prior to Linux kernel 5.13, this limit was hardcoded to 8192 (per
        /// group, not per user). After 5.13, one can change it via file
        /// `/proc/sys/fs/fanotify/max_user_marks`.
        ///
        /// See `fanotify(7)` for details about this limit. Use of this flag
        /// requires the `CAP_SYS_ADMIN` capability.
        FAN_UNLIMITED_MARKS;

        /// Make `FanotifyEvent::pid` return pidfd. Since Linux 5.15.
        FAN_REPORT_PIDFD;
        /// Make `FanotifyEvent::pid` return thread id. Since Linux 4.20.
        FAN_REPORT_TID;

        /// Allows the receipt of events which contain additional information
        /// about the underlying filesystem object correlated to an event.
        ///
        /// This will make `FanotifyEvent::fd` return `FAN_NOFD`.
        /// This should be used with `Fanotify::read_events_with_info_records` to
        /// recieve `FanotifyInfoRecord::Fid` info records.
        /// Since Linux 5.1
        FAN_REPORT_FID;

        /// Allows the receipt of events which contain additional information
        /// about the underlying filesystem object correlated to an event.
        ///
        /// This will make `FanotifyEvent::fd` return `FAN_NOFD`.
        /// This should be used with `Fanotify::read_events_with_info_records` to
        /// recieve `FanotifyInfoRecord::Fid` info records.
        ///
        /// An additional event of `FAN_EVENT_INFO_TYPE_DFID` will also be received,
        /// encapsulating information about the target directory (or parent directory of a file)
        /// Since Linux 5.9
        FAN_REPORT_DIR_FID;

        /// Events for fanotify groups initialized with this flag will contain additional
        /// information about the child correlated with directory entry modification events.
        /// This flag must be provided in conjunction with the flags `FAN_REPORT_FID`,
        /// `FAN_REPORT_DIR_FID` and `FAN_REPORT_NAME`.
        /// Since Linux 5.17
        FAN_REPORT_TARGET_FID;

        /// Events for fanotify groups initialized with this flag will contain additional
        /// information about the name of the directory entry correlated to an event.  This
        /// flag must be provided in conjunction with the flag `FAN_REPORT_DIR_FID`.
        /// Since Linux 5.9
        FAN_REPORT_NAME;

        /// This is a synonym for `FAN_REPORT_DIR_FD | FAN_REPORT_NAME`.
        FAN_REPORT_DFID_NAME;

        /// This is a synonym for `FAN_REPORT_DIR_FD | FAN_REPORT_NAME | FAN_REPORT_TARGET_FID`.
        FAN_REPORT_DFID_NAME_TARGET;
    }
}

libc_bitflags! {
    /// File status flags for fanotify events file descriptors.
    pub struct EventFFlags: libc::c_uint {
        /// Read only access.
        O_RDONLY as libc::c_uint;
        /// Write only access.
        O_WRONLY as libc::c_uint;
        /// Read and write access.
        O_RDWR as libc::c_uint;
        /// Support for files exceeded 2 GB.
        O_LARGEFILE as libc::c_uint;
        /// Close-on-exec flag for the file descriptor. Since Linux 3.18.
        O_CLOEXEC as libc::c_uint;
        /// Append mode for the file descriptor.
        O_APPEND as libc::c_uint;
        /// Synchronized I/O data integrity completion.
        O_DSYNC as libc::c_uint;
        /// No file last access time update.
        O_NOATIME as libc::c_uint;
        /// Nonblocking mode for the file descriptor.
        O_NONBLOCK as libc::c_uint;
        /// Synchronized I/O file integrity completion.
        O_SYNC as libc::c_uint;
    }
}

impl TryFrom<OFlag> for EventFFlags {
    type Error = Errno;

    fn try_from(o_flag: OFlag) -> Result<Self> {
        EventFFlags::from_bits(o_flag.bits() as u32).ok_or(Errno::EINVAL)
    }
}

impl From<EventFFlags> for OFlag {
    fn from(event_f_flags: EventFFlags) -> Self {
        OFlag::from_bits_retain(event_f_flags.bits() as i32)
    }
}

libc_bitflags! {
    /// Configuration options for [`Fanotify::mark()`].
    pub struct MarkFlags: libc::c_uint {
        /// Add the events to the marks.
        FAN_MARK_ADD;
        /// Remove the events to the marks.
        FAN_MARK_REMOVE;
        /// Don't follow symlinks, mark them.
        FAN_MARK_DONT_FOLLOW;
        /// Raise an error if filesystem to be marked is not a directory.
        FAN_MARK_ONLYDIR;
        /// Events added to or removed from the marks.
        FAN_MARK_IGNORED_MASK;
        /// Ignore mask shall survive modify events.
        FAN_MARK_IGNORED_SURV_MODIFY;
        /// Remove all marks.
        FAN_MARK_FLUSH;
        /// Do not pin inode object in the inode cache. Since Linux 5.19.
        FAN_MARK_EVICTABLE;
        /// Events added to or removed from the marks. Since Linux 6.0.
        FAN_MARK_IGNORE;

        /// Default flag.
        FAN_MARK_INODE;
        /// Mark the mount specified by pathname.
        FAN_MARK_MOUNT;
        /// Mark the filesystem specified by pathname. Since Linux 4.20.
        FAN_MARK_FILESYSTEM;

        /// Combination of `FAN_MARK_IGNORE` and `FAN_MARK_IGNORED_SURV_MODIFY`.
        FAN_MARK_IGNORE_SURV;
    }
}

/// Compile version number of fanotify API.
pub const FANOTIFY_METADATA_VERSION: u8 = libc::FANOTIFY_METADATA_VERSION;

/// Abstract over [`libc::fanotify_event_info_fid`], which represents an
/// information record received via [`Fanotify::read_events_with_info_records`].
#[derive(Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
#[allow(missing_copy_implementations)]
pub struct LibcFanotifyFidRecord(libc::fanotify_event_info_fid);

/// Extends LibcFanotifyFidRecord to include file_handle bytes.
/// This allows Rust to move the record around in memory and not lose the file_handle
/// as the libc::fanotify_event_info_fid does not include any of the file_handle bytes.
// Is not Clone due to fd field, to avoid use-after-close scenarios.
#[derive(Debug, Eq, Hash, PartialEq)]
#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct FanotifyFidRecord {
    record: LibcFanotifyFidRecord,
    handle_bytes: *const u8,
}

impl FanotifyFidRecord {
    /// The filesystem id where this event occurred. The value this method returns
    /// differs depending on the host system. Please read the statfs(2) documentation
    /// for more information:
    /// <https://man7.org/linux/man-pages/man2/statfs.2.html#VERSIONS>
    pub fn filesystem_id(&self) -> libc::__kernel_fsid_t {
        self.record.0.fsid
    }

    /// The file handle for the filesystem object where the event occurred. The handle is
    /// represented as a 0-length u8 array, but it actually points to variable-length
    /// file_handle struct.For more information:
    /// <https://man7.org/linux/man-pages/man2/open_by_handle_at.2.html>
    pub fn handle(&self) -> *const u8 {
        self.handle_bytes
    }

    /// The specific info_type for this Fid Record. Fanotify can return an Fid Record
    /// with many different possible info_types. The info_type is not always necessary
    /// but can be useful for connecting similar events together (like a FAN_RENAME) 
    pub fn info_type(&self) -> u8 {
        self.record.0.hdr.info_type
    }
}

/// Abstract over [`libc::fanotify_event_info_error`], which represents an
/// information record received via [`Fanotify::read_events_with_info_records`].
// Is not Clone due to fd field, to avoid use-after-close scenarios.
#[derive(Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
#[allow(missing_copy_implementations)]
#[cfg(target_env = "gnu")]
pub struct FanotifyErrorRecord(libc::fanotify_event_info_error);

#[cfg(target_env = "gnu")]
impl FanotifyErrorRecord {
    /// Errno of the FAN_FS_ERROR that occurred.
    pub fn err(&self) -> Errno {
        Errno::from_raw(self.0.error)
    }

    /// Number of errors that occurred in the filesystem Fanotify in watching.
    /// Only a single FAN_FS_ERROR is stored per filesystem at once. As such, Fanotify
    /// suppresses subsequent error messages and only increments the `err_count` value.
    pub fn err_count(&self) -> u32 {
        self.0.error_count
    }
}

/// Abstract over [`libc::fanotify_event_info_pidfd`], which represents an
/// information record received via [`Fanotify::read_events_with_info_records`].
// Is not Clone due to fd field, to avoid use-after-close scenarios.
#[derive(Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
#[allow(missing_copy_implementations)]
#[cfg(target_env = "gnu")]
pub struct FanotifyPidfdRecord(libc::fanotify_event_info_pidfd);

#[cfg(target_env = "gnu")]
impl FanotifyPidfdRecord {
    /// The process file descriptor that refers to the process responsible for
    /// generating this event. If the underlying pidfd_create fails, `None` is returned.
    pub fn pidfd(&self) -> Option<BorrowedFd> {
        if self.0.pidfd == libc::FAN_NOPIDFD || self.0.pidfd == libc::FAN_EPIDFD
        {
            None
        } else {
            // SAFETY: self.0.pidfd will be opened for the lifetime of `Self`,
            // which is longer than the lifetime of the returned BorrowedFd, so
            // it is safe.
            Some(unsafe { BorrowedFd::borrow_raw(self.0.pidfd) })
        }
    }
}

#[cfg(target_env = "gnu")]
impl Drop for FanotifyPidfdRecord {
    fn drop(&mut self) {
        if self.0.pidfd == libc::FAN_NOFD {
            return;
        }
        let e = close(self.0.pidfd);
        if !std::thread::panicking() && e == Err(Errno::EBADF) {
            panic!("Closing an invalid file descriptor!");
        };
    }
}

/// After a [`libc::fanotify_event_metadata`], there can be 0 or more event_info
/// structs depending on which InitFlags were used in [`Fanotify::init`].
// Is not Clone due to pidfd in `libc::fanotify_event_info_pidfd`
// Other fanotify_event_info records are not implemented as they don't exist in
// the libc crate yet.
#[derive(Debug, Eq, Hash, PartialEq)]
#[allow(missing_copy_implementations)]
pub enum FanotifyInfoRecord {
    /// A [`libc::fanotify_event_info_fid`] event was recieved, usually as
    /// a result of passing [`InitFlags::FAN_REPORT_FID`] or [`InitFlags::FAN_REPORT_DIR_FID`]
    /// into [`Fanotify::init`]. The containing struct includes a `file_handle` for
    /// use with `open_by_handle_at(2)`.
    Fid(FanotifyFidRecord),

    /// A [`libc::fanotify_event_info_error`] event was recieved.
    /// This occurs when a FAN_FS_ERROR occurs, indicating an error with
    /// the watch filesystem object. (such as a bad file or bad link lookup)
    #[cfg(target_env = "gnu")]
    Error(FanotifyErrorRecord),

    /// A [`libc::fanotify_event_info_pidfd`] event was recieved, usually as
    /// a result of passing [`InitFlags::FAN_REPORT_PIDFD`] into [`Fanotify::init`].
    /// The containing struct includes a `pidfd` for reliably determining
    /// whether the process responsible for generating an event has been recycled or terminated
    #[cfg(target_env = "gnu")]
    Pidfd(FanotifyPidfdRecord),
}

/// Abstract over [`libc::fanotify_event_metadata`], which represents an event
/// received via [`Fanotify::read_events`].
// Is not Clone due to fd field, to avoid use-after-close scenarios.
#[derive(Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
#[allow(missing_copy_implementations)]
pub struct FanotifyEvent(libc::fanotify_event_metadata);

impl FanotifyEvent {
    /// Version number for the structure. It must be compared to
    /// `FANOTIFY_METADATA_VERSION` to verify compile version and runtime
    /// version does match. It can be done with the
    /// `FanotifyEvent::check_version` method.
    pub fn version(&self) -> u8 {
        self.0.vers
    }

    /// Checks that compile fanotify API version is equal to the version of the
    /// event.
    pub fn check_version(&self) -> bool {
        self.version() == FANOTIFY_METADATA_VERSION
    }

    /// Mask flags of the events.
    pub fn mask(&self) -> MaskFlags {
        MaskFlags::from_bits_truncate(self.0.mask)
    }

    /// The file descriptor of the event. If the value is `None` when reading
    /// from the fanotify group, this event is to notify that a group queue
    /// overflow occured.
    pub fn fd(&self) -> Option<BorrowedFd> {
        if self.0.fd == libc::FAN_NOFD {
            None
        } else {
            // SAFETY: self.0.fd will be opened for the lifetime of `Self`,
            // which is longer than the lifetime of the returned BorrowedFd, so
            // it is safe.
            Some(unsafe { BorrowedFd::borrow_raw(self.0.fd) })
        }
    }

    /// PID of the process that caused the event. TID in case flag
    /// `FAN_REPORT_TID` was set at group initialization.
    pub fn pid(&self) -> i32 {
        self.0.pid
    }
}

impl Drop for FanotifyEvent {
    fn drop(&mut self) {
        if self.0.fd == libc::FAN_NOFD {
            return;
        }
        let e = close(self.0.fd);
        if !std::thread::panicking() && e == Err(Errno::EBADF) {
            panic!("Closing an invalid file descriptor!");
        };
    }
}

/// Abstraction over the structure to be sent to allow or deny a given event.
#[derive(Debug)]
#[repr(transparent)]
pub struct FanotifyResponse<'a> {
    inner: libc::fanotify_response,
    _borrowed_fd: PhantomData<BorrowedFd<'a>>,
}

impl<'a> FanotifyResponse<'a> {
    /// Create a new response.
    pub fn new(fd: BorrowedFd<'a>, response: Response) -> Self {
        Self {
            inner: libc::fanotify_response {
                fd: fd.as_raw_fd(),
                response: response.bits(),
            },
            _borrowed_fd: PhantomData,
        }
    }
}

libc_bitflags! {
    /// Response to be wrapped in [`FanotifyResponse`] and sent to the [`Fanotify`]
    /// group to allow or deny an event.
    pub struct Response: u32 {
        /// Allow the event.
        FAN_ALLOW;
        /// Deny the event.
        FAN_DENY;
    }
}

/// A fanotify group. This is also a file descriptor that can feed to other
/// interfaces consuming file descriptors.
#[derive(Debug)]
pub struct Fanotify {
    fd: OwnedFd,
}

impl Fanotify {
    /// Initialize a new fanotify group.
    ///
    /// Returns a Result containing a Fanotify instance.
    ///
    /// For more information, see [fanotify_init(2)](https://man7.org/linux/man-pages/man7/fanotify_init.2.html).
    pub fn init(
        flags: InitFlags,
        event_f_flags: EventFFlags,
    ) -> Result<Fanotify> {
        let res = Errno::result(unsafe {
            libc::fanotify_init(flags.bits(), event_f_flags.bits())
        });
        res.map(|fd| Fanotify {
            fd: unsafe { OwnedFd::from_raw_fd(fd) },
        })
    }

    /// Add, remove, or modify an fanotify mark on a filesystem object.
    ///
    /// Returns a Result containing either `()` on success or errno otherwise.
    ///
    /// For more information, see [fanotify_mark(2)](https://man7.org/linux/man-pages/man7/fanotify_mark.2.html).
    pub fn mark<Fd: std::os::fd::AsFd, P: ?Sized + NixPath>(
        &self,
        flags: MarkFlags,
        mask: MaskFlags,
        dirfd: Fd,
        path: Option<&P>,
    ) -> Result<()> {
        let res = crate::with_opt_nix_path(path, |p| unsafe {
            libc::fanotify_mark(
                self.fd.as_raw_fd(),
                flags.bits(),
                mask.bits(),
                dirfd.as_fd().as_raw_fd(),
                p,
            )
        })?;

        Errno::result(res).map(|_| ())
    }

    fn get_struct<T>(&self, buffer: &[u8; 4096], offset: usize) -> T {
        let struct_size = size_of::<T>();
        unsafe {
            let mut struct_obj = MaybeUninit::<T>::uninit();
            std::ptr::copy_nonoverlapping(
                buffer.as_ptr().add(offset),
                struct_obj.as_mut_ptr().cast(),
                (4096 - offset).min(struct_size),
            );
            struct_obj.assume_init()
        }
    }

    /// Read incoming events from the fanotify group.
    ///
    /// Returns a Result containing either a `Vec` of events on success or errno
    /// otherwise.
    ///
    /// # Errors
    ///
    /// Possible errors can be those that are explicitly listed in
    /// [fanotify(2)](https://man7.org/linux/man-pages/man7/fanotify.2.html) in
    /// addition to the possible errors caused by `read` call.
    /// In particular, `EAGAIN` is returned when no event is available on a
    /// group that has been initialized with the flag `InitFlags::FAN_NONBLOCK`,
    /// thus making this method nonblocking.
    pub fn read_events(&self) -> Result<Vec<FanotifyEvent>> {
        let metadata_size = size_of::<libc::fanotify_event_metadata>();
        const BUFSIZ: usize = 4096;
        let mut buffer = [0u8; BUFSIZ];
        let mut events = Vec::new();
        let mut offset = 0;

        let nread = read(&self.fd, &mut buffer)?;

        while (nread - offset) >= metadata_size {
            let metadata = unsafe {
                let mut metadata =
                    MaybeUninit::<libc::fanotify_event_metadata>::uninit();
                ptr::copy_nonoverlapping(
                    buffer.as_ptr().add(offset),
                    metadata.as_mut_ptr().cast(),
                    (BUFSIZ - offset).min(metadata_size),
                );
                metadata.assume_init()
            };

            events.push(FanotifyEvent(metadata));
            offset += metadata.event_len as usize;
        }

        Ok(events)
    }

    /// Read incoming events and information records from the fanotify group.
    ///
    /// Returns a Result containing either a `Vec` of events and information records on success or errno
    /// otherwise.
    ///
    /// # Errors
    ///
    /// Possible errors can be those that are explicitly listed in
    /// [fanotify(2)](https://man7.org/linux/man-pages/man7/fanotify.2.html) in
    /// addition to the possible errors caused by `read` call.
    /// In particular, `EAGAIN` is returned when no event is available on a
    /// group that has been initialized with the flag `InitFlags::FAN_NONBLOCK`,
    /// thus making this method nonblocking.
    #[allow(clippy::cast_ptr_alignment)]    // False positive
    pub fn read_events_with_info_records(
        &self,
    ) -> Result<Vec<(FanotifyEvent, Vec<FanotifyInfoRecord>)>> {
        let metadata_size = size_of::<libc::fanotify_event_metadata>();
        const BUFSIZ: usize = 4096;
        let mut buffer = [0u8; BUFSIZ];
        let mut events = Vec::new();
        let mut offset = 0;

        let nread = read(&self.fd, &mut buffer)?;

        while (nread - offset) >= metadata_size {
            let metadata = unsafe {
                let mut metadata =
                    MaybeUninit::<libc::fanotify_event_metadata>::uninit();
                std::ptr::copy_nonoverlapping(
                    buffer.as_ptr().add(offset),
                    metadata.as_mut_ptr().cast(),
                    (BUFSIZ - offset).min(metadata_size),
                );
                metadata.assume_init()
            };

            let mut remaining_len = metadata.event_len - metadata_size as u32;
            let mut info_records = Vec::new();
            let mut current_event_offset = offset + metadata_size;

            while remaining_len > 0 {
                let header = self
                    .get_struct::<libc::fanotify_event_info_header>(
                        &buffer,
                        current_event_offset,
                    );

                let info_record = match header.info_type {
                    // FanotifyFidRecord can be returned for any of the following info_type.
                    // This isn't found in the fanotify(7) documentation, but the fanotify_init(2) documentation
                    // https://man7.org/linux/man-pages/man2/fanotify_init.2.html
                    libc::FAN_EVENT_INFO_TYPE_FID
                    | libc::FAN_EVENT_INFO_TYPE_DFID
                    | libc::FAN_EVENT_INFO_TYPE_DFID_NAME
                    | libc::FAN_EVENT_INFO_TYPE_NEW_DFID_NAME
                    | libc::FAN_EVENT_INFO_TYPE_OLD_DFID_NAME => {
                        let record = self
                            .get_struct::<libc::fanotify_event_info_fid>(
                                &buffer,
                                current_event_offset,
                            );

                        let record_ptr: *const libc::fanotify_event_info_fid = unsafe {
                            buffer.as_ptr().add(current_event_offset)
                                as *const libc::fanotify_event_info_fid
                        };

                        let file_handle_ptr = unsafe { record_ptr.add(1) as *const u8 };

                        Some(FanotifyInfoRecord::Fid(FanotifyFidRecord {
                            record: LibcFanotifyFidRecord(record),
                            handle_bytes: file_handle_ptr,
                        }))
                    }
                    #[cfg(target_env = "gnu")]
                    libc::FAN_EVENT_INFO_TYPE_ERROR => {
                        let record = self
                            .get_struct::<libc::fanotify_event_info_error>(
                                &buffer,
                                current_event_offset,
                            );

                        Some(FanotifyInfoRecord::Error(FanotifyErrorRecord(
                            record,
                        )))
                    }
                    #[cfg(target_env = "gnu")]
                    libc::FAN_EVENT_INFO_TYPE_PIDFD => {
                        let record = self
                            .get_struct::<libc::fanotify_event_info_pidfd>(
                                &buffer,
                                current_event_offset,
                            );
                        Some(FanotifyInfoRecord::Pidfd(FanotifyPidfdRecord(
                            record,
                        )))
                    }
                    // Ignore unsupported events
                    _ => None,
                };

                if let Some(record) = info_record {
                    info_records.push(record);
                }

                remaining_len -= header.len as u32;
                current_event_offset += header.len as usize;
            }

            // libc::fanotify_event_info_header

            events.push((FanotifyEvent(metadata), info_records));
            offset += metadata.event_len as usize;
        }

        Ok(events)
    }

    /// Write an event response on the fanotify group.
    ///
    /// Returns a Result containing either `()` on success or errno otherwise.
    ///
    /// # Errors
    ///
    /// Possible errors can be those that are explicitly listed in
    /// [fanotify(2)](https://man7.org/linux/man-pages/man7/fanotify.2.html) in
    /// addition to the possible errors caused by `write` call.
    /// In particular, `EAGAIN` or `EWOULDBLOCK` is returned when no event is
    /// available on a group that has been initialized with the flag
    /// `InitFlags::FAN_NONBLOCK`, thus making this method nonblocking.
    pub fn write_response(&self, response: FanotifyResponse) -> Result<()> {
        write(self.fd.as_fd(), unsafe {
            std::slice::from_raw_parts(
                (&response.inner as *const libc::fanotify_response).cast(),
                size_of::<libc::fanotify_response>(),
            )
        })?;
        Ok(())
    }
}

impl FromRawFd for Fanotify {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Fanotify {
            fd: unsafe { OwnedFd::from_raw_fd(fd) },
        }
    }
}

impl AsFd for Fanotify {
    fn as_fd(&'_ self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl AsRawFd for Fanotify {
    fn as_raw_fd(&self) -> RawFd
    {
        self.fd.as_raw_fd()
    }
}

impl From<Fanotify> for OwnedFd {
    fn from(value: Fanotify) -> Self {
        value.fd
    }
}

impl Fanotify {
    /// Constructs a `Fanotify` wrapping an existing `OwnedFd`.
    ///
    /// # Safety
    ///
    /// `OwnedFd` is a valid `Fanotify`.
    pub unsafe fn from_owned_fd(fd: OwnedFd) -> Self {
        Self {
            fd
        }
    }
}
