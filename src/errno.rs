use libc::c_int;
use std::{fmt, io, error};
use {Error, Result};

pub use self::consts::*;
pub use self::consts::Errno::*;

#[cfg(any(target_os = "macos",
          target_os = "ios",
          target_os = "freebsd"))]
unsafe fn errno_location() -> *mut c_int {
    extern { fn __error() -> *mut c_int; }
    __error()
}

#[cfg(target_os = "bitrig")]
fn errno_location() -> *mut c_int {
    extern {
        fn __errno() -> *mut c_int;
    }
    unsafe {
        __errno()
    }
}

#[cfg(target_os = "dragonfly")]
unsafe fn errno_location() -> *mut c_int {
    extern { fn __dfly_error() -> *mut c_int; }
    __dfly_error()
}

#[cfg(any(target_os = "openbsd", target_os = "netbsd"))]
unsafe fn errno_location() -> *mut c_int {
    extern { fn __errno() -> *mut c_int; }
    __errno()
}

#[cfg(any(target_os = "linux", target_os = "android"))]
unsafe fn errno_location() -> *mut c_int {
    extern { fn __errno_location() -> *mut c_int; }
    __errno_location()
}

/// Sets the platform-specific errno to no-error
unsafe fn clear() -> () {
    *errno_location() = 0;
}

/// Returns the platform-specific value of errno
pub fn errno() -> i32 {
    unsafe {
        (*errno_location()) as i32
    }
}

impl Errno {
    pub fn last() -> Self {
        last()
    }

    pub fn desc(self) -> &'static str {
        desc(self)
    }

    pub fn from_i32(err: i32) -> Errno {
        from_i32(err)
    }

    pub unsafe fn clear() -> () {
        clear()
    }

    /// Returns `Ok(value)` if it does not contain the sentinel value. This
    /// should not be used when `-1` is not the errno sentinel value.
    pub fn result<S: ErrnoSentinel + PartialEq<S>>(value: S) -> Result<S> {
        if value == S::sentinel() {
            Err(Error::Sys(Self::last()))
        } else {
            Ok(value)
        }
    }
}

/// The sentinel value indicates that a function failed and more detailed
/// information about the error can be found in `errno`
pub trait ErrnoSentinel: Sized {
    fn sentinel() -> Self;
}

impl ErrnoSentinel for isize {
    fn sentinel() -> Self { -1 }
}

impl ErrnoSentinel for i32 {
    fn sentinel() -> Self { -1 }
}

impl ErrnoSentinel for i64 {
    fn sentinel() -> Self { -1 }
}

impl error::Error for Errno {
    fn description(&self) -> &str {
        self.desc()
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {}", self, self.desc())
    }
}

impl From<Errno> for io::Error {
    fn from(err: Errno) -> Self {
        io::Error::from_raw_os_error(err as i32)
    }
}

fn last() -> Errno {
    Errno::from_i32(errno())
}

