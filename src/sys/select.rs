use std::ptr::null_mut;
use std::os::unix::io::RawFd;
use libc::{c_int, timeval};
use {Errno, Result};
use sys::time::TimeVal;

pub const FD_SETSIZE: RawFd = 1024;

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[repr(C)]
#[derive(Clone)]
pub struct FdSet {
    bits: [i32; FD_SETSIZE as usize / 32]
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
const BITS: usize = 32;

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
#[repr(C)]
#[derive(Clone)]
pub struct FdSet {
    bits: [u64; FD_SETSIZE as usize / 64]
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
const BITS: usize = 64;

impl FdSet {
    pub fn new() -> FdSet {
        FdSet {
            bits: [0; FD_SETSIZE as usize / BITS]
        }
    }

    pub fn insert(&mut self, fd: RawFd) {
        let fd = fd as usize;
        self.bits[fd / BITS] |= 1 << (fd % BITS);
    }

    pub fn remove(&mut self, fd: RawFd) {
        let fd = fd as usize;
        self.bits[fd / BITS] &= !(1 << (fd % BITS));
    }

    pub fn contains(&self, fd: RawFd) -> bool {
        let fd = fd as usize;
        self.bits[fd / BITS] & (1 << (fd % BITS)) > 0
    }

    pub fn clear(&mut self) {
        for bits in &mut self.bits {
            *bits = 0
        }
    }

    /// Finds the highest file descriptor in the set.
    ///
    /// Returns `None` if the set is empty.
    ///
    /// This can be used to calculate the `nfds` parameter of the [`select`] function.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate nix;
    /// # use nix::sys::select::FdSet;
    /// # fn main() {
    /// let mut set = FdSet::new();
    /// set.insert(4);
    /// set.insert(9);
    /// assert_eq!(set.highest(), Some(9));
    /// # }
    /// ```
    ///
    /// [`select`]: fn.select.html
    pub fn highest(&self) -> Option<RawFd> {
        for (i, &block) in self.bits.iter().enumerate().rev() {
            if block != 0 {
                // Highest bit is located at `BITS - 1 - n.leading_zeros()`. Examples:
                // 0b00000001
                // 7 leading zeros, result should be 0 (bit at index 0 is 1)
                // 0b001xxxxx
                // 2 leading zeros, result should be 5 (bit at index 5 is 1) - x may be 0 or 1

                return Some((i * BITS + BITS - 1 - block.leading_zeros() as usize) as RawFd);
            }
        }

        None
    }
}

mod ffi {
    use libc::{c_int, timeval};
    use super::FdSet;

