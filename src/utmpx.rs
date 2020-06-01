//! User accounting database definitions.
//!
//! This module contains safe wrappers and helpers for accounting-related
//! functions defined in `utmpx.h`. For further details, check
//! [standard specifications](http://pubs.opengroup.org/onlinepubs/9699919799/basedefs/utmpx.h.html).
//!
//! # Safety
//!
//! Standard-defined `utmpx` functions are explicitly **NOT** thread-safe.
//! The caller must make sure to operate only on one `Utmp` singleton per process.

use crate::errno::ErrnoSentinel;
use crate::sys::time::TimeVal;
use crate::unistd::Pid;
use crate::{Errno, Error, Result};
use std::convert::TryFrom;
use std::ffi::CStr;
use std::marker::PhantomData;

libc_enum! {
    /// Valid `UtmpEntry` entry types.
    #[repr(i16)]
    pub enum EntryType {
        /// No valid user accounting information.
        EMPTY,
        /// Change in system run-level.
        #[cfg(target_os = "linux")]
        RUN_LVL,
        /// Identifies time of system boot.
        BOOT_TIME,
        /// Identifies time of system shutdown.
        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos"
        ))]
        SHUTDOWN_TIME,
        /// Identifies time after system clock changed.
        NEW_TIME,
        /// Identifies time when system clock changed.
        OLD_TIME,
        /// Identifies a process spawned by the init process.
        INIT_PROCESS,
        /// Identifies the session leader of a logged-in user.
        LOGIN_PROCESS,
        /// Identifies a process.
        USER_PROCESS,
        /// Identifies a session leader who has exited.
        DEAD_PROCESS,
        /// Accounting.
        #[cfg(target_env = "gnu")]
        ACCOUNTING,
    }
}

impl TryFrom<i16> for EntryType {
    type Error = Error;

    /// Try to build an `EntryType` from its discriminant.
    fn try_from(value: i16) -> Result<Self> {
        match value {
            libc::EMPTY => Ok(EntryType::EMPTY),
            #[cfg(target_os = "linux")]
            libc::RUN_LVL => Ok(EntryType::RUN_LVL),
            libc::BOOT_TIME => Ok(EntryType::BOOT_TIME),
            #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos"
            ))]
            libc::SHUTDOWN_TIME => Ok(EntryType::SHUTDOWN_TIME),
            libc::OLD_TIME => Ok(EntryType::OLD_TIME),
            libc::NEW_TIME => Ok(EntryType::NEW_TIME),
            libc::USER_PROCESS => Ok(EntryType::USER_PROCESS),
            libc::INIT_PROCESS => Ok(EntryType::INIT_PROCESS),
            libc::LOGIN_PROCESS => Ok(EntryType::LOGIN_PROCESS),
            libc::DEAD_PROCESS => Ok(EntryType::DEAD_PROCESS),
            #[cfg(target_env = "gnu")]
            libc::ACCOUNTING => Ok(EntryType::ACCOUNTING),
            _ => Err(Error::invalid_argument()),
        }
    }
}

/// Utmp login record.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct UtmpEntry {
    ut_host: String,
    ut_id: Vec<u8>,
    ut_line: String,
    ut_pid: Pid,
    ut_tv: TimeVal,
    ut_type: EntryType,
    ut_user: String,
}

impl UtmpEntry {
    /// Return the type of this entry.
    pub fn entry_type(&self) -> &EntryType {
        &self.ut_type
    }

    /// Return the line related to this entry.
    pub fn line(&self) -> &str {
        &self.ut_line
    }

    /// Return the user related to this entry.
    pub fn user(&self) -> &str {
        &self.ut_user
    }

    /// Return the PID related to this entry.
    pub fn pid(&self) -> &Pid {
        &self.ut_pid
    }

    /// Try to build an umtp entry from a raw pointer.
    fn try_from_ptr(ptr: *mut libc::utmpx) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::invalid_argument());
        }

        // The lifetime of this whole buffer is very shady and the overall
        // content small, so we copy the data out and start owning it.
        let entry = unsafe { *ptr };
        let res = Self {
            ut_host: Self::charbuf_to_string(&entry.ut_host)?,
            ut_id: Self::charbuf_to_bytes(&entry.ut_id),
            ut_line: Self::charbuf_to_string(&entry.ut_line)?,
            ut_pid: Pid::from_raw(entry.ut_pid),
            ut_tv: TimeVal::from_usec(entry.ut_tv.tv_sec.into(), entry.ut_tv.tv_usec.into()),
            ut_type: EntryType::try_from(entry.ut_type)?,
            ut_user: Self::charbuf_to_string(&entry.ut_user)?,
        };

        Ok(res)
    }

    /// Copy out buffer content, as a bytes vector.
    fn charbuf_to_bytes(input: &[libc::c_char]) -> Vec<u8> {
        let bytes = unsafe { &*(input as *const _ as *const [u8]) };
        let mut buf = vec![0u8; bytes.len()];
        buf.copy_from_slice(bytes);
        buf
    }

    /// Copy out buffer content, as a string, up to the first NUL-terminator.
    fn charbuf_to_string(input: &[libc::c_char]) -> Result<String> {
        // Ensure input is NUL-terminated (`ut_id` often isn't).
        let mut buf: Vec<_> = input.to_vec();
        buf.push(0);

        let wrapped = unsafe { CStr::from_ptr(buf.as_ptr()) };
        wrapped
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| Error::InvalidUtf8)
    }
}

/// Iterator over accounting entries.
///
/// Created using `Utmp.entries()`.
#[derive(PartialEq, Eq, Debug)]
pub struct UtmpIter<'a> {
    db: &'a mut Utmp,
}

impl<'a> Iterator for UtmpIter<'a> {
    type Item = Result<UtmpEntry>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = unsafe { libc::getutxent() };
        // Stop iterating on listing error.
        let entry_ptr = match Errno::result(res) {
            Ok(ptr) => ptr,
            _ => return None,
        };
        Some(UtmpEntry::try_from_ptr(entry_ptr))
    }
}

/// Utmp accounting database.
#[derive(PartialEq, Eq, Debug)]
pub struct Utmp {
    /// Mark the DB and iterator as non-Send and non-Sync.
    _phantom: PhantomData<*mut ()>,
}

impl Utmp {
    /// Open the default `utmp` database, positioning pointer at the beginning.
    ///
    /// # Safety
    ///
    /// This operation is unsafe because it mutates global libc state. In order to
    /// safely invoke this, the caller must ensure that nothing else in the process
    /// is accessing the `utmp` database at the same time.
    pub unsafe fn open() -> Result<Utmp> {
        let mut db = Utmp {
            _phantom: PhantomData,
        };
        db.rewind();
        Ok(db)
    }

    /// Iterate through accounting entries.
    pub fn entries(&mut self) -> UtmpIter {
        UtmpIter { db: self }
    }

    /// Rewind the file pointer to the beginning of the database.
    pub fn rewind(&mut self) {
        unsafe { libc::setutxent() }
    }
}

impl Drop for Utmp {
    fn drop(&mut self) {
        unsafe { libc::endutxent() }
    }
}

impl ErrnoSentinel for *mut libc::utmpx {
    fn sentinel() -> Self {
        std::ptr::null_mut()
    }
}