fn desc(errno: Errno) -> &'static str {
    match errno {
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

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ECHRNG          => "Channel number out of range",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EL2NSYNC        => "Level 2 not synchronized",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EL3HLT          => "Level 3 halted",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EL3RST          => "Level 3 reset",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ELNRNG          => "Link number out of range",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EUNATCH         => "Protocol driver not attached",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENOCSI          => "No CSI structure available",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EL2HLT          => "Level 2 halted",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EBADE           => "Invalid exchange",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EBADR           => "Invalid request descriptor",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EXFULL          => "Exchange full",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENOANO          => "No anode",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EBADRQC         => "Invalid request code",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EBADSLT         => "Invalid slot",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EBFONT          => "Bad font file format",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENOSTR          => "Device not a stream",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENODATA         => "No data available",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ETIME           => "Timer expired",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENOSR           => "Out of streams resources",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENONET          => "Machine is not on the network",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENOPKG          => "Package not installed",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EREMOTE         => "Object is remote",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENOLINK         => "Link has been severed",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EADV            => "Advertise error",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ESRMNT          => "Srmount error",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ECOMM           => "Communication error on send",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EPROTO          => "Protocol error",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EMULTIHOP       => "Multihop attempted",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EDOTDOT         => "RFS specific error",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EBADMSG         => "Not a data message",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EOVERFLOW       => "Value too large for defined data type",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENOTUNIQ        => "Name not unique on network",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EBADFD          => "File descriptor in bad state",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EREMCHG         => "Remote address changed",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ELIBACC         => "Can not acces a needed shared library",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ELIBBAD         => "Accessing a corrupted shared library",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ELIBSCN         => ".lib section in a.out corrupted",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ELIBMAX         => "Attempting to link in too many shared libraries",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ELIBEXEC        => "Cannot exec a shared library directly",

        #[cfg(any(target_os = "linux", target_os = "android", target_os = "openbsd"))]
        EILSEQ          => "Illegal byte sequence",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ERESTART        => "Interrupted system call should be restarted",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ESTRPIPE        => "Streams pipe error",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EUSERS          => "Too many users",

        #[cfg(any(target_os = "linux", target_os = "android", target_os = "netbsd"))]
        EOPNOTSUPP      => "Operation not supported on transport endpoint",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ESTALE          => "Stale file handle",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EUCLEAN         => "Structure needs cleaning",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENOTNAM         => "Not a XENIX named type file",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENAVAIL         => "No XENIX semaphores available",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EISNAM          => "Is a named type file",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EREMOTEIO       => "Remote I/O error",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EDQUOT          => "Quota exceeded",

        #[cfg(any(target_os = "linux", target_os = "android", target_os = "openbsd", target_os = "dragonfly"))]
        ENOMEDIUM       => "No medium found",

        #[cfg(any(target_os = "linux", target_os = "android", target_os = "openbsd"))]
        EMEDIUMTYPE     => "Wrong medium type",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ECANCELED       => "Operation canceled",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENOKEY          => "Required key not available",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EKEYEXPIRED     => "Key has expired",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EKEYREVOKED     => "Key has been revoked",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EKEYREJECTED    => "Key was rejected by service",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        EOWNERDEAD      => "Owner died",

        #[cfg(any(target_os = "linux", target_os = "android"))]
        ENOTRECOVERABLE => "State not recoverable",

        #[cfg(target_os = "linux")]
        ERFKILL         => "Operation not possible due to RF-kill",

        #[cfg(target_os = "linux")]
        EHWPOISON       => "Memory page has hardware error",

        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        EDOOFUS         => "Programming error",

        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        EMULTIHOP       => "Multihop attempted",

        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        ENOLINK         => "Link has been severed",

        #[cfg(target_os = "freebsd")]
        ENOTCAPABLE     => "Capabilities insufficient",

        #[cfg(target_os = "freebsd")]
        ECAPMODE        => "Not permitted in capability mode",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        ENEEDAUTH       => "Need authenticator",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        EOVERFLOW       => "Value too large to be stored in data type",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "netbsd"))]
        EILSEQ          => "Illegal byte sequence",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        ENOATTR         => "Attribute not found",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "netbsd"))]
        EBADMSG         => "Bad message",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "netbsd"))]
        EPROTO          => "Protocol error",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "ios"))]
        ENOTRECOVERABLE => "State not recoverable",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "ios"))]
        EOWNERDEAD      => "Previous owner died",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        ENOTSUP         => "Operation not supported",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        EPROCLIM        => "Too many processes",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        EUSERS          => "Too many users",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        EDQUOT          => "Disc quota exceeded",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        ESTALE          => "Stale NFS file handle",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        EREMOTE         => "Too many levels of remote in path",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        EBADRPC         => "RPC struct is bad",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        ERPCMISMATCH    => "RPC version wrong",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        EPROGUNAVAIL    => "RPC prog. not avail",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        EPROGMISMATCH   => "Program version wrong",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        EPROCUNAVAIL    => "Bad procedure for program",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        EFTYPE          => "Inappropriate file type or format",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        EAUTH           => "Authentication error",

        #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "ios", target_os = "openbsd", target_os = "netbsd"))]
        ECANCELED       => "Operation canceled",

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EPWROFF         => "Device power is off",

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EDEVERR         => "Device error, e.g. paper out",

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EBADEXEC        => "Bad executable",

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EBADARCH        => "Bad CPU type in executable",

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        ESHLIBVERS      => "Shared library version mismatch",

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EBADMACHO       => "Malformed Macho file",

        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "netbsd"))]
        EMULTIHOP       => "Reserved",

        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "netbsd"))]
        ENODATA         => "No message available on STREAM",

        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "netbsd"))]
        ENOLINK         => "Reserved",

        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "netbsd"))]
        ENOSR           => "No STREAM resources",

        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "netbsd"))]
        ENOSTR          => "Not a STREAM",

        #[cfg(any(target_os = "macos", target_os = "ios", target_os = "netbsd"))]
        ETIME           => "STREAM ioctl timeout",

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EOPNOTSUPP      => "Operation not supported on socket",

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        ENOPOLICY       => "No such policy registered",

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        EQFULL          => "Interface output queue is full",

        #[cfg(target_os = "openbsd")]
        EOPNOTSUPP      => "Operation not supported",

        #[cfg(target_os = "openbsd")]
        EIPSEC          => "IPsec processing failure",

        #[cfg(target_os = "dragonfly")]
        EUNUSED94 | EUNUSED95 | EUNUSED96 | EUNUSED97 | EUNUSED98 => "Unused",
 
        #[cfg(target_os = "dragonfly")]
        EASYNC          => "Async",
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod consts {
    #[derive(Debug, Clone, PartialEq, Copy)]
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

        #[cfg(not(target_os = "android"))]
        ERFKILL         = 132,
        #[cfg(not(target_os = "android"))]
        EHWPOISON       = 133,
    }

    pub const EWOULDBLOCK: Errno = Errno::EAGAIN;
    pub const EDEADLOCK:   Errno = Errno::EDEADLK;

    pub fn from_i32(e: i32) -> Errno {
        use self::Errno::*;

        match e {
            0   => UnknownErrno,
            1   => EPERM,
            2   => ENOENT,
            3   => ESRCH,
            4   => EINTR,
            5   => EIO,
            6   => ENXIO,
            7   => E2BIG,
            8   => ENOEXEC,
            9   => EBADF,
            10  => ECHILD,
            11  => EAGAIN,
            12  => ENOMEM,
            13  => EACCES,
            14  => EFAULT,
            15  => ENOTBLK,
            16  => EBUSY,
            17  => EEXIST,
            18  => EXDEV,
            19  => ENODEV,
            20  => ENOTDIR,
            21  => EISDIR,
            22  => EINVAL,
            23  => ENFILE,
            24  => EMFILE,
            25  => ENOTTY,
            26  => ETXTBSY,
            27  => EFBIG,
            28  => ENOSPC,
            29  => ESPIPE,
            30  => EROFS,
            31  => EMLINK,
            32  => EPIPE,
            33  => EDOM,
            34  => ERANGE,
            35  => EDEADLK,
            36  => ENAMETOOLONG,
            37  => ENOLCK,
            38  => ENOSYS,
            39  => ENOTEMPTY,
            40  => ELOOP,
            42  => ENOMSG,
            43  => EIDRM,
            44  => ECHRNG,
            45  => EL2NSYNC,
            46  => EL3HLT,
            47  => EL3RST,
            48  => ELNRNG,
            49  => EUNATCH,
            50  => ENOCSI,
            51  => EL2HLT,
            52  => EBADE,
            53  => EBADR,
            54  => EXFULL,
            55  => ENOANO,
            56  => EBADRQC,
            57  => EBADSLT,
            59  => EBFONT,
            60  => ENOSTR,
            61  => ENODATA,
            62  => ETIME,
            63  => ENOSR,
            64  => ENONET,
            65  => ENOPKG,
            66  => EREMOTE,
            67  => ENOLINK,
            68  => EADV,
            69  => ESRMNT,
            70  => ECOMM,
            71  => EPROTO,
            72  => EMULTIHOP,
            73  => EDOTDOT,
            74  => EBADMSG,
            75  => EOVERFLOW,
            76  => ENOTUNIQ,
            77  => EBADFD,
            78  => EREMCHG,
            79  => ELIBACC,
            80  => ELIBBAD,
            81  => ELIBSCN,
            82  => ELIBMAX,
            83  => ELIBEXEC,
            84  => EILSEQ,
            85  => ERESTART,
            86  => ESTRPIPE,
            87  => EUSERS,
            88  => ENOTSOCK,
            89  => EDESTADDRREQ,
            90  => EMSGSIZE,
            91  => EPROTOTYPE,
            92  => ENOPROTOOPT,
            93  => EPROTONOSUPPORT,
            94  => ESOCKTNOSUPPORT,
            95  => EOPNOTSUPP,
            96  => EPFNOSUPPORT,
            97  => EAFNOSUPPORT,
            98  => EADDRINUSE,
            99  => EADDRNOTAVAIL,
            100 => ENETDOWN,
            101 => ENETUNREACH,
            102 => ENETRESET,
            103 => ECONNABORTED,
            104 => ECONNRESET,
            105 => ENOBUFS,
            106 => EISCONN,
            107 => ENOTCONN,
            108 => ESHUTDOWN,
            109 => ETOOMANYREFS,
            110 => ETIMEDOUT,
            111 => ECONNREFUSED,
            112 => EHOSTDOWN,
            113 => EHOSTUNREACH,
            114 => EALREADY,
            115 => EINPROGRESS,
            116 => ESTALE,
            117 => EUCLEAN,
            118 => ENOTNAM,
            119 => ENAVAIL,
            120 => EISNAM,
            121 => EREMOTEIO,
            122 => EDQUOT,
            123 => ENOMEDIUM,
            124 => EMEDIUMTYPE,
            125 => ECANCELED,
            126 => ENOKEY,
            127 => EKEYEXPIRED,
            128 => EKEYREVOKED,
            129 => EKEYREJECTED,
            130 => EOWNERDEAD,
            131 => ENOTRECOVERABLE,

            #[cfg(not(target_os = "android"))]
            132 => ERFKILL,
            #[cfg(not(target_os = "android"))]
            133 => EHWPOISON,
            _   => UnknownErrno,
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod consts {
    #[derive(Copy, Debug, Clone, PartialEq)]
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

    pub fn from_i32(e: i32) -> Errno {
        use self::Errno::*;

        match e {
            0   => UnknownErrno,
            1   => EPERM,
            2   => ENOENT,
            3   => ESRCH,
            4   => EINTR,
            5   => EIO,
            6   => ENXIO,
            7   => E2BIG,
            8   => ENOEXEC,
            9   => EBADF,
            10  => ECHILD,
            11  => EDEADLK,
            12  => ENOMEM,
            13  => EACCES,
            14  => EFAULT,
            15  => ENOTBLK,
            16  => EBUSY,
            17  => EEXIST,
            18  => EXDEV,
            19  => ENODEV,
            20  => ENOTDIR,
            21  => EISDIR,
            22  => EINVAL,
            23  => ENFILE,
            24  => EMFILE,
            25  => ENOTTY,
            26  => ETXTBSY,
            27  => EFBIG,
            28  => ENOSPC,
            29  => ESPIPE,
            30  => EROFS,
            31  => EMLINK,
            32  => EPIPE,
            33  => EDOM,
            34  => ERANGE,
            35  => EAGAIN,
            36  => EINPROGRESS,
            37  => EALREADY,
            38  => ENOTSOCK,
            39  => EDESTADDRREQ,
            40  => EMSGSIZE,
            41  => EPROTOTYPE,
            42  => ENOPROTOOPT,
            43  => EPROTONOSUPPORT,
            44  => ESOCKTNOSUPPORT,
            45  => ENOTSUP,
            46  => EPFNOSUPPORT,
            47  => EAFNOSUPPORT,
            48  => EADDRINUSE,
            49  => EADDRNOTAVAIL,
            50  => ENETDOWN,
            51  => ENETUNREACH,
            52  => ENETRESET,
            53  => ECONNABORTED,
            54  => ECONNRESET,
            55  => ENOBUFS,
            56  => EISCONN,
            57  => ENOTCONN,
            58  => ESHUTDOWN,
            59  => ETOOMANYREFS,
            60  => ETIMEDOUT,
            61  => ECONNREFUSED,
            62  => ELOOP,
            63  => ENAMETOOLONG,
            64  => EHOSTDOWN,
            65  => EHOSTUNREACH,
            66  => ENOTEMPTY,
            67  => EPROCLIM,
            68  => EUSERS,
            69  => EDQUOT,
            70  => ESTALE,
            71  => EREMOTE,
            72  => EBADRPC,
            73  => ERPCMISMATCH,
            74  => EPROGUNAVAIL,
            75  => EPROGMISMATCH,
            76  => EPROCUNAVAIL,
            77  => ENOLCK,
            78  => ENOSYS,
            79  => EFTYPE,
            80  => EAUTH,
            81  => ENEEDAUTH,
            82  => EPWROFF,
            83  => EDEVERR,
            84  => EOVERFLOW,
            85  => EBADEXEC,
            86  => EBADARCH,
            87  => ESHLIBVERS,
            88  => EBADMACHO,
            89  => ECANCELED,
            90  => EIDRM,
            91  => ENOMSG,
            92  => EILSEQ,
            93  => ENOATTR,
            94  => EBADMSG,
            95  => EMULTIHOP,
            96  => ENODATA,
            97  => ENOLINK,
            98  => ENOSR,
            99  => ENOSTR,
            100 => EPROTO,
            101 => ETIME,
            102 => EOPNOTSUPP,
            103 => ENOPOLICY,
            104 => ENOTRECOVERABLE,
            105 => EOWNERDEAD,
            106 => EQFULL,
            _   => UnknownErrno,
        }
    }
}

#[cfg(target_os = "freebsd")]
mod consts {
    #[derive(Copy, Debug, Clone, PartialEq)]
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
        EIDRM           = 82,
        ENOMSG          = 83,
        EOVERFLOW       = 84,
        ECANCELED       = 85,
        EILSEQ          = 86,
        ENOATTR         = 87,
        EDOOFUS         = 88,
        EBADMSG         = 89,
        EMULTIHOP       = 90,
        ENOLINK         = 91,
        EPROTO          = 92,
        ENOTCAPABLE     = 93,
        ECAPMODE        = 94,
        ENOTRECOVERABLE = 95,
        EOWNERDEAD      = 96

    }

    pub const ELAST: Errno       = Errno::EOWNERDEAD;
    pub const EWOULDBLOCK: Errno = Errno::EAGAIN;
    pub const EDEADLOCK:   Errno = Errno::EDEADLK;

    pub const EL2NSYNC: Errno = Errno::UnknownErrno;

    pub fn from_i32(e: i32) -> Errno {
        use self::Errno::*;

        match e {
            0   => UnknownErrno,
            1   => EPERM,
            2   => ENOENT,
            3   => ESRCH,
            4   => EINTR,
            5   => EIO,
            6   => ENXIO,
            7   => E2BIG,
            8   => ENOEXEC,
            9   => EBADF,
            10  => ECHILD,
            11  => EDEADLK,
            12  => ENOMEM,
            13  => EACCES,
            14  => EFAULT,
            15  => ENOTBLK,
            16  => EBUSY,
            17  => EEXIST,
            18  => EXDEV,
            19  => ENODEV,
            20  => ENOTDIR,
            21  => EISDIR,
            22  => EINVAL,
            23  => ENFILE,
            24  => EMFILE,
            25  => ENOTTY,
            26  => ETXTBSY,
            27  => EFBIG,
            28  => ENOSPC,
            29  => ESPIPE,
            30  => EROFS,
            31  => EMLINK,
            32  => EPIPE,
            33  => EDOM,
            34  => ERANGE,
            35  => EAGAIN,
            36  => EINPROGRESS,
            37  => EALREADY,
            38  => ENOTSOCK,
            39  => EDESTADDRREQ,
            40  => EMSGSIZE,
            41  => EPROTOTYPE,
            42  => ENOPROTOOPT,
            43  => EPROTONOSUPPORT,
            44  => ESOCKTNOSUPPORT,
            45  => ENOTSUP,
            46  => EPFNOSUPPORT,
            47  => EAFNOSUPPORT,
            48  => EADDRINUSE,
            49  => EADDRNOTAVAIL,
            50  => ENETDOWN,
            51  => ENETUNREACH,
            52  => ENETRESET,
            53  => ECONNABORTED,
            54  => ECONNRESET,
            55  => ENOBUFS,
            56  => EISCONN,
            57  => ENOTCONN,
            58  => ESHUTDOWN,
            59  => ETOOMANYREFS,
            60  => ETIMEDOUT,
            61  => ECONNREFUSED,
            62  => ELOOP,
            63  => ENAMETOOLONG,
            64  => EHOSTDOWN,
            65  => EHOSTUNREACH,
            66  => ENOTEMPTY,
            67  => EPROCLIM,
            68  => EUSERS,
            69  => EDQUOT,
            70  => ESTALE,
            71  => EREMOTE,
            72  => EBADRPC,
            73  => ERPCMISMATCH,
            74  => EPROGUNAVAIL,
            75  => EPROGMISMATCH,
            76  => EPROCUNAVAIL,
            77  => ENOLCK,
            78  => ENOSYS,
            79  => EFTYPE,
            80  => EAUTH,
            81  => ENEEDAUTH,
            82  => EIDRM,
            83  => ENOMSG,
            84  => EOVERFLOW,
            85  => ECANCELED,
            86  => EILSEQ,
            87  => ENOATTR,
            88  => EDOOFUS,
            89  => EBADMSG,
            90  => EMULTIHOP,
            91  => ENOLINK,
            92  => EPROTO,
            93  => ENOTCAPABLE,
            94  => ECAPMODE,
            95  => ENOTRECOVERABLE,
            96  => EOWNERDEAD,
            _   => UnknownErrno,
        }
    }
}


#[cfg(target_os = "dragonfly")]
mod consts {
    #[derive(Copy, Debug, Clone, PartialEq)]
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
        EIDRM           = 82,
        ENOMSG          = 83,
        EOVERFLOW       = 84,
        ECANCELED       = 85,
        EILSEQ          = 86,
        ENOATTR         = 87,
        EDOOFUS         = 88,
        EBADMSG         = 89,
        EMULTIHOP       = 90,
        ENOLINK         = 91,
        EPROTO          = 92,

        ENOMEDIUM       = 93,
        EUNUSED94       = 94,
        EUNUSED95       = 95,
        EUNUSED96       = 96,
        EUNUSED97       = 97,
        EUNUSED98       = 98,
        EASYNC          = 99,
    }

    pub const ELAST: Errno       = Errno::EASYNC;
    pub const EWOULDBLOCK: Errno = Errno::EAGAIN;
    pub const EDEADLOCK:   Errno = Errno::EDEADLK;
    pub const EOPNOTSUPP:  Errno = Errno::ENOTSUP;

    pub const EL2NSYNC: Errno = Errno::UnknownErrno;

    pub fn from_i32(e: i32) -> Errno {
        use self::Errno::*;

        match e {
            0   => UnknownErrno,
            1   => EPERM,
            2   => ENOENT,
            3   => ESRCH,
            4   => EINTR,
            5   => EIO,
            6   => ENXIO,
            7   => E2BIG,
            8   => ENOEXEC,
            9   => EBADF,
            10  => ECHILD,
            11  => EDEADLK,
            12  => ENOMEM,
            13  => EACCES,
            14  => EFAULT,
            15  => ENOTBLK,
            16  => EBUSY,
            17  => EEXIST,
            18  => EXDEV,
            19  => ENODEV,
            20  => ENOTDIR,
            21  => EISDIR,
            22  => EINVAL,
            23  => ENFILE,
            24  => EMFILE,
            25  => ENOTTY,
            26  => ETXTBSY,
            27  => EFBIG,
            28  => ENOSPC,
            29  => ESPIPE,
            30  => EROFS,
            31  => EMLINK,
            32  => EPIPE,
            33  => EDOM,
            34  => ERANGE,
            35  => EAGAIN,
            36  => EINPROGRESS,
            37  => EALREADY,
            38  => ENOTSOCK,
            39  => EDESTADDRREQ,
            40  => EMSGSIZE,
            41  => EPROTOTYPE,
            42  => ENOPROTOOPT,
            43  => EPROTONOSUPPORT,
            44  => ESOCKTNOSUPPORT,
            45  => ENOTSUP,
            46  => EPFNOSUPPORT,
            47  => EAFNOSUPPORT,
            48  => EADDRINUSE,
            49  => EADDRNOTAVAIL,
            50  => ENETDOWN,
            51  => ENETUNREACH,
            52  => ENETRESET,
            53  => ECONNABORTED,
            54  => ECONNRESET,
            55  => ENOBUFS,
            56  => EISCONN,
            57  => ENOTCONN,
            58  => ESHUTDOWN,
            59  => ETOOMANYREFS,
            60  => ETIMEDOUT,
            61  => ECONNREFUSED,
            62  => ELOOP,
            63  => ENAMETOOLONG,
            64  => EHOSTDOWN,
            65  => EHOSTUNREACH,
            66  => ENOTEMPTY,
            67  => EPROCLIM,
            68  => EUSERS,
            69  => EDQUOT,
            70  => ESTALE,
            71  => EREMOTE,
            72  => EBADRPC,
            73  => ERPCMISMATCH,
            74  => EPROGUNAVAIL,
            75  => EPROGMISMATCH,
            76  => EPROCUNAVAIL,
            77  => ENOLCK,
            78  => ENOSYS,
            79  => EFTYPE,
            80  => EAUTH,
            81  => ENEEDAUTH,
            82  => EIDRM,
            83  => ENOMSG,
            84  => EOVERFLOW,
            85  => ECANCELED,
            86  => EILSEQ,
            87  => ENOATTR,
            88  => EDOOFUS,
            89  => EBADMSG,
            90  => EMULTIHOP,
            91  => ENOLINK,
            92  => EPROTO,
            93  => ENOMEDIUM,
            94  => EUNUSED94,
            95  => EUNUSED95,
            96  => EUNUSED96,
            97  => EUNUSED97,
            98  => EUNUSED98,
            99  => EASYNC,
            _   => UnknownErrno,
        }
    }
}


