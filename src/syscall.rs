#![cfg(target_os = "linux")]

use libc::c_int;

pub use self::arch::*;

#[cfg(target_arch = "x86_64")]
mod arch {
    use libc::c_long;

    pub type Syscall = c_long;

    pub static SysPivotRoot: Syscall = 155;
}

extern {
    pub fn syscall(num: Syscall, ...) -> c_int;
}
