//! Wrapper around getrandom.

use crate::Result;
use libc;

libc_enum! {
    /// How the random bytes should be filled
    #[repr(u32)]
    #[non_exhaustive]
    pub enum RandomMode{
        /// If  this  bit is set, then random bytes are drawn from the
        /// random source (i.e., the same source as the /dev/random device)
        ///  instead of the urandom source.
        GRND_RANDOM,
        /// By default, when reading from the random source, getrandom()
        /// blocks if no random bytes are available, and when reading
        /// from the urandom source, it blocks if the entropy pool has
        /// not yet been initialized. If the GRND_NONBLOCK flag is
        /// set, then getrandom() does not block in these cases
        GRND_NONBLOCK,
    }
}

/// Returns a vector of random bytes
pub fn getrandom(size: usize, mode: RandomMode) -> Result<Vec<u8>> {
    let mut buffer = vec![0; size];
    unsafe {
        assert_eq!(
            libc::getrandom(
                buffer.as_mut_ptr() as *mut libc::c_void,
                size,
                mode as u32,
            ),
            size as isize
        )
    };
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_getrandom() {
        let random_bytes = getrandom(1000, RandomMode::GRND_RANDOM).unwrap();
        assert_eq!(random_bytes.len(), 1000)
    }
}
