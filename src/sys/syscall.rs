//! Indirect system call
//!
use libc::c_int;

pub use self::arch::*;

#[cfg(target_arch = "x86_64")]
mod arch {
    use libc::c_long;

    pub type Syscall = c_long;

    pub static SYSPIVOTROOT: Syscall = 155;
    pub static MEMFD_CREATE: Syscall = 319;
}

#[cfg(target_arch = "x86")]
mod arch {
    use libc::c_long;

    pub type Syscall = c_long;

    pub static SYSPIVOTROOT: Syscall = 217;
    pub static MEMFD_CREATE: Syscall = 356;
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

    pub static SYSPIVOTROOT: Syscall = 218;
    pub static MEMFD_CREATE: Syscall = 385;
}


extern {
    pub fn syscall(num: Syscall, ...) -> c_int;
}
