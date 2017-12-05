use libc::{self, c_int};
use std::{fmt, io, error};
use {Error, Result};

pub use self::consts::*;

cfg_if! {
    if #[cfg(any(target_os = "freebsd",
                 target_os = "ios",
                 target_os = "macos"))] {
        unsafe fn errno_location() -> *mut c_int {
            libc::__error()
        }
    } else if #[cfg(target_os = "dragonfly")] {
        unsafe fn errno_location() -> *mut c_int {
            // FIXME: Replace with errno-dragonfly crate as this is no longer the correct
            //        implementation.
            extern { fn __dfly_error() -> *mut c_int; }
            __dfly_error()
        }
    } else if #[cfg(any(target_os = "android",
                        target_os = "netbsd",
                        target_os = "openbsd"))] {
        unsafe fn errno_location() -> *mut c_int {
            libc::__errno()
        }
    } else if #[cfg(target_os = "linux")] {
        unsafe fn errno_location() -> *mut c_int {
            libc::__errno_location()
        }
    }
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
    use self::Errno::*;
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

        #[cfg(all(target_os = "linux", not(target_arch="mips")))]
        ERFKILL         => "Operation not possible due to RF-kill",

        #[cfg(all(target_os = "linux", not(target_arch="mips")))]
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
    use libc;

    #[derive(Debug, Clone, PartialEq, Copy)]
    pub enum Errno {
        UnknownErrno    = 0,
        EPERM           = libc::EPERM as isize,
        ENOENT          = libc::ENOENT as isize,
        ESRCH           = libc::ESRCH as isize,
        EINTR           = libc::EINTR as isize,
        EIO             = libc::EIO as isize,
        ENXIO           = libc::ENXIO as isize,
        E2BIG           = libc::E2BIG as isize,
        ENOEXEC         = libc::ENOEXEC as isize,
        EBADF           = libc::EBADF as isize,
        ECHILD          = libc::ECHILD as isize,
        EAGAIN          = libc::EAGAIN as isize,
        ENOMEM          = libc::ENOMEM as isize,
        EACCES          = libc::EACCES as isize,
        EFAULT          = libc::EFAULT as isize,
        ENOTBLK         = libc::ENOTBLK as isize,
        EBUSY           = libc::EBUSY as isize,
        EEXIST          = libc::EEXIST as isize,
        EXDEV           = libc::EXDEV as isize,
        ENODEV          = libc::ENODEV as isize,
        ENOTDIR         = libc::ENOTDIR as isize,
        EISDIR          = libc::EISDIR as isize,
        EINVAL          = libc::EINVAL as isize,
        ENFILE          = libc::ENFILE as isize,
        EMFILE          = libc::EMFILE as isize,
        ENOTTY          = libc::ENOTTY as isize,
        ETXTBSY         = libc::ETXTBSY as isize,
        EFBIG           = libc::EFBIG as isize,
        ENOSPC          = libc::ENOSPC as isize,
        ESPIPE          = libc::ESPIPE as isize,
        EROFS           = libc::EROFS as isize,
        EMLINK          = libc::EMLINK as isize,
        EPIPE           = libc::EPIPE as isize,
        EDOM            = libc::EDOM as isize,
        ERANGE          = libc::ERANGE as isize,
        EDEADLK         = libc::EDEADLK as isize,
        ENAMETOOLONG    = libc::ENAMETOOLONG as isize,
        ENOLCK          = libc::ENOLCK as isize,
        ENOSYS          = libc::ENOSYS as isize,
        ENOTEMPTY       = libc::ENOTEMPTY as isize,
        ELOOP           = libc::ELOOP as isize,
        ENOMSG          = libc::ENOMSG as isize,
        EIDRM           = libc::EIDRM as isize,
        ECHRNG          = libc::ECHRNG as isize,
        EL2NSYNC        = libc::EL2NSYNC as isize,
        EL3HLT          = libc::EL3HLT as isize,
        EL3RST          = libc::EL3RST as isize,
        ELNRNG          = libc::ELNRNG as isize,
        EUNATCH         = libc::EUNATCH as isize,
        ENOCSI          = libc::ENOCSI as isize,
        EL2HLT          = libc::EL2HLT as isize,
        EBADE           = libc::EBADE as isize,
        EBADR           = libc::EBADR as isize,
        EXFULL          = libc::EXFULL as isize,
        ENOANO          = libc::ENOANO as isize,
        EBADRQC         = libc::EBADRQC as isize,
        EBADSLT         = libc::EBADSLT as isize,
        EBFONT          = libc::EBFONT as isize,
        ENOSTR          = libc::ENOSTR as isize,
        ENODATA         = libc::ENODATA as isize,
        ETIME           = libc::ETIME as isize,
        ENOSR           = libc::ENOSR as isize,
        ENONET          = libc::ENONET as isize,
        ENOPKG          = libc::ENOPKG as isize,
        EREMOTE         = libc::EREMOTE as isize,
        ENOLINK         = libc::ENOLINK as isize,
        EADV            = libc::EADV as isize,
        ESRMNT          = libc::ESRMNT as isize,
        ECOMM           = libc::ECOMM as isize,
        EPROTO          = libc::EPROTO as isize,
        EMULTIHOP       = libc::EMULTIHOP as isize,
        EDOTDOT         = libc::EDOTDOT as isize,
        EBADMSG         = libc::EBADMSG as isize,
        EOVERFLOW       = libc::EOVERFLOW as isize,
        ENOTUNIQ        = libc::ENOTUNIQ as isize,
        EBADFD          = libc::EBADFD as isize,
        EREMCHG         = libc::EREMCHG as isize,
        ELIBACC         = libc::ELIBACC as isize,
        ELIBBAD         = libc::ELIBBAD as isize,
        ELIBSCN         = libc::ELIBSCN as isize,
        ELIBMAX         = libc::ELIBMAX as isize,
        ELIBEXEC        = libc::ELIBEXEC as isize,
        EILSEQ          = libc::EILSEQ as isize,
        ERESTART        = libc::ERESTART as isize,
        ESTRPIPE        = libc::ESTRPIPE as isize,
        EUSERS          = libc::EUSERS as isize,
        ENOTSOCK        = libc::ENOTSOCK as isize,
        EDESTADDRREQ    = libc::EDESTADDRREQ as isize,
        EMSGSIZE        = libc::EMSGSIZE as isize,
        EPROTOTYPE      = libc::EPROTOTYPE as isize,
        ENOPROTOOPT     = libc::ENOPROTOOPT as isize,
        EPROTONOSUPPORT = libc::EPROTONOSUPPORT as isize,
        ESOCKTNOSUPPORT = libc::ESOCKTNOSUPPORT as isize,
        EOPNOTSUPP      = libc::EOPNOTSUPP as isize,
        EPFNOSUPPORT    = libc::EPFNOSUPPORT as isize,
        EAFNOSUPPORT    = libc::EAFNOSUPPORT as isize,
        EADDRINUSE      = libc::EADDRINUSE as isize,
        EADDRNOTAVAIL   = libc::EADDRNOTAVAIL as isize,
        ENETDOWN        = libc::ENETDOWN as isize,
        ENETUNREACH     = libc::ENETUNREACH as isize,
        ENETRESET       = libc::ENETRESET as isize,
        ECONNABORTED    = libc::ECONNABORTED as isize,
        ECONNRESET      = libc::ECONNRESET as isize,
        ENOBUFS         = libc::ENOBUFS as isize,
        EISCONN         = libc::EISCONN as isize,
        ENOTCONN        = libc::ENOTCONN as isize,
        ESHUTDOWN       = libc::ESHUTDOWN as isize,
        ETOOMANYREFS    = libc::ETOOMANYREFS as isize,
        ETIMEDOUT       = libc::ETIMEDOUT as isize,
        ECONNREFUSED    = libc::ECONNREFUSED as isize,
        EHOSTDOWN       = libc::EHOSTDOWN as isize,
        EHOSTUNREACH    = libc::EHOSTUNREACH as isize,
        EALREADY        = libc::EALREADY as isize,
        EINPROGRESS     = libc::EINPROGRESS as isize,
        ESTALE          = libc::ESTALE as isize,
        EUCLEAN         = libc::EUCLEAN as isize,
        ENOTNAM         = libc::ENOTNAM as isize,
        ENAVAIL         = libc::ENAVAIL as isize,
        EISNAM          = libc::EISNAM as isize,
        EREMOTEIO       = libc::EREMOTEIO as isize,
        EDQUOT          = libc::EDQUOT as isize,
        ENOMEDIUM       = libc::ENOMEDIUM as isize,
        EMEDIUMTYPE     = libc::EMEDIUMTYPE as isize,
        ECANCELED       = libc::ECANCELED as isize,
        ENOKEY          = libc::ENOKEY as isize,
        EKEYEXPIRED     = libc::EKEYEXPIRED as isize,
        EKEYREVOKED     = libc::EKEYREVOKED as isize,
        EKEYREJECTED    = libc::EKEYREJECTED as isize,
        EOWNERDEAD      = libc::EOWNERDEAD as isize,
        ENOTRECOVERABLE = libc::ENOTRECOVERABLE as isize,

        #[cfg(not(any(target_os = "android", target_arch="mips")))]
        ERFKILL         = libc::ERFKILL as isize,
        #[cfg(not(any(target_os = "android", target_arch="mips")))]
        EHWPOISON       = libc::EHWPOISON as isize,
    }

    pub const EWOULDBLOCK: Errno = Errno::EAGAIN;
    pub const EDEADLOCK:   Errno = Errno::EDEADLK;

    pub fn from_i32(e: i32) -> Errno {
        use self::Errno::*;

        match e {
            0   => UnknownErrno,
            libc::EPERM => EPERM,
            libc::ENOENT => ENOENT,
            libc::ESRCH => ESRCH,
            libc::EINTR => EINTR,
            libc::EIO => EIO,
            libc::ENXIO => ENXIO,
            libc::E2BIG => E2BIG,
            libc::ENOEXEC => ENOEXEC,
            libc::EBADF => EBADF,
            libc::ECHILD => ECHILD,
            libc::EAGAIN => EAGAIN,
            libc::ENOMEM => ENOMEM,
            libc::EACCES => EACCES,
            libc::EFAULT => EFAULT,
            libc::ENOTBLK => ENOTBLK,
            libc::EBUSY => EBUSY,
            libc::EEXIST => EEXIST,
            libc::EXDEV => EXDEV,
            libc::ENODEV => ENODEV,
            libc::ENOTDIR => ENOTDIR,
            libc::EISDIR => EISDIR,
            libc::EINVAL => EINVAL,
            libc::ENFILE => ENFILE,
            libc::EMFILE => EMFILE,
            libc::ENOTTY => ENOTTY,
            libc::ETXTBSY => ETXTBSY,
            libc::EFBIG => EFBIG,
            libc::ENOSPC => ENOSPC,
            libc::ESPIPE => ESPIPE,
            libc::EROFS => EROFS,
            libc::EMLINK => EMLINK,
            libc::EPIPE => EPIPE,
            libc::EDOM => EDOM,
            libc::ERANGE => ERANGE,
            libc::EDEADLK => EDEADLK,
            libc::ENAMETOOLONG => ENAMETOOLONG,
            libc::ENOLCK => ENOLCK,
            libc::ENOSYS => ENOSYS,
            libc::ENOTEMPTY => ENOTEMPTY,
            libc::ELOOP => ELOOP,
            libc::ENOMSG => ENOMSG,
            libc::EIDRM => EIDRM,
            libc::ECHRNG => ECHRNG,
            libc::EL2NSYNC => EL2NSYNC,
            libc::EL3HLT => EL3HLT,
            libc::EL3RST => EL3RST,
            libc::ELNRNG => ELNRNG,
            libc::EUNATCH => EUNATCH,
            libc::ENOCSI => ENOCSI,
            libc::EL2HLT => EL2HLT,
            libc::EBADE => EBADE,
            libc::EBADR => EBADR,
            libc::EXFULL => EXFULL,
            libc::ENOANO => ENOANO,
            libc::EBADRQC => EBADRQC,
            libc::EBADSLT => EBADSLT,
            libc::EBFONT => EBFONT,
            libc::ENOSTR => ENOSTR,
            libc::ENODATA => ENODATA,
            libc::ETIME => ETIME,
            libc::ENOSR => ENOSR,
            libc::ENONET => ENONET,
            libc::ENOPKG => ENOPKG,
            libc::EREMOTE => EREMOTE,
            libc::ENOLINK => ENOLINK,
            libc::EADV => EADV,
            libc::ESRMNT => ESRMNT,
            libc::ECOMM => ECOMM,
            libc::EPROTO => EPROTO,
            libc::EMULTIHOP => EMULTIHOP,
            libc::EDOTDOT => EDOTDOT,
            libc::EBADMSG => EBADMSG,
            libc::EOVERFLOW => EOVERFLOW,
            libc::ENOTUNIQ => ENOTUNIQ,
            libc::EBADFD => EBADFD,
            libc::EREMCHG => EREMCHG,
            libc::ELIBACC => ELIBACC,
            libc::ELIBBAD => ELIBBAD,
            libc::ELIBSCN => ELIBSCN,
            libc::ELIBMAX => ELIBMAX,
            libc::ELIBEXEC => ELIBEXEC,
            libc::EILSEQ => EILSEQ,
            libc::ERESTART => ERESTART,
            libc::ESTRPIPE => ESTRPIPE,
            libc::EUSERS => EUSERS,
            libc::ENOTSOCK => ENOTSOCK,
            libc::EDESTADDRREQ => EDESTADDRREQ,
            libc::EMSGSIZE => EMSGSIZE,
            libc::EPROTOTYPE => EPROTOTYPE,
            libc::ENOPROTOOPT => ENOPROTOOPT,
            libc::EPROTONOSUPPORT => EPROTONOSUPPORT,
            libc::ESOCKTNOSUPPORT => ESOCKTNOSUPPORT,
            libc::EOPNOTSUPP => EOPNOTSUPP,
            libc::EPFNOSUPPORT => EPFNOSUPPORT,
            libc::EAFNOSUPPORT => EAFNOSUPPORT,
            libc::EADDRINUSE => EADDRINUSE,
            libc::EADDRNOTAVAIL => EADDRNOTAVAIL,
            libc::ENETDOWN => ENETDOWN,
            libc::ENETUNREACH => ENETUNREACH,
            libc::ENETRESET => ENETRESET,
            libc::ECONNABORTED => ECONNABORTED,
            libc::ECONNRESET => ECONNRESET,
            libc::ENOBUFS => ENOBUFS,
            libc::EISCONN => EISCONN,
            libc::ENOTCONN => ENOTCONN,
            libc::ESHUTDOWN => ESHUTDOWN,
            libc::ETOOMANYREFS => ETOOMANYREFS,
            libc::ETIMEDOUT => ETIMEDOUT,
            libc::ECONNREFUSED => ECONNREFUSED,
            libc::EHOSTDOWN => EHOSTDOWN,
            libc::EHOSTUNREACH => EHOSTUNREACH,
            libc::EALREADY => EALREADY,
            libc::EINPROGRESS => EINPROGRESS,
            libc::ESTALE => ESTALE,
            libc::EUCLEAN => EUCLEAN,
            libc::ENOTNAM => ENOTNAM,
            libc::ENAVAIL => ENAVAIL,
            libc::EISNAM => EISNAM,
            libc::EREMOTEIO => EREMOTEIO,
            libc::EDQUOT => EDQUOT,
            libc::ENOMEDIUM => ENOMEDIUM,
            libc::EMEDIUMTYPE => EMEDIUMTYPE,
            libc::ECANCELED => ECANCELED,
            libc::ENOKEY => ENOKEY,
            libc::EKEYEXPIRED => EKEYEXPIRED,
            libc::EKEYREVOKED => EKEYREVOKED,
            libc::EKEYREJECTED => EKEYREJECTED,
            libc::EOWNERDEAD => EOWNERDEAD,
            libc::ENOTRECOVERABLE => ENOTRECOVERABLE,

            #[cfg(not(any(target_os = "android", target_arch="mips")))]
            libc::ERFKILL => ERFKILL,
            #[cfg(not(any(target_os = "android", target_arch="mips")))]
            libc::EHWPOISON => EHWPOISON,
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
    use super::Errno::*;
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
    #[cfg(all(target_os = "linux", not(target_arch = "mips")))]
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
