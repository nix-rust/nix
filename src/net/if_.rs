//! Network interface name resolution.
//!
//! Uses Linux and/or POSIX functions to resolve interface names like "eth0"
//! or "socan1" into device numbers.

use libc;
use libc::c_uint;
use {Result, Error, NixPath};

/// Resolve an interface into a interface number.
pub fn if_nametoindex<P: ?Sized + NixPath>(name: &P) -> Result<c_uint> {
    let if_index = try!(name.with_nix_path(|name| unsafe { libc::if_nametoindex(name.as_ptr()) }));

    if if_index == 0 {
        Err(Error::last())
    } else {
        Ok(if_index)
    }
}
