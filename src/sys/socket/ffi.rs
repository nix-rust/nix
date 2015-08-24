// Silence invalid warnings due to rust-lang/rust#16719
#![allow(improper_ctypes)]

use libc::{c_int, c_void, socklen_t, ssize_t};
pub use libc::{socket, listen, bind, accept, connect, setsockopt, sendto, recvfrom, getsockname, getpeername, recv, send};
use super::msghdr;

extern {
    pub fn getsockopt(
        sockfd: c_int,
        level: c_int,
        optname: c_int,
        optval: *mut c_void,
        optlen: *mut socklen_t) -> c_int;

    pub fn socketpair(
        domain:     c_int,
        typ:        c_int,
        protocol:   c_int,
        sv:         *mut c_int
    ) -> c_int;

    pub fn sendmsg(sockfd: c_int, msg: *const msghdr, flags: c_int) -> ssize_t;
    pub fn recvmsg(sockfd: c_int, msg: *mut msghdr, flags: c_int) -> ssize_t;
}