#[cfg(target_os = "openbsd")]
mod consts {
    #[derive(Copy, Debug, Clone, PartialEq)]
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
        EOPNOTSUPP      = 45,
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
        EIPSEC          = 82,
        ENOATTR         = 83,
        EILSEQ          = 84,
        ENOMEDIUM       = 85,
        EMEDIUMTYPE     = 86,
        EOVERFLOW       = 87,
        ECANCELED       = 88,
        EIDRM           = 89,
        ENOMSG          = 90,
        ENOTSUP         = 91,
    }

    pub const ELAST: Errno       = Errno::ENOTSUP;
    pub const EWOULDBLOCK: Errno = Errno::EAGAIN;

    pub const EL2NSYNC: Errno = Errno::UnknownErrno;

    pub fn from_i32(e: i32) -> Errno {
        use self::Errno::*;

        match e {
            0   => UnknownErrno,
            1   => EPERM,
            2   => ENOENT,
            3   => ESRCH,
            4   => EINTR,
            5   => EIO,
            6   => ENXIO,
            7   => E2BIG,
            8   => ENOEXEC,
            9   => EBADF,
            10  => ECHILD,
            11  => EDEADLK,
            12  => ENOMEM,
            13  => EACCES,
            14  => EFAULT,
            15  => ENOTBLK,
            16  => EBUSY,
            17  => EEXIST,
            18  => EXDEV,
            19  => ENODEV,
            20  => ENOTDIR,
            21  => EISDIR,
            22  => EINVAL,
            23  => ENFILE,
            24  => EMFILE,
            25  => ENOTTY,
            26  => ETXTBSY,
            27  => EFBIG,
            28  => ENOSPC,
            29  => ESPIPE,
            30  => EROFS,
            31  => EMLINK,
            32  => EPIPE,
            33  => EDOM,
            34  => ERANGE,
            35  => EAGAIN,
            36  => EINPROGRESS,
            37  => EALREADY,
            38  => ENOTSOCK,
            39  => EDESTADDRREQ,
            40  => EMSGSIZE,
            41  => EPROTOTYPE,
            42  => ENOPROTOOPT,
            43  => EPROTONOSUPPORT,
            44  => ESOCKTNOSUPPORT,
            45  => EOPNOTSUPP,
            46  => EPFNOSUPPORT,
            47  => EAFNOSUPPORT,
            48  => EADDRINUSE,
            49  => EADDRNOTAVAIL,
            50  => ENETDOWN,
            51  => ENETUNREACH,
            52  => ENETRESET,
            53  => ECONNABORTED,
            54  => ECONNRESET,
            55  => ENOBUFS,
            56  => EISCONN,
            57  => ENOTCONN,
            58  => ESHUTDOWN,
            59  => ETOOMANYREFS,
            60  => ETIMEDOUT,
            61  => ECONNREFUSED,
            62  => ELOOP,
            63  => ENAMETOOLONG,
            64  => EHOSTDOWN,
            65  => EHOSTUNREACH,
            66  => ENOTEMPTY,
            67  => EPROCLIM,
            68  => EUSERS,
            69  => EDQUOT,
            70  => ESTALE,
            71  => EREMOTE,
            72  => EBADRPC,
            73  => ERPCMISMATCH,
            74  => EPROGUNAVAIL,
            75  => EPROGMISMATCH,
            76  => EPROCUNAVAIL,
            77  => ENOLCK,
            78  => ENOSYS,
            79  => EFTYPE,
            80  => EAUTH,
            81  => ENEEDAUTH,
            82  => EIPSEC,
            83  => ENOATTR,
            84  => EILSEQ,
            85  => ENOMEDIUM,
            86  => EMEDIUMTYPE,
            87  => EOVERFLOW,
            88  => ECANCELED,
            89  => EIDRM,
            90  => ENOMSG,
            91  => ENOTSUP,
            _   => UnknownErrno,
        }
    }
}

