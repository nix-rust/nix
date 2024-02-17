//! System V IPC: Message Queue

use crate::errno::Errno;
use crate::Result;
use libc::{c_int, c_long, key_t};

libc_bitflags! {
    /// `flag` argument of [`MsgQueue::new()`].
    pub struct MsgCreateFlag: c_int {
        /// Create key if key does not exist.
        IPC_CREAT;
        /// Fail if key exists.
        IPC_EXCL;
        /// Return error on wait.
        IPC_NOWAIT;
        /// Permits read access when the effective user ID of the caller matches
        /// either `msg_perm.cuid` or `msg_perm.uid`
        S_IRUSR as c_int;
        /// Permits write access when the effective user ID of the caller matches
        /// either `msg_perm.cuid` or `msg_perm.uid`
        S_IWUSR as c_int;
        /// Permits read access when the effective group ID of the caller matches
        /// either `msg_perm.cgid` or `msg_perm.gid`
        S_IRGRP as c_int;
        /// Permits write access when the effective group ID of the caller matches
        /// either `msg_perm.cgid` or `msg_perm.gid`
        S_IWGRP as c_int;
        /// Permits other read access
        S_IROTH as c_int;
        /// Permits other write access
        S_IWOTH as c_int;
    }
}

libc_bitflags! {
    /// `flag` argument of [`MsgQueue::send()`] and [`MsgQueue::recv()`].
    pub struct MsgSendRecvFlag: c_int {
        /// Return immediately if no message of the requested type is in the
        /// queue. The system call fails with errno set to ENOMSG.
        IPC_NOWAIT;
        /// Nondestructively fetch a copy of the message at the ordinal position
        /// in the queue specified by msgtyp (messages are considered to be
        /// numbered starting at 0).
        ///
        /// This flag must be specified in conjunction with `IPC_NOWAIT`, with the
        /// result that, if there is no message available at the given position,
        /// the call fails immediately with the error `ENOMSG`.  Because they
        /// alter the meaning of msgtyp in orthogonal ways, `MSG_COPY` and
        /// `MSG_EXCEPT` may not both be specified in msgflg.
        ///
        /// The MSG_COPY flag was added for the implementation of the kernel
        /// checkpoint-restore facility and is available only if the kernel was
        /// built with the `CONFIG_CHECKPOINT_RESTORE` option.
        MSG_COPY;
        /// Used with `msgtyp` greater than 0 to read the first message in the queue
        /// with message type that differs from `msgtyp`.
        MSG_EXCEPT;
        /// To truncate the message text if longer than `msgsz` bytes.
        MSG_NOERROR;
    }
}

mod private {
    /// A sealed marker trait representing types that can be used as messages.
    pub trait Msg {}
}

/// Creates your custom message type.
#[macro_export]
macro_rules! msg {
    ($name:ident { $($fields:tt)* }) => {
        #[repr(C)]
        #[derive(Debug)]
        struct $name {
            msg_type: libc::c_long,
            $($fields)*
        }

        impl private::Msg for $name { }
    };
}

/// Operations that can be performed on a [`MsgQueue`].
#[derive(Debug)]
pub enum MsgCmd<'a> {
    /// Copy information from the kernel data structure to the buf.
    Stat(&'a mut ()),
    /// Write the values of some members of the `msqid_ds` structure to the
    /// kernel data structure associated with this message queue,
    Set(&'a ()),
    /// Immediately remove the message queue
    RMID,
}

/// Represents a System V message queue.
#[derive(Debug, Copy, Clone)]
pub struct MsgQueue(c_int);

impl MsgQueue {
    /// Returns the message queue associated with the value of the `key` argument,
    /// or obtains a queue that was previously created.
    pub fn new(key: key_t, flag: MsgCreateFlag) -> Result<Self> {
        let flag = flag.bits();
        let res = unsafe { libc::msgget(key, flag) };

        Errno::result(res).map(|raw_id| MsgQueue(raw_id))
    }

    /// Sends a message to this queue.
    pub fn send<T: private::Msg>(
        &self,
        msg: T,
        flag: MsgSendRecvFlag,
    ) -> Result<()> {
        let flag = flag.bits();
        let msg_ptr = (&msg as *const T).cast();
        let res = unsafe {
            libc::msgsnd(self.0, msg_ptr, std::mem::size_of::<T>(), flag)
        };

        Errno::result(res).map(|_| ())
    }

    /// Receives a message from this queue.
    pub fn recv<T: private::Msg>(
        &self,
        flag: MsgSendRecvFlag,
        msg_type: c_long,
    ) -> Result<T> {
        let mut buf = std::mem::MaybeUninit::<T>::uninit();
        let flag = flag.bits();
        let res = unsafe {
            libc::msgrcv(
                self.0,
                buf.as_mut_ptr().cast(),
                std::mem::size_of::<T>(),
                msg_type,
                flag,
            )
        };
        Errno::result(res)?;

        // SAFETY: TODO
        Ok(unsafe { buf.assume_init() })
    }

    /// Performs the operation specified by `cmd` on the queue.
    pub fn ctl(&self, cmd: MsgCmd) -> Result<()> {
        match cmd {
            MsgCmd::Stat(_) => {
                todo!()
            }
            MsgCmd::Set(_) => {
                todo!()
            }
            MsgCmd::RMID => unsafe {
                Errno::result(libc::msgctl(
                    self.0,
                    libc::IPC_RMID,
                    std::ptr::null_mut(),
                ))
                .map(|_| ())
            },
        }
    }
}
