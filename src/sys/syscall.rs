//! Indirect system call
//!
use libc::c_int;

pub use self::arch::*;

#[cfg(target_arch = "x86_64")]
mod arch {
    use libc::c_long;

    pub type Syscall = c_long;

    pub const SOCKET: Syscall = 41;
    pub const CONNECT: Syscall = 42;
    pub const SENDMSG: Syscall = 46;
    pub const RECVMSG: Syscall = 47;
    pub const SYSPIVOTROOT: Syscall = 155;
    pub const MEMFD_CREATE: Syscall = 319;
}

#[cfg(target_arch = "x86")]
mod arch {
    use libc::c_long;

    pub type Syscall = c_long;

    pub const SOCKETCALL: Syscall = 102;
    pub const SYSPIVOTROOT: Syscall = 217;
    pub const MEMFD_CREATE: Syscall = 356;
}

#[cfg(target_arch = "aarch64")]
mod arch {
    use libc::c_long;

    pub type Syscall = c_long;

    pub static SYSPIVOTROOT: Syscall = 41;
    pub static MEMFD_CREATE: Syscall = 279;
}

#[cfg(target_arch = "arm")]
mod arch {
    use libc::c_long;

    pub type Syscall = c_long;

    pub const SYSPIVOTROOT: Syscall = 218;
    pub const SOCKET: Syscall = 281;
    pub const CONNECT: Syscall = 283;
    pub const SENDMSG: Syscall = 296;
    pub const RECVMSG: Syscall = 297;
    pub const MEMFD_CREATE: Syscall = 385;
}


extern {
    pub fn syscall(num: Syscall, ...) -> c_int;
}
