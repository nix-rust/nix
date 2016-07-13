use {Errno, Error, Result};
use libc;
use void::Void;
use std::mem::drop;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RebootMode {
    Halt,
    kexec,
    PowerOff,
    Restart,
    // we do not support Restart2,
    Suspend,
}

pub fn reboot(how: RebootMode) -> Result<Void> {
    let cmd = match how {
        RebootMode::Halt => libc::RB_HALT_SYSTEM,
        RebootMode::kexec => libc::RB_KEXEC,
        RebootMode::PowerOff => libc::RB_POWER_OFF,
        RebootMode::Restart => libc::RB_AUTOBOOT,
        // we do not support Restart2,
        RebootMode::Suspend => libc::RB_SW_SUSPEND,
    };
    unsafe {
        libc::reboot(cmd)
    };
    Err(Error::Sys(Errno::last()))
}

/// Enable or disable the reboot keystroke (Ctrl-Alt-Delete).
///
/// Corresponds to calling `reboot(RB_ENABLE_CAD)` or `reboot(RB_DISABLE_CAD)` in C.
pub fn set_cad_enabled(enable: bool) -> Result<()> {
    let cmd = if enable {
        libc::RB_ENABLE_CAD
    } else {
        libc::RB_DISABLE_CAD
    };
    let res = unsafe {
        libc::reboot(cmd)
    };
    Errno::result(res).map(drop)
}
