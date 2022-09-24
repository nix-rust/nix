//! Wrapper around getrandom.

use crate::{Errno, Result};
use libc;

libc_enum! {
    /// How the random bytes should be filled
    #[repr(u32)]
    #[non_exhaustive]
    pub enum RandomMode{
        /// Random bytes are drawn from random source
        GRND_RANDOM,
        /// Doesn't block if no random bytes are available
        GRND_NONBLOCK,
    }
}

/// Returns the number of bytes copied to the slice
pub fn getrandom(buffer: &mut [u8], mode: RandomMode) -> Result<isize> {
    let n = unsafe {
        libc::getrandom(
            buffer.as_mut_ptr() as *mut libc::c_void,
            buffer.len(),
            mode as u32,
        )
    };
    Errno::result(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_getrandom() {
        let mut buffer: Vec<u8> = vec![0; 100];
        let n = getrandom(&mut buffer, RandomMode::GRND_RANDOM).unwrap();
        assert_eq!(n, 100)
    }
}
