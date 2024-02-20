use std::ptr::{null, null_mut};

use crate::errno::Errno;
use crate::Result;

use libc::{self, c_int, c_short, c_void, key_t, size_t};
#[cfg(target_env = "gnu")]
use libc::{shmid_ds, semid_ds, seminfo};

#[derive(Debug, Default, Clone, Copy)]
/// Type used to transform a raw number to an octal permission, while performing a clamp to u9
///
/// # Example
///
/// ```
/// # use nix::errno::Errno;
/// # use nix::sys::system_v::Permissions;
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

libc_bitflags!(
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
        SHM_NORESERVE;
    }
);
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

libc_bitflags!(
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
);
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
        SHM_EXEC;
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
    shmaddr: Option<c_void>,
    shmflg: Vec<ShmatFlag>,
    permission: Permissions,
) -> Result<*mut c_void> {
    let shmaddr_ptr: *const c_void = match shmaddr {
        Some(_) => &mut shmaddr.unwrap(),
        None => null(),
    };
    let flags = permission.to_octal(shmflg);
    Errno::result(unsafe { libc::shmat(shmid, shmaddr_ptr, flags) })
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
pub fn shmdt(shmaddr: c_void) -> Result<()> {
    let shmaddr_ptr: *const c_void = &shmaddr;
    Errno::result(unsafe { libc::shmdt(shmaddr_ptr) }).map(drop)
}

libc_bitflags!(
    /// Valid flags for the second parameter of the function [`shmctl`]
    pub struct ShmctlFlag: c_int {
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
        // SHM_INFO;
        // SHM_STAT;
        // SHM_STAT_ANY;
        /// Prevent swapping of the shared memory segment. The caller must
        /// fault in any pages that are required to be present after locking is
        /// enabled.
        /// If a segment has been locked, then the (nonstandard) SHM_LOCKED
        /// flag of the shm_perm.mode field in the associated data structure
        /// retrieved by IPC_STAT will be set.
        SHM_LOCK;
        /// Unlock the segment, allowing it to be swapped out.
        SHM_UNLOCK;
    }
);
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
    buf: Option<shmid_ds>,
    permission: Permissions,
) -> Result<c_int> {
    let buf_ptr: *mut shmid_ds = match buf {
        Some(_) => &mut buf.unwrap(),
        None => null_mut(),
    };
    let command = permission.to_octal(vec![cmd]);
    Errno::result(unsafe { libc::shmctl(shmid, command, buf_ptr) })
}

#[derive(Debug)]
/// Called as the fourth parameter of the function [`semctl`]
///
pub enum Semun {
    /// Value for SETVAL
    val(c_int),
    /// Buffer for IPC_STAT, IPC_SET
    #[cfg(target_env = "gnu")]
    buf(*mut semid_ds),
    /// Array for GETALL, SETALL
    array(*mut c_short),
    /// Buffer for IPC_INFO
    #[cfg(target_env = "gnu")]
    __buf(*mut seminfo),
}
libc_bitflags! (
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
        /// Return information about system-wide semaphore limits and
        /// parameters in the structure pointed to by arg.__buf. This structure
        /// is of type [`seminfo`].
        IPC_INFO;
        // TODO: None of the one following are defined in libc/linux
        // SEM_INFO;
        // SEM_STAT;
        // SEM_STAT_ANY;
        // GETALL;
        // GETNCNT;
        // GETPID;
        // GETVAL;
        // GETZCNT;
        // SETALL;
        // SETVAL;
    }
);
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
