use {Errno, Error, Result};
use libc::c_int;
use void::Void;
use std::mem::drop;

#[allow(overflowing_literals)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RebootMode {
    Halt = 0xcdef0123,
    kexec = 0x45584543,
    PowerOff = 0x4321fedc,
    Restart = 0x1234567,
    // we do not support Restart2,
    Suspend = 0xd000fce1,
}

pub fn reboot(how: RebootMode) -> Result<Void> {
    unsafe {
        ext::reboot(how as c_int)
    };
    Err(Error::Sys(Errno::last()))
}


/// Enable or disable the reboot keystroke (Ctrl-Alt-Delete).
///
/// Corresponds to calling `reboot(RB_ENABLE_CAD)` or `reboot(RB_DISABLE_CAD)` in C.
#[allow(overflowing_literals)]
pub fn set_cad_enabled(enable: bool) -> Result<()> {
    let res = unsafe {
        ext::reboot(if enable { 0x89abcdef } else { 0 })
    };
    Errno::result(res).map(drop)
}

mod ext {
    use libc::c_int;
    extern {
        pub fn reboot(cmd: c_int) -> c_int;
    }
}