    extern {
        pub fn select(nfds: c_int,
                      readfds: *mut FdSet,
                      writefds: *mut FdSet,
                      errorfds: *mut FdSet,
                      timeout: *mut timeval) -> c_int;
    }
}

/// Monitors file descriptors for readiness (see [select(2)]).
///
/// Returns the total number of ready file descriptors in all sets. The sets are changed so that all
/// file descriptors that are ready for the given operation are set.
///
/// When this function returns, `timeout` has an implementation-defined value.
///
/// # Parameters
///
/// * `nfds`: The highest file descriptor set in any of the passed `FdSet`s, plus 1. If `None`, this
///   is calculated automatically by calling [`FdSet::highest`] on all descriptor sets and adding 1
///   to the maximum of that.
/// * `readfds`: File descriptors to check for being ready to read.
/// * `writefds`: File descriptors to check for being ready to write.
/// * `errorfds`: File descriptors to check for pending error conditions.
/// * `timeout`: Maximum time to wait for descriptors to become ready (`None` to block
///   indefinitely).
///
/// [select(2)]: http://man7.org/linux/man-pages/man2/select.2.html
/// [`FdSet::highest`]: struct.FdSet.html#method.highest
pub fn select<'a, N, R, W, E, T>(nfds: N,
                                 readfds: R,
                                 writefds: W,
                                 errorfds: E,
                                 timeout: T) -> Result<c_int>
where
    N: Into<Option<c_int>>,
    R: Into<Option<&'a mut FdSet>>,
    W: Into<Option<&'a mut FdSet>>,
    E: Into<Option<&'a mut FdSet>>,
    T: Into<Option<&'a mut TimeVal>>,
{
    let readfds = readfds.into();
    let writefds = writefds.into();
    let errorfds = errorfds.into();
    let timeout = timeout.into();

    let nfds = nfds.into().unwrap_or_else(|| {
        readfds.iter()
            .chain(writefds.iter())
            .chain(errorfds.iter())
            .map(|set| set.highest().unwrap_or(-1))
            .max()
            .unwrap_or(-1) + 1
    });

    let readfds = readfds.map(|set| set as *mut FdSet).unwrap_or(null_mut());
    let writefds = writefds.map(|set| set as *mut FdSet).unwrap_or(null_mut());
    let errorfds = errorfds.map(|set| set as *mut FdSet).unwrap_or(null_mut());
    let timeout = timeout.map(|tv| tv as *mut TimeVal as *mut timeval)
                         .unwrap_or(null_mut());

    let res = unsafe {
        ffi::select(nfds, readfds, writefds, errorfds, timeout)
    };

    Errno::result(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sys::time::{TimeVal, TimeValLike};
    use unistd::{write, pipe};

    #[test]
    fn fdset_insert() {
        let mut fd_set = FdSet::new();

        for i in 0..FD_SETSIZE {
            assert!(!fd_set.contains(i));
        }

        fd_set.insert(7);

        assert!(fd_set.contains(7));
    }

    #[test]
    fn fdset_remove() {
        let mut fd_set = FdSet::new();

        for i in 0..FD_SETSIZE {
            assert!(!fd_set.contains(i));
        }

        fd_set.insert(7);
        fd_set.remove(7);

        for i in 0..FD_SETSIZE {
            assert!(!fd_set.contains(i));
        }
    }

    #[test]
    fn fdset_clear() {
        let mut fd_set = FdSet::new();
        fd_set.insert(1);
        fd_set.insert(FD_SETSIZE / 2);
        fd_set.insert(FD_SETSIZE - 1);

        fd_set.clear();

        for i in 0..FD_SETSIZE {
            assert!(!fd_set.contains(i));
        }
    }

    #[test]
    fn fdset_highest() {
        let mut set = FdSet::new();
        assert_eq!(set.highest(), None);
        set.insert(0);
        assert_eq!(set.highest(), Some(0));
        set.insert(90);
        assert_eq!(set.highest(), Some(90));
        set.remove(0);
        assert_eq!(set.highest(), Some(90));
        set.remove(90);
        assert_eq!(set.highest(), None);

        set.insert(4);
        set.insert(5);
        set.insert(7);
        assert_eq!(set.highest(), Some(7));
    }

    // powerpc-unknown-linux-gnu currently fails on the first `assert_eq` because
    // `select()` returns a 0 instead of a 1. Since this test has only been run on
    // qemu, it's unclear if this is a OS or qemu bug. Just disable it on that arch
    // for now.
    // FIXME: Fix tests for powerpc and mips
    // FIXME: Add a link to an upstream qemu bug if there is one
    #[test]
    #[cfg_attr(any(target_arch = "powerpc", target_arch = "mips"), ignore)]
    fn test_select() {
        let (r1, w1) = pipe().unwrap();
        write(w1, b"hi!").unwrap();
        let (r2, _w2) = pipe().unwrap();

        let mut fd_set = FdSet::new();
        fd_set.insert(r1);
        fd_set.insert(r2);

        let mut timeout = TimeVal::seconds(10);
        assert_eq!(1, select(None,
                             &mut fd_set,
                             None,
                             None,
                             &mut timeout).unwrap());
        assert!(fd_set.contains(r1));
        assert!(!fd_set.contains(r2));
    }

    #[test]
    #[cfg_attr(any(target_arch = "powerpc", target_arch = "mips"), ignore)]
    fn test_select_nfds() {
        let (r1, w1) = pipe().unwrap();
        write(w1, b"hi!").unwrap();
        let (r2, _w2) = pipe().unwrap();

        let mut fd_set = FdSet::new();
        fd_set.insert(r1);
        fd_set.insert(r2);

        let mut timeout = TimeVal::seconds(10);
        assert_eq!(1, select(Some(fd_set.highest().unwrap() + 1),
                             &mut fd_set,
                             None,
                             None,
                             &mut timeout).unwrap());
        assert!(fd_set.contains(r1));
        assert!(!fd_set.contains(r2));
    }

    #[test]
    #[cfg_attr(any(target_arch = "powerpc", target_arch = "mips"), ignore)]
    fn test_select_nfds2() {
        let (r1, w1) = pipe().unwrap();
        write(w1, b"hi!").unwrap();
        let (r2, _w2) = pipe().unwrap();

        let mut fd_set = FdSet::new();
        fd_set.insert(r1);
        fd_set.insert(r2);

        let mut timeout = TimeVal::seconds(10);
        assert_eq!(1, select(::std::cmp::max(r1, r2) + 1,
                             &mut fd_set,
                             None,
                             None,
                             &mut timeout).unwrap());
        assert!(fd_set.contains(r1));
        assert!(!fd_set.contains(r2));
    }
}
