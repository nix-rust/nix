//! Safe wrapper around a SystemV shared memory segment
//!

use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::{null, null_mut},
};

use crate::Result;
use crate::{errno::Errno, sys::stat::Mode};

use libc::{self, c_int, c_void, key_t, shmid_ds};

#[derive(Debug)]
/// Safe wrapper to create and connect to a SystemV shared memory segment.
///
/// # Example
///
/// ```no_run
/// # use nix::errno::Errno;
/// # use nix::sys::system_v::shm::*;
/// # use nix::sys::stat::Mode;
/// #
/// struct MyData(i64);
///
/// const MY_KEY: i32 = 1337;
/// let mem_segment = Shm::<MyData>::create_and_connect(
///     MY_KEY,
///     Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO,
/// )?;
/// let mut shared_memory = mem_segment.attach(ShmatFlag::empty())?;
/// # Ok::<(), Errno>(())
/// ```
///
pub struct Shm<T> {
    id: c_int,
    _phantom: PhantomData<T>,
}

impl<T> Shm<T> {
    /// Attach to the current SystemV shared memory segment.
///
    /// To create a new segment, use [`Shm::create_and_connect`].\
    /// If you need more customisation, use the unsafe version,
    /// [`Shm::shmget`], with the key [`ShmgetFlag::IPC_CREAT`].
    ///
    /// Attaching a segment to a specific adress isn't supported. This is
    /// because there is no way to create a void pointer on rust.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use nix::errno::Errno;
    /// # use nix::sys::system_v::shm::*;
    /// # use nix::sys::stat::Mode;
    /// #
    /// struct MyData(i64);
    ///
    /// const MY_KEY: i32 = 1337;
    /// let mem_segment = Shm::<MyData>::create_and_connect(
    ///     MY_KEY,
    ///     Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO,
    /// )?;
    /// let mut shared_memory = mem_segment.attach(ShmatFlag::empty())?;
    /// # Ok::<(), Errno>(())
    /// ```
    ///
    pub fn attach(&self, shmat_flag: ShmatFlag) -> Result<SharedMemory<T>> {
        unsafe {
            Ok(SharedMemory::<T> {
                id: self.id,
                shm: ManuallyDrop::new(Box::from_raw(self.shmat(shmat_flag)?)),
            })
        }
    }

    /// Creates and returns a new System V shared memory segment identifier.
    ///
    /// # Example
    /// ```no_run
    /// # use nix::errno::Errno;
    /// # use nix::sys::system_v::shm::*;
    /// # use nix::sys::stat::Mode;
    /// #
    /// struct MyData(i64);
    /// const MY_KEY: i32 = 1337;
    ///
    /// let mem_segment = Shm::<MyData>::create_and_connect(
    ///     MY_KEY,
    ///     Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO,
    /// )?;
    /// # Ok::<(), Errno>(())
    /// ```
    ///
    pub fn create_and_connect(key: key_t, mode: Mode) -> Result<Self> {
        let size = std::mem::size_of::<T>();
        // This is the main difference between this function and [`Shm::shmget`]
        // Because we are always creating a new segment, we can be sure that the size match
        let shmget_flag = ShmgetFlag::IPC_CREAT | ShmgetFlag::IPC_EXCL;
        let flags = mode.bits() as i32 | shmget_flag.bits();
        let id = Errno::result(unsafe { libc::shmget(key, size, flags) })?;
        Ok(Self {
            id,
            _phantom: PhantomData,
        })
    }

