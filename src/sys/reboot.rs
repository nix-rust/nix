//! Reboot/shutdown or enable/disable Ctrl-Alt-Delete.

use {Errno, Error, Result};
use libc;
use void::Void;
use std::mem::drop;

/// How exactly should the system be rebooted.
///
/// See [`set_cad_enabled()`](fn.set_cad_enabled.html) for
/// enabling/disabling Ctrl-Alt-Delete.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RebootMode {
    RB_HALT_SYSTEM = libc::RB_HALT_SYSTEM,
    RB_KEXEC = libc::RB_KEXEC,
    RB_POWER_OFF = libc::RB_POWER_OFF,
    RB_AUTOBOOT = libc::RB_AUTOBOOT,
    // we do not support Restart2,
    RB_SW_SUSPEND = libc::RB_SW_SUSPEND,
}

pub fn reboot(how: RebootMode) -> Result<Void> {
    unsafe {
        libc::reboot(how as libc::c_int)
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
