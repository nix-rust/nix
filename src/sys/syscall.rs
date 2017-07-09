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

// Rust on mips uses the N32 ABI
#[cfg(target_arch = "mips")]
mod arch {
    use libc::c_long;

    pub type Syscall = c_long;

    pub static SYSPIVOTROOT: Syscall = 216;
    pub static MEMFD_CREATE: Syscall = 354;
}

// Rust on mips64 uses the N64 ABI
#[cfg(target_arch = "mips64")]
mod arch {
    use libc::c_long;

    pub type Syscall = c_long;

    pub static SYSPIVOTROOT: Syscall = 151;
    pub static MEMFD_CREATE: Syscall = 314;
}

#[cfg(any(target_arch = "powerpc", target_arch = "powerpc64"))]
mod arch {
    use libc::c_long;

    pub type Syscall = c_long;

    pub static SYSPIVOTROOT: Syscall = 203;
    pub static MEMFD_CREATE: Syscall = 360;
}

#[cfg(target_arch = "s390x")]
mod arch {
    use libc::c_long;

    pub type Syscall = c_long;

    pub static SYSPIVOTROOT: Syscall = 217;
    pub static MEMFD_CREATE: Syscall = 350;
}

extern {
    pub fn syscall(num: Syscall, ...) -> c_int;
}