    /// Creates and returns a new, or returns an existing, System V shared memory
    /// segment identifier.
    ///
    /// For more information, see [`shmget(2)`].
    ///
    /// # Safety
    ///
    /// If you are using this function to connect to an existing memory segment,
    /// care must be taken that the generic type `T` matches what is actually
    /// stored on the memory segment.\
    /// For example, if a memory segment of size 4 bytes exist, and you connect
    /// with a type of size 8 bytes, then undefined behaviour will be invoked.
    ///
    /// # Example
    ///
    /// ## Connecting to an existing shared memory segment
    ///
    /// ```no_run
    /// # use nix::errno::Errno;
    /// # use nix::sys::system_v::shm::*;
    /// # use nix::sys::stat::Mode;
    /// #
    /// struct MyData(i64);
    /// const MY_KEY: i32 = 1337;
    ///
    /// unsafe {
    ///     let mem_segment = Shm::<MyData>::shmget(
    ///         MY_KEY,
    ///         ShmgetFlag::empty(),
    ///         Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO,
    ///     )?;
    /// }
    /// # Ok::<(), Errno>(())
    /// ```
    ///
    /// [`shmget(2)`]: https://man7.org/linux/man-pages/man2/shmget.2.html
    pub unsafe fn shmget(
        key: key_t,
        shmget_flag: ShmgetFlag,
        mode: Mode,
    ) -> Result<Self> {
        let size = std::mem::size_of::<T>();
        let flags = mode.bits() as i32 | shmget_flag.bits();
        let id = Errno::result(unsafe { libc::shmget(key, size, flags) })?;
        Ok(Self {
            id,
            _phantom: PhantomData,
        })
    }

    // -- Private --

    /// Attaches the System V shared memory segment identified by `shmid` to the
    /// address space of the calling process.
    ///
    /// This is called automatically on [`Shm::attach`].
    ///
    /// For more information, see [`shmat(2)`].
    ///
    /// [`shmat(2)`]: https://man7.org/linux/man-pages/man2/shmat.2.html
    fn shmat(&self, shmat_flag: ShmatFlag) -> Result<*mut T> {
        Errno::result(unsafe {
            libc::shmat(self.id, ptr::null(), shmat_flag.bits())
        })
        .map(|ok| ok.cast::<T>())
    }
}

#[derive(Debug)]
/// Safe wrapper around a SystemV shared memory segment data
///
/// This is a smart pointer, and so implement the [`Deref`] and [`DerefMut`] traits.
/// This means that you can work with the shared memory segment like you would with a [`Box`].
///
/// This type does not automatically destroy the shared memory segment, but
/// only detach from it using RAII.
///
/// To delete a shared memory segment, use [`SharedMemory::shmctl`], with the key [`ShmctlFlag::IPC_RMID`].
///
/// # Example
///
/// ```no_run
/// # use nix::errno::Errno;
/// # use nix::sys::system_v::shm::*;
/// # use nix::sys::stat::Mode;
/// #
/// struct MyData(i64);
/// const MY_KEY: i32 = 1337;
///
/// let mem_segment = Shm::<MyData>::create_and_connect(
///     MY_KEY,
///     Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO,
/// )?;
/// let mut shared_memory = mem_segment.attach(ShmatFlag::empty())?;
///
/// // This is writing to the stored [`MyData`] struct
/// shared_memory.0 = 0xDEADBEEF;
/// # Ok::<(), Errno>(())
/// ```
///
pub struct SharedMemory<T> {
    id: c_int,
    shm: ManuallyDrop<Box<T>>,
}

impl<T> Deref for SharedMemory<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.shm
    }
}
impl<T> DerefMut for SharedMemory<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shm
    }
}

impl<T> Drop for SharedMemory<T> {
    fn drop(&mut self) {
        Self::shmdt(self).expect("SharedMemory detach from SysV IPC");
    }
}

impl<T> SharedMemory<T> {
    /// Performs control operation specified by `cmd` on the System V shared
    /// memory segment given by `shmid`.
    ///
    /// For more information, see [`shmctl(2)`].
    ///
    /// # Example
    ///
    /// ## Deleting a shared memory segment
    ///
    /// ```no_run
    /// # use nix::errno::Errno;
    /// # use nix::sys::system_v::shm::*;
    /// # use nix::sys::stat::Mode;
    /// #
    /// struct MyData(i64);
    /// const MY_KEY: i32 = 1337;
    ///
    /// let mem_segment = Shm::<MyData>::create_and_connect(
    ///     MY_KEY,
    ///     Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO,
    /// )?;
    /// let shared_memory = mem_segment.attach(ShmatFlag::empty())?;
    ///
    /// let _ = shared_memory.shmctl(ShmctlFlag::IPC_RMID, None)?;
    /// # Ok::<(), Errno>(())
    /// ```
    ///
    /// [`shmctl(2)`]: https://man7.org/linux/man-pages/man2/shmctl.2.html
    pub fn shmctl(
        &self,
        shm_cmd: ShmctlFlag,
        buf: Option<&mut shmid_ds>,
    ) -> Result<c_int> {
        let buf_ptr: *mut shmid_ds = match buf {
            Some(ptr) => ptr::from_mut(ptr),
            None => ptr::null_mut(),
        };
        Errno::result(unsafe { libc::shmctl(self.id, shm_cmd.bits(), buf_ptr) })
    }

