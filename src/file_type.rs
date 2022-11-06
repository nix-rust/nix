//! File type conversion utilities

/// Type of file referenced by a directory entry
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum FileType {
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
    /// Unknown
    Unknown,
}

impl From<libc::c_uchar> for FileType {
    fn from(value: libc::c_uchar) -> Self {
        match value {
            libc::DT_FIFO => Self::Fifo,
            libc::DT_CHR => Self::CharacterDevice,
            libc::DT_DIR => Self::Directory,
            libc::DT_BLK => Self::BlockDevice,
            libc::DT_REG => Self::File,
            libc::DT_LNK => Self::Symlink,
            libc::DT_SOCK => Self::Socket,
            /* libc::DT_UNKNOWN | */ _ => Self::Unknown,
        }
    }
}
