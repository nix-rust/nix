//! Network interface name resolution.
//!
//! Uses Linux and/or POSIX functions to resolve interface names like "eth0"
//! or "socan1" into device numbers.

use libc;
use libc::c_uint;
use {Errno, Result, NixString};

/// Resolve an interface into a interface number.
pub fn if_nametoindex<P: NixString>(name: P) -> Result<c_uint> {
    let if_index = unsafe { libc::if_nametoindex(name.as_ref().as_ptr()) };

    if if_index == 0 {
        Err(Errno::last())
    } else {
        Ok(if_index)
    }
}
