use std::fmt;
use std::os::errno;
use std::num::from_uint;
use libc::c_int;

pub use self::consts::*;
pub use self::consts::Errno::*;

pub type SysResult<T> = Result<T, SysError>;

#[derive(Clone, PartialEq, Copy)]
pub struct SysError {
    pub kind: Errno,
}

impl SysError {
    pub fn last() -> SysError {
        match from_uint(errno()) {
            Some(no) => SysError::new(no),
            _ => SysError::new(UnknownErrno)
        }
    }

    pub fn new(kind: Errno) -> SysError {
        SysError { kind: kind }
    }

    pub fn errno(&self) -> usize {
        self.kind as usize
    }

    pub fn desc(&self) -> &'static str {
        match self.kind {
            UnknownErrno    => "Unknown errno",
            EPERM           => "Operation not permitted",
            ENOENT          => "No such file or directory",
            ESRCH           => "No such process",
            EINTR           => "Interrupted system call",
            EIO             => "I/O error",
            ENXIO           => "No such device or address",
            E2BIG           => "Argument list too long",
            ENOEXEC         => "Exec format error",
            EBADF           => "Bad file number",
            ECHILD          => "No child processes",
            EAGAIN          => "Try again",
            ENOMEM          => "Out of memory",
            EACCES          => "Permission denied",
            EFAULT          => "Bad address",
            ENOTBLK         => "Block device required",
            EBUSY           => "Device or resource busy",
            EEXIST          => "File exists",
            EXDEV           => "Cross-device link",
            ENODEV          => "No such device",
            ENOTDIR         => "Not a directory",
            EISDIR          => "Is a directory",
            EINVAL          => "Invalid argument",
            ENFILE          => "File table overflow",
            EMFILE          => "Too many open files",
            ENOTTY          => "Not a typewriter",
            ETXTBSY         => "Text file busy",
            EFBIG           => "File too large",
            ENOSPC          => "No space left on device",
            ESPIPE          => "Illegal seek",
            EROFS           => "Read-only file system",
            EMLINK          => "Too many links",
            EPIPE           => "Broken pipe",
            EDOM            => "Math argument out of domain of func",
            ERANGE          => "Math result not representable",
            EDEADLK         => "Resource deadlock would occur",
            ENAMETOOLONG    => "File name too long",
            ENOLCK          => "No record locks available",
            ENOSYS          => "Function not implemented",
            ENOTEMPTY       => "Directory not empty",
            ELOOP           => "Too many symbolic links encountered",
            ENOMSG          => "No message of desired type",
            EIDRM           => "Identifier removed",
            EINPROGRESS     => "Operation now in progress",
            EALREADY        => "Operation already in progress",
            ENOTSOCK        => "Socket operation on non-socket",
            EDESTADDRREQ    => "Destination address required",
            EMSGSIZE        => "Message too long",
            EPROTOTYPE      => "Protocol wrong type for socket",
            ENOPROTOOPT     => "Protocol not available",
            EPROTONOSUPPORT => "Protocol not supported",
            ESOCKTNOSUPPORT => "Socket type not supported",
            EPFNOSUPPORT    => "Protocol family not supported",
            EAFNOSUPPORT    => "Address family not supported by protocol",
            EADDRINUSE      => "Address already in use",
            EADDRNOTAVAIL   => "Cannot assign requested address",
            ENETDOWN        => "Network is down",
            ENETUNREACH     => "Network is unreachable",
            ENETRESET       => "Network dropped connection because of reset",
            ECONNABORTED    => "Software caused connection abort",
            ECONNRESET      => "Connection reset by peer",
            ENOBUFS         => "No buffer space available",
            EISCONN         => "Transport endpoint is already connected",
            ENOTCONN        => "Transport endpoint is not connected",
            ESHUTDOWN       => "Cannot send after transport endpoint shutdown",
            ETOOMANYREFS    => "Too many references: cannot splice",
            ETIMEDOUT       => "Connection timed out",
            ECONNREFUSED    => "Connection refused",
            EHOSTDOWN       => "Host is down",
            EHOSTUNREACH    => "No route to host",

            #[cfg(target_os = "linux")]
            ECHRNG          => "Channel number out of range",

            #[cfg(target_os = "linux")]
            EL2NSYNC        => "Level 2 not synchronized",

            #[cfg(target_os = "linux")]
            EL3HLT          => "Level 3 halted",

            #[cfg(target_os = "linux")]
            EL3RST          => "Level 3 reset",

            #[cfg(target_os = "linux")]
            ELNRNG          => "Link number out of range",

            #[cfg(target_os = "linux")]
            EUNATCH         => "Protocol driver not attached",

            #[cfg(target_os = "linux")]
            ENOCSI          => "No CSI structure available",

            #[cfg(target_os = "linux")]
            EL2HLT          => "Level 2 halted",

            #[cfg(target_os = "linux")]
            EBADE           => "Invalid exchange",

            #[cfg(target_os = "linux")]
            EBADR           => "Invalid request descriptor",

            #[cfg(target_os = "linux")]
            EXFULL          => "Exchange full",

            #[cfg(target_os = "linux")]
            ENOANO          => "No anode",

            #[cfg(target_os = "linux")]
            EBADRQC         => "Invalid request code",

            #[cfg(target_os = "linux")]
            EBADSLT         => "Invalid slot",

            #[cfg(target_os = "linux")]
            EBFONT          => "Bad font file format",

            #[cfg(target_os = "linux")]
            ENOSTR          => "Device not a stream",

            #[cfg(target_os = "linux")]
            ENODATA         => "No data available",

            #[cfg(target_os = "linux")]
            ETIME           => "Timer expired",

            #[cfg(target_os = "linux")]
            ENOSR           => "Out of streams resources",

            #[cfg(target_os = "linux")]
            ENONET          => "Machine is not on the network",

            #[cfg(target_os = "linux")]
            ENOPKG          => "Package not installed",

            #[cfg(target_os = "linux")]
            EREMOTE         => "Object is remote",

            #[cfg(target_os = "linux")]
            ENOLINK         => "Link has been severed",

            #[cfg(target_os = "linux")]
            EADV            => "Advertise error",

            #[cfg(target_os = "linux")]
            ESRMNT          => "Srmount error",

            #[cfg(target_os = "linux")]
            ECOMM           => "Communication error on send",

            #[cfg(target_os = "linux")]
            EPROTO          => "Protocol error",

            #[cfg(target_os = "linux")]
            EMULTIHOP       => "Multihop attempted",

            #[cfg(target_os = "linux")]
            EDOTDOT         => "RFS specific error",

            #[cfg(target_os = "linux")]
            EBADMSG         => "Not a data message",

            #[cfg(target_os = "linux")]
            EOVERFLOW       => "Value too large for defined data type",

            #[cfg(target_os = "linux")]
            ENOTUNIQ        => "Name not unique on network",

            #[cfg(target_os = "linux")]
            EBADFD          => "File descriptor in bad state",

            #[cfg(target_os = "linux")]
            EREMCHG         => "Remote address changed",

            #[cfg(target_os = "linux")]
            ELIBACC         => "Can not acces a needed shared library",

            #[cfg(target_os = "linux")]
            ELIBBAD         => "Accessing a corrupted shared library",

            #[cfg(target_os = "linux")]
            ELIBSCN         => ".lib section in a.out corrupted",

            #[cfg(target_os = "linux")]
            ELIBMAX         => "Attempting to link in too many shared libraries",

            #[cfg(target_os = "linux")]
            ELIBEXEC        => "Cannot exec a shared library directly",

            #[cfg(target_os = "linux")]
            EILSEQ          => "Illegal byte sequence",

            #[cfg(target_os = "linux")]
            ERESTART        => "Interrupted system call should be restarted",

            #[cfg(target_os = "linux")]
            ESTRPIPE        => "Streams pipe error",

            #[cfg(target_os = "linux")]
            EUSERS          => "Too many users",

            #[cfg(target_os = "linux")]
            EOPNOTSUPP      => "Operation not supported on transport endpoint",

            #[cfg(target_os = "linux")]
            ESTALE          => "Stale file handle",

            #[cfg(target_os = "linux")]
            EUCLEAN         => "Structure needs cleaning",

            #[cfg(target_os = "linux")]
            ENOTNAM         => "Not a XENIX named type file",

            #[cfg(target_os = "linux")]
            ENAVAIL         => "No XENIX semaphores available",

            #[cfg(target_os = "linux")]
            EISNAM          => "Is a named type file",

            #[cfg(target_os = "linux")]
            EREMOTEIO       => "Remote I/O error",

            #[cfg(target_os = "linux")]
            EDQUOT          => "Quota exceeded",

            #[cfg(target_os = "linux")]
            ENOMEDIUM       => "No medium found",

            #[cfg(target_os = "linux")]
            EMEDIUMTYPE     => "Wrong medium type",

            #[cfg(target_os = "linux")]
            ECANCELED       => "Operation canceled",

            #[cfg(target_os = "linux")]
            ENOKEY          => "Required key not available",

            #[cfg(target_os = "linux")]
            EKEYEXPIRED     => "Key has expired",

            #[cfg(target_os = "linux")]
            EKEYREVOKED     => "Key has been revoked",

            #[cfg(target_os = "linux")]
            EKEYREJECTED    => "Key was rejected by service",

            #[cfg(target_os = "linux")]
            EOWNERDEAD      => "Owner died",

            #[cfg(target_os = "linux")]
            ENOTRECOVERABLE => "State not recoverable",

            #[cfg(target_os = "linux")]
            ERFKILL         => "Operation not possible due to RF-kill",

            #[cfg(target_os = "linux")]
            EHWPOISON       => "Memory page has hardware error",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ENOTSUP         => "Operation not supported",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EPROCLIM        => "Too many processes",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EUSERS          => "Too many users",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EDQUOT          => "Disc quota exceeded",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ESTALE          => "Stale NFS file handle",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EREMOTE         => "Stale NFS file handle",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EBADRPC         => "RPC struct is bad",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ERPCMISMATCH    => "RPC version wrong",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EPROGUNAVAIL    => "RPC prog. not avail",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EPROGMISMATCH   => "Program version wrong",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EPROCUNAVAIL    => "Bad procedure for program",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EFTYPE          => "Inappropriate file type or format",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EAUTH           => "Authentication error",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ENEEDAUTH       => "Need authenticator",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EPWROFF         => "Device power is off",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EDEVERR         => "Device error, e.g. paper out",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EOVERFLOW       => "Value too large to be stored in data type",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EBADEXEC        => "Bad executable",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EBADARCH        => "Bad CPU type in executable",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ESHLIBVERS      => "Shared library version mismatch",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EBADMACHO       => "Malformed Macho file",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ECANCELED       => "Operation canceled",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EILSEQ          => "Illegal byte sequence",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ENOATTR         => "Attribute not found",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EBADMSG         => "Bad message",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EMULTIHOP       => "Reserved",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ENODATA         => "No message available on STREAM",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ENOLINK         => "Reserved",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ENOSR           => "No STREAM resources",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ENOSTR          => "Not a STREAM",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EPROTO          => "Protocol error",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ETIME           => "STREAM ioctl timeout",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EOPNOTSUPP      => "Operation not supported on socket",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ENOPOLICY       => "No such policy registered",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ENOTRECOVERABLE => "State not recoverable",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EOWNERDEAD      => "Previous owner died",

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            EQFULL          => "Interface output queue is full",
        }
    }
}