    // -- Private --

    /// Performs the reverse of [`SharedMemory::shmat`], detaching the shared memory segment at
    /// the given address from the address space of the calling process.
    ///
    /// This is called automatically on [`Drop`].
    ///
    /// For more information, see [`shmdt(2)`].
    ///
    /// [`shmdt(2)`]: https://man7.org/linux/man-pages/man2/shmdt.2.html
    fn shmdt(&self) -> Result<()> {
        let shmaddr_ref: *const T = &**self;
        Errno::result(unsafe { libc::shmdt(shmaddr_ref.cast::<c_void>()) })
            .map(drop)
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
        #[cfg(linux)]
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
        #[cfg(linux)]
        SHM_NORESERVE;
    }
);
libc_bitflags! {
    /// Valid flags for the third parameter of the function [`shmat`]
    pub struct ShmatFlag: c_int
    {
        /// Allow the contents of the segment to be executed. The caller must
        /// have execute permission on the segment.
        #[cfg(linux)]
        SHM_EXEC;
        /// This flag specifies that the mapping of the segment should replace
        /// any existing mapping in the range starting at shmaddr and
        /// continuing for the size of the segment.
        /// (Normally, an EINVAL error would result if a mapping already exists
        /// in this address range.)
        /// In this case, shmaddr must not be NULL.
        #[cfg(linux)]
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

libc_bitflags!(
    /// Valid flags for the second parameter of the function [`shmctl`]
    pub struct ShmctlFlag: c_int {
        /// Returns the index of the highest used entry in the kernel's internal
        /// array recording information about all shared memory segment
        #[cfg(linux)]
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
        #[cfg(linux)]
        SHM_LOCK;
        /// Unlock the segment, allowing it to be swapped out.
        #[cfg(linux)]
        SHM_UNLOCK;
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use parking_lot::Mutex;

    static SHM_MTX: Mutex<()> = Mutex::new(());

    const SHM_TEST: i32 = 1337;

    #[derive(Debug)]
    /// Test struct used to store some data on the shared memory zone
    ///
    struct TestData {
        data: i64,
    }

    #[derive(Debug)]
    struct FixtureShm {
        ipc: SharedMemory<TestData>,
    }

    impl FixtureShm {
        fn setup() -> Result<Self> {
            let id = SharedMemory::<TestData>::shmget(
                SHM_TEST,
                ShmgetFlag::IPC_CREAT | ShmgetFlag::IPC_EXCL,
                Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO,
            )?;
            Ok(Self {
                ipc: SharedMemory::<TestData>::new(
                    id,
                    None,
                    ShmatFlag::empty(),
                    Mode::empty(),
                )?,
            })
        }
    }

    impl Drop for FixtureShm {
        fn drop(&mut self) {
            let _ = self
                .ipc
                .shmctl(ShmctlFlag::IPC_RMID, None, Mode::empty())
                .map_err(|_| {
                    panic!("Failed to delete the test shared memory zone")
                });
        }
    }

    #[test]
    fn create_ipc() -> Result<()> {
        let _m = SHM_MTX.lock();

        FixtureShm::setup()?;
        Ok(())
    }

    #[test]
    fn create_ipc_already_exist() -> Result<()> {
        let _m = SHM_MTX.lock();

        // Keep the IPC in scope, so we don't destroy it
        let _ipc = FixtureShm::setup()?;
        let expected = Errno::EEXIST;
        let actual = FixtureShm::setup().expect_err("Return EExist");

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn create_ipc_and_get_value() -> Result<()> {
        let _m = SHM_MTX.lock();

        let mut sem = FixtureShm::setup()?;
        let expected = 0xDEADBEEF;
        sem.ipc.data = expected;

        let actual = SharedMemory::<TestData>::new(
            sem.ipc.id,
            None,
            ShmatFlag::empty(),
            Mode::empty(),
        )?.data;
        assert_eq!(expected, actual);
        Ok(())
    }
}