#[cfg(target_os = "netbsd")]
mod consts {
    #[derive(Copy, Debug, Clone, PartialEq)]
    pub enum Errno {
        UnknownErrno    = 0,
        EPERM		= 1,
        ENOENT		= 2,
        ESRCH		= 3,
        EINTR		= 4,
        EIO		= 5,
        ENXIO		= 6,
        E2BIG		= 7,
        ENOEXEC		= 8,
        EBADF		= 9,
        ECHILD		= 10,
        EDEADLK		= 11,
        ENOMEM		= 12,
        EACCES		= 13,
        EFAULT		= 14,
        ENOTBLK		= 15,
        EBUSY		= 16,
        EEXIST		= 17,
        EXDEV		= 18,
        ENODEV		= 19,
        ENOTDIR		= 20,
        EISDIR		= 21,
        EINVAL		= 22,
        ENFILE		= 23,
        EMFILE		= 24,
        ENOTTY		= 25,
        ETXTBSY		= 26,
        EFBIG		= 27,
        ENOSPC		= 28,
        ESPIPE		= 29,
        EROFS		= 30,
        EMLINK		= 31,
        EPIPE		= 32,
        EDOM		= 33,
        ERANGE		= 34,
        EAGAIN		= 35,
        EINPROGRESS	= 36,
        EALREADY	= 37,
        ENOTSOCK	= 38,
        EDESTADDRREQ	= 39,
        EMSGSIZE	= 40,
        EPROTOTYPE	= 41,
        ENOPROTOOPT	= 42,
        EPROTONOSUPPORT	= 43,
        ESOCKTNOSUPPORT	= 44,
        EOPNOTSUPP	= 45,
        EPFNOSUPPORT	= 46,
        EAFNOSUPPORT	= 47,
        EADDRINUSE	= 48,
        EADDRNOTAVAIL	= 49,
        ENETDOWN	= 50,
        ENETUNREACH	= 51,
        ENETRESET	= 52,
        ECONNABORTED	= 53,
        ECONNRESET	= 54,
        ENOBUFS		= 55,
        EISCONN		= 56,
        ENOTCONN	= 57,
        ESHUTDOWN	= 58,
        ETOOMANYREFS	= 59,
        ETIMEDOUT	= 60,
        ECONNREFUSED	= 61,
        ELOOP		= 62,
        ENAMETOOLONG	= 63,
        EHOSTDOWN	= 64,
        EHOSTUNREACH	= 65,
        ENOTEMPTY	= 66,
        EPROCLIM	= 67,
        EUSERS		= 68,
        EDQUOT		= 69,
        ESTALE		= 70,
        EREMOTE		= 71,
        EBADRPC		= 72,
        ERPCMISMATCH	= 73,
        EPROGUNAVAIL	= 74,
        EPROGMISMATCH	= 75,
        EPROCUNAVAIL	= 76,
        ENOLCK		= 77,
        ENOSYS		= 78,
        EFTYPE		= 79,
        EAUTH		= 80,
        ENEEDAUTH	= 81,
        EIDRM		= 82,
        ENOMSG		= 83,
        EOVERFLOW	= 84,
        EILSEQ		= 85,
        ENOTSUP		= 86,
        ECANCELED	= 87,
        EBADMSG		= 88,
        ENODATA		= 89,
        ENOSR		= 90,
        ENOSTR		= 91,
        ETIME		= 92,
        ENOATTR		= 93,
        EMULTIHOP	= 94,
        ENOLINK		= 95,
        EPROTO		= 96,
    }

