use libc::c_int;
use {Errno, Result};

pub use self::ffi::PollFd;
pub use self::ffi::consts::*;

mod ffi {
    use libc::c_int;
    pub use self::consts::*;

    #[derive(Clone, Copy, Debug)]
    #[repr(C)]
    pub struct PollFd {
        pub fd: c_int,
        pub events: EventFlags,
        pub revents: EventFlags
    }

    #[cfg(target_os = "linux")]
    pub mod consts {
        use libc::{c_short, c_ulong};

        bitflags! {
            flags EventFlags: c_short {
                const POLLIN     = 0x001,
                const POLLPRI    = 0x002,
                const POLLOUT    = 0x004,
                const POLLRDNORM = 0x040,
                const POLLWRNORM = 0x100,
                const POLLRDBAND = 0x080,
                const POLLWRBAND = 0x200,
                const POLLERR    = 0x008,
                const POLLHUP    = 0x010,
                const POLLNVAL   = 0x020,
            }
        }

        pub type nfds_t = c_ulong;
    }

    #[cfg(target_os = "macos")]
    pub mod consts {
        use libc::{c_short, c_uint};

        bitflags! {
            flags EventFlags: c_short {
                const POLLIN     = 0x0001,
                const POLLPRI    = 0x0002,
                const POLLOUT    = 0x0004,
                const POLLRDNORM = 0x0040,
                const POLLWRNORM = 0x0004,
                const POLLRDBAND = 0x0080,
                const POLLWRBAND = 0x0100,
                const POLLERR    = 0x0008,
                const POLLHUP    = 0x0010,
                const POLLNVAL   = 0x0020,
            }
        }

        pub type nfds_t = c_uint;
    }

    #[allow(improper_ctypes)]
    extern {
        pub fn poll(fds: *mut PollFd, nfds: nfds_t, timeout: c_int) -> c_int;
    }
}

pub fn poll(fds: &mut [PollFd], timeout: c_int) -> Result<c_int> {
    let res = unsafe {
        ffi::poll(fds.as_mut_ptr(), fds.len() as ffi::nfds_t, timeout)
    };

    Errno::result(res)
}