impl fmt::Show for SysError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?} - {:?}", self.kind, self.desc())
    }
}

#[inline]
pub fn from_ffi(res: c_int) -> SysResult<()> {
    if res != 0 {
        return Err(SysError::last());
    }

    Ok(())
}


#[cfg(target_os = "linux")]
mod consts {
    #[derive(Show, Clone, PartialEq, FromPrimitive, Copy)]
    pub enum Errno {
        UnknownErrno    = 0,
        EPERM           = 1,
        ENOENT          = 2,
        ESRCH           = 3,
        EINTR           = 4,
        EIO             = 5,
        ENXIO           = 6,
        E2BIG           = 7,
        ENOEXEC         = 8,
        EBADF           = 9,
        ECHILD          = 10,
        EAGAIN          = 11,
        ENOMEM          = 12,
        EACCES          = 13,
        EFAULT          = 14,
        ENOTBLK         = 15,
        EBUSY           = 16,
        EEXIST          = 17,
        EXDEV           = 18,
        ENODEV          = 19,
        ENOTDIR         = 20,
        EISDIR          = 21,
        EINVAL          = 22,
        ENFILE          = 23,
        EMFILE          = 24,
        ENOTTY          = 25,
        ETXTBSY         = 26,
        EFBIG           = 27,
        ENOSPC          = 28,
        ESPIPE          = 29,
        EROFS           = 30,
        EMLINK          = 31,
        EPIPE           = 32,
        EDOM            = 33,
        ERANGE          = 34,
        EDEADLK         = 35,
        ENAMETOOLONG    = 36,
        ENOLCK          = 37,
        ENOSYS          = 38,
        ENOTEMPTY       = 39,
        ELOOP           = 40,
        ENOMSG          = 42,
        EIDRM           = 43,
        ECHRNG          = 44,
        EL2NSYNC        = 45,
        EL3HLT          = 46,
        EL3RST          = 47,
        ELNRNG          = 48,
        EUNATCH         = 49,
        ENOCSI          = 50,
        EL2HLT          = 51,
        EBADE           = 52,
        EBADR           = 53,
        EXFULL          = 54,
        ENOANO          = 55,
        EBADRQC         = 56,
        EBADSLT         = 57,
        EBFONT          = 59,
        ENOSTR          = 60,
        ENODATA         = 61,
        ETIME           = 62,
        ENOSR           = 63,
        ENONET          = 64,
        ENOPKG          = 65,
        EREMOTE         = 66,
        ENOLINK         = 67,
        EADV            = 68,
        ESRMNT          = 69,
        ECOMM           = 70,
        EPROTO          = 71,
        EMULTIHOP       = 72,
        EDOTDOT         = 73,
        EBADMSG         = 74,
        EOVERFLOW       = 75,
        ENOTUNIQ        = 76,
        EBADFD          = 77,
        EREMCHG         = 78,
        ELIBACC         = 79,
        ELIBBAD         = 80,
        ELIBSCN         = 81,
        ELIBMAX         = 82,
        ELIBEXEC        = 83,
        EILSEQ          = 84,
        ERESTART        = 85,
        ESTRPIPE        = 86,
        EUSERS          = 87,
        ENOTSOCK        = 88,
        EDESTADDRREQ    = 89,
        EMSGSIZE        = 90,
        EPROTOTYPE      = 91,
        ENOPROTOOPT     = 92,
        EPROTONOSUPPORT = 93,
        ESOCKTNOSUPPORT = 94,
        EOPNOTSUPP      = 95,
        EPFNOSUPPORT    = 96,
        EAFNOSUPPORT    = 97,
        EADDRINUSE      = 98,
        EADDRNOTAVAIL   = 99,
        ENETDOWN        = 100,
        ENETUNREACH     = 101,
        ENETRESET       = 102,
        ECONNABORTED    = 103,
        ECONNRESET      = 104,
        ENOBUFS         = 105,
        EISCONN         = 106,
        ENOTCONN        = 107,
        ESHUTDOWN       = 108,
        ETOOMANYREFS    = 109,
        ETIMEDOUT       = 110,
        ECONNREFUSED    = 111,
        EHOSTDOWN       = 112,
        EHOSTUNREACH    = 113,
        EALREADY        = 114,
        EINPROGRESS     = 115,
        ESTALE          = 116,
        EUCLEAN         = 117,
        ENOTNAM         = 118,
        ENAVAIL         = 119,
        EISNAM          = 120,
        EREMOTEIO       = 121,
        EDQUOT          = 122,
        ENOMEDIUM       = 123,
        EMEDIUMTYPE     = 124,
        ECANCELED       = 125,
        ENOKEY          = 126,
        EKEYEXPIRED     = 127,
        EKEYREVOKED     = 128,
        EKEYREJECTED    = 129,
        EOWNERDEAD      = 130,
        ENOTRECOVERABLE = 131,
        ERFKILL         = 132,
        EHWPOISON       = 133,
    }

