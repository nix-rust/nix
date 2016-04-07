//! Indirect system call

pub use self::os::*;

#[cfg(any(target_os = "linux",
          target_os = "android",
          target_os = "emscripten"))]
mod os {
    use libc::c_long;
    pub type Syscall = c_long;
}

#[cfg(any(target_os = "macos",
          target_os = "ios",
          target_os = "freebsd",
          target_os = "dragonfly",
          target_os = "openbsd",
          target_os = "netbsd",
          target_os = "bitrig"))]
mod os {
    use libc::c_int;
    pub type Syscall = c_int;
}

pub use self::arch::*;

#[cfg(target_arch = "x86_64")]
mod arch {
    use super::Syscall;

    pub static SYSPIVOTROOT: Syscall = 155;
    pub static MEMFD_CREATE: Syscall = 319;
}

#[cfg(target_arch = "x86")]
mod arch {
    use super::Syscall;

    pub static SYSPIVOTROOT: Syscall = 217;
    pub static MEMFD_CREATE: Syscall = 356;
}

#[cfg(target_arch = "aarch64")]
mod arch {
    use super::Syscall;

    pub static SYSPIVOTROOT: Syscall = 41;
    pub static MEMFD_CREATE: Syscall = 279;
}

#[cfg(target_arch = "arm")]
mod arch {
    use super::Syscall;

    pub static SYSPIVOTROOT: Syscall = 218;
    pub static MEMFD_CREATE: Syscall = 385;
}

#[cfg(target_arch = "mips")]
mod arch {
    use super::Syscall;

    pub static SYSPIVOTROOT: Syscall = 216;
    pub static MEMFD_CREATE: Syscall = 354;
}

pub use libc::syscall;