    pub const ELAST: Errno       = Errno::ENOTSUP;
    pub const EWOULDBLOCK: Errno = Errno::EAGAIN;

    pub const EL2NSYNC: Errno = Errno::UnknownErrno;

    pub fn from_i32(e: i32) -> Errno {
        use self::Errno::*;

        match e {
            0   => UnknownErrno,
            1	=> EPERM,
            2	=> ENOENT,
            3	=> ESRCH,
            4	=> EINTR,
            5	=> EIO,
            6	=> ENXIO,
            7	=> E2BIG,
            8	=> ENOEXEC,
            9	=> EBADF,
            10	=> ECHILD,
            11	=> EDEADLK,
            12	=> ENOMEM,
            13	=> EACCES,
            14	=> EFAULT,
            15	=> ENOTBLK,
            16	=> EBUSY,
            17	=> EEXIST,
            18	=> EXDEV,
            19	=> ENODEV,
            20	=> ENOTDIR,
            21	=> EISDIR,
            22	=> EINVAL,
            23	=> ENFILE,
            24	=> EMFILE,
            25	=> ENOTTY,
            26	=> ETXTBSY,
            27	=> EFBIG,
            28	=> ENOSPC,
            29	=> ESPIPE,
            30	=> EROFS,
            31	=> EMLINK,
            32	=> EPIPE,
            33	=> EDOM,
            34	=> ERANGE,
            35	=> EAGAIN,
            36	=> EINPROGRESS,
            37	=> EALREADY,
            38	=> ENOTSOCK,
            39	=> EDESTADDRREQ,
            40	=> EMSGSIZE,
            41	=> EPROTOTYPE,
            42	=> ENOPROTOOPT,
            43	=> EPROTONOSUPPORT,
            44	=> ESOCKTNOSUPPORT,
            45	=> EOPNOTSUPP,
            46	=> EPFNOSUPPORT,
            47	=> EAFNOSUPPORT,
            48	=> EADDRINUSE,
            49	=> EADDRNOTAVAIL,
            50	=> ENETDOWN,
            51	=> ENETUNREACH,
            52	=> ENETRESET,
            53	=> ECONNABORTED,
            54	=> ECONNRESET,
            55	=> ENOBUFS,
            56	=> EISCONN,
            57	=> ENOTCONN,
            58	=> ESHUTDOWN,
            59	=> ETOOMANYREFS,
            60	=> ETIMEDOUT,
            61	=> ECONNREFUSED,
            62	=> ELOOP,
            63	=> ENAMETOOLONG,
            64	=> EHOSTDOWN,
            65	=> EHOSTUNREACH,
            66	=> ENOTEMPTY,
            67	=> EPROCLIM,
            68	=> EUSERS,
            69	=> EDQUOT,
            70	=> ESTALE,
            71	=> EREMOTE,
            72	=> EBADRPC,
            73	=> ERPCMISMATCH,
            74	=> EPROGUNAVAIL,
            75	=> EPROGMISMATCH,
            76	=> EPROCUNAVAIL,
            77	=> ENOLCK,
            78	=> ENOSYS,
            79	=> EFTYPE,
            80	=> EAUTH,
            81	=> ENEEDAUTH,
            82	=> EIDRM,
            83	=> ENOMSG,
            84	=> EOVERFLOW,
            85	=> EILSEQ,
            86	=> ENOTSUP,
            87	=> ECANCELED,
            88	=> EBADMSG,
            89	=> ENODATA,
            90	=> ENOSR,
            91	=> ENOSTR,
            92	=> ETIME,
            93	=> ENOATTR,
            94	=> EMULTIHOP,
            95	=> ENOLINK,
            96	=> EPROTO,
            _   => UnknownErrno,
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use nixtest::assert_const_eq;
    use libc::c_int;

    macro_rules! check_errno {
        ($($errno:ident),+) => {{
            $(assert_const_eq(stringify!($errno), $errno as c_int);)+
        }};
    }

    #[test]
    pub fn test_errno_values() {
        check_errno!(
            EPERM,
            ENOENT,
            ESRCH,
            EINTR,
            EIO,
            ENXIO,
            E2BIG,
            ENOEXEC,
            EBADF,
            ECHILD,
            EAGAIN,
            ENOMEM,
            EACCES,
            EFAULT,
            ENOTBLK,
            EBUSY,
            EEXIST,
            EXDEV,
            ENODEV,
            ENOTDIR,
            EISDIR,
            EINVAL,
            ENFILE,
            EMFILE,
            ENOTTY,
            ETXTBSY,
            EFBIG,
            ENOSPC,
            ESPIPE,
            EROFS,
            EMLINK,
            EPIPE,
            EDOM,
            ERANGE,
            EDEADLK,
            ENAMETOOLONG,
            ENOLCK,
            ENOSYS,
            ENOTEMPTY,
            ELOOP,
            ENOMSG,
            EIDRM);

        check_errno!(
            EINPROGRESS,
            EALREADY,
            ENOTSOCK,
            EDESTADDRREQ,
            EMSGSIZE,
            EPROTOTYPE,
            ENOPROTOOPT,
            EPROTONOSUPPORT,
            ESOCKTNOSUPPORT,
            EPFNOSUPPORT,
            EAFNOSUPPORT,
            EADDRINUSE,
            EADDRNOTAVAIL,
            ENETDOWN,
            ENETUNREACH,
            ENETRESET,
            ECONNABORTED,
            ECONNRESET,
            ENOBUFS,
            EISCONN,
            ENOTCONN,
            ESHUTDOWN,
            ETOOMANYREFS,
            ETIMEDOUT,
            ECONNREFUSED,
            EHOSTDOWN,
            EHOSTUNREACH);
    }

    #[test]
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub fn test_linux_errnos() {
        check_errno!(
            ECHRNG,
            EL2NSYNC,
            EL3HLT,
            EL3RST,
            ELNRNG,
            EUNATCH,
            ENOCSI,
            EL2HLT,
            EBADE,
            EBADR,
            EXFULL,
            ENOANO,
            EBADRQC,
            EBADSLT,
            EBFONT,
            ENOSTR,
            ENODATA,
            ETIME,
            ENOSR,
            ENONET,
            ENOPKG,
            EREMOTE,
            ENOLINK,
            EADV,
            ESRMNT,
            ECOMM,
            EPROTO,
            EMULTIHOP,
            EDOTDOT,
            EBADMSG,
            EOVERFLOW,
            ENOTUNIQ,
            EBADFD,
            EREMCHG,
            ELIBACC,
            ELIBBAD,
            ELIBSCN,
            ELIBMAX,
            ELIBEXEC,
            EILSEQ,
            ERESTART,
            ESTRPIPE,
            EUSERS,
            EOPNOTSUPP,
            ESTALE,
            EUCLEAN,
            ENOTNAM,
            ENAVAIL,
            EISNAM,
            EREMOTEIO,
            EDQUOT,
            ENOMEDIUM,
            EMEDIUMTYPE,
            ECANCELED,
            ENOKEY,
            EKEYEXPIRED,
            EKEYREVOKED,
            EKEYREJECTED,
            EOWNERDEAD,
            ENOTRECOVERABLE);
    }

    #[test]
    #[cfg(target_os = "linux")]
    pub fn test_linux_not_android_errnos() {
        check_errno!(
            ERFKILL /*,
            EHWPOISON */);
    }

    #[test]
    #[cfg(target_os = "freebsd")]
    pub fn test_freebsd_errnos() {
        check_errno!(
            EDOOFUS,
            EMULTIHOP,
            ENOLINK,
            ENOTCAPABLE,
            ECAPMODE,
            ENEEDAUTH,
            EOVERFLOW,
            EILSEQ,
            ENOATTR,
            EBADMSG,
            EPROTO,
            ENOTRECOVERABLE,
            EOWNERDEAD,
            ENOTSUP,
            EPROCLIM,
            EUSERS,
            EDQUOT,
            ESTALE,
            EREMOTE,
            EBADRPC,
            ERPCMISMATCH,
            EPROGUNAVAIL,
            EPROGMISMATCH,
            EPROCUNAVAIL,
            EAUTH);
    }

    #[test]
    #[cfg(target_os = "dragonfly")]
    pub fn test_dragonfly_errnos() {
        check_errno!(
            EDOOFUS,
            EMULTIHOP,
            ENOLINK,
            ENEEDAUTH,
            EOVERFLOW,
            EILSEQ,
            ENOATTR,
            EBADMSG,
            EPROTO,
            ENOTSUP,
            EPROCLIM,
            EUSERS,
            EDQUOT,
            ESTALE,
            EREMOTE,
            EBADRPC,
            ERPCMISMATCH,
            EPROGUNAVAIL,
            EPROGMISMATCH,
            EPROCUNAVAIL,
            EAUTH);
    }

    #[test]
    #[cfg(target_os = "openbsd")]
    pub fn test_openbsd_errnos() {
        check_errno!(
            EADDRINUSE,
            EADDRNOTAVAIL,
            EAFNOSUPPORT,
            EALREADY,
            EAUTH,
            EBADRPC,
            ECANCELED,
            ECONNABORTED,
            ECONNREFUSED,
            ECONNRESET,
            EDESTADDRREQ,
            EDQUOT,
            EFTYPE,
            EHOSTDOWN,
            EHOSTUNREACH,
            EILSEQ,
            EINPROGRESS,
            EIPSEC,
            EISCONN,
            EMEDIUMTYPE,
            EMSGSIZE,
            ENEEDAUTH,
            ENETDOWN,
            ENETRESET,
            ENETUNREACH,
            ENOATTR,
            ENOBUFS,
            ENOMEDIUM,
            ENOPROTOOPT,
            ENOTCONN,
            ENOTSOCK,
            ENOTSUP,
            EOPNOTSUPP,
            EOVERFLOW,
            EPFNOSUPPORT,
            EPROCLIM,
            EPROCUNAVAIL,
            EPROGMISMATCH,
            EPROGUNAVAIL,
            EPROTONOSUPPORT,
            EPROTOTYPE,
            EREMOTE,
            ESHUTDOWN,
            ESOCKTNOSUPPORT,
            ESTALE,
            ETIMEDOUT,
            ETOOMANYREFS,
            EUSERS);
    }

    #[test]
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub fn test_darwin_errnos() {
        check_errno!(
            ENOTSUP,
            EPROCLIM,
            EUSERS,
            EDQUOT,
            ESTALE,
            EREMOTE,
            EBADRPC,
            ERPCMISMATCH,
            EPROGUNAVAIL,
            EPROGMISMATCH,
            EPROCUNAVAIL,
            EFTYPE,
            EAUTH,
            ENEEDAUTH,
            EPWROFF,
            EDEVERR,
            EOVERFLOW,
            EBADEXEC,
            EBADARCH,
            ESHLIBVERS,
            EBADMACHO,
            ECANCELED,
            EILSEQ,
            ENOATTR,
            EBADMSG,
            EMULTIHOP,
            ENODATA,
            ENOLINK,
            ENOSR,
            ENOSTR,
            EPROTO,
            ETIME,
            EOPNOTSUPP,
            ENOPOLICY,
            ENOTRECOVERABLE,
            EOWNERDEAD,
            EQFULL
        );
    }
}