    pub const EWOULDBLOCK: Errno = Errno::EAGAIN;
    pub const EDEADLOCK:   Errno = Errno::EDEADLK;
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod consts {
    #[derive(Copy, Show, Clone, PartialEq, FromPrimitive)]
    pub enum Errno {
        UnknownErrno    = 0,
        EPERM           = 1,
        ENOENT          = 2,
        ESRCH           = 3,
        EINTR           = 4,
        EIO             = 5,
        ENXIO           = 6,
        E2BIG           = 7,
        ENOEXEC         = 8,
        EBADF           = 9,
        ECHILD          = 10,
        EDEADLK         = 11,
        ENOMEM          = 12,
        EACCES          = 13,
        EFAULT          = 14,
        ENOTBLK         = 15,
        EBUSY           = 16,
        EEXIST          = 17,
        EXDEV           = 18,
        ENODEV          = 19,
        ENOTDIR         = 20,
        EISDIR          = 21,
        EINVAL          = 22,
        ENFILE          = 23,
        EMFILE          = 24,
        ENOTTY          = 25,
        ETXTBSY         = 26,
        EFBIG           = 27,
        ENOSPC          = 28,
        ESPIPE          = 29,
        EROFS           = 30,
        EMLINK          = 31,
        EPIPE           = 32,
        EDOM            = 33,
        ERANGE          = 34,
        EAGAIN          = 35,
        EINPROGRESS     = 36,
        EALREADY        = 37,
        ENOTSOCK        = 38,
        EDESTADDRREQ    = 39,
        EMSGSIZE        = 40,
        EPROTOTYPE      = 41,
        ENOPROTOOPT     = 42,
        EPROTONOSUPPORT = 43,
        ESOCKTNOSUPPORT = 44,
        ENOTSUP         = 45,
        EPFNOSUPPORT    = 46,
        EAFNOSUPPORT    = 47,
        EADDRINUSE      = 48,
        EADDRNOTAVAIL   = 49,
        ENETDOWN        = 50,
        ENETUNREACH     = 51,
        ENETRESET       = 52,
        ECONNABORTED    = 53,
        ECONNRESET      = 54,
        ENOBUFS         = 55,
        EISCONN         = 56,
        ENOTCONN        = 57,
        ESHUTDOWN       = 58,
        ETOOMANYREFS    = 59,
        ETIMEDOUT       = 60,
        ECONNREFUSED    = 61,
        ELOOP           = 62,
        ENAMETOOLONG    = 63,
        EHOSTDOWN       = 64,
        EHOSTUNREACH    = 65,
        ENOTEMPTY       = 66,
        EPROCLIM        = 67,
        EUSERS          = 68,
        EDQUOT          = 69,
        ESTALE          = 70,
        EREMOTE         = 71,
        EBADRPC         = 72,
        ERPCMISMATCH    = 73,
        EPROGUNAVAIL    = 74,
        EPROGMISMATCH   = 75,
        EPROCUNAVAIL    = 76,
        ENOLCK          = 77,
        ENOSYS          = 78,
        EFTYPE          = 79,
        EAUTH           = 80,
        ENEEDAUTH       = 81,
        EPWROFF         = 82,
        EDEVERR         = 83,
        EOVERFLOW       = 84,
        EBADEXEC        = 85,
        EBADARCH        = 86,
        ESHLIBVERS      = 87,
        EBADMACHO       = 88,
        ECANCELED       = 89,
        EIDRM           = 90,
        ENOMSG          = 91,
        EILSEQ          = 92,
        ENOATTR         = 93,
        EBADMSG         = 94,
        EMULTIHOP       = 95,
        ENODATA         = 96,
        ENOLINK         = 97,
        ENOSR           = 98,
        ENOSTR          = 99,
        EPROTO          = 100,
        ETIME           = 101,
        EOPNOTSUPP      = 102,
        ENOPOLICY       = 103,
        ENOTRECOVERABLE = 104,
        EOWNERDEAD      = 105,
        EQFULL          = 106,
    }

    pub const ELAST: Errno       = Errno::EQFULL;
    pub const EWOULDBLOCK: Errno = Errno::EAGAIN;
    pub const EDEADLOCK:   Errno = Errno::EDEADLK;

    pub const EL2NSYNC: Errno = Errno::UnknownErrno;
}
