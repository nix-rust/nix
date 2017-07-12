#[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd", target_os = "linux"))]
use sys::time::TimeSpec;
#[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd", target_os = "linux"))]
use sys::signal::SigSet;

use libc;
use {Errno, Result};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PollFd {
    pollfd: libc::pollfd,
}

impl PollFd {
    pub fn new(fd: libc::c_int, events: EventFlags) -> PollFd {
        PollFd {
            pollfd: libc::pollfd {
                fd: fd,
                events: events.bits(),
                revents: EventFlags::empty().bits(),
            },
        }
    }

    pub fn revents(&self) -> Option<EventFlags> {
        EventFlags::from_bits(self.pollfd.revents)
    }
}

libc_bitflags! {
    pub flags EventFlags: libc::c_short {
        POLLIN,
        POLLPRI,
        POLLOUT,
        POLLRDNORM,
        POLLWRNORM,
        POLLRDBAND,
        POLLWRBAND,
        POLLERR,
        POLLHUP,
        POLLNVAL,
    }
}

pub fn poll(fds: &mut [PollFd], timeout: libc::c_int) -> Result<libc::c_int> {
    let res = unsafe {
        libc::poll(fds.as_mut_ptr() as *mut libc::pollfd,
                   fds.len() as libc::nfds_t,
                   timeout)
    };

    Errno::result(res)
}

#[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd", target_os = "linux"))]
pub fn ppoll(fds: &mut [PollFd], timeout: TimeSpec, sigmask: SigSet) -> Result<libc::c_int> {


    let res = unsafe {
        libc::ppoll(fds.as_mut_ptr() as *mut libc::pollfd,
                    fds.len() as libc::nfds_t,
                    timeout.as_ref(),
                    sigmask.as_ref())
    };
    Errno::result(res)
}
