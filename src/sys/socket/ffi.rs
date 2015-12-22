// Silence invalid warnings due to rust-lang/rust#16719
#![allow(improper_ctypes)]

pub use libc::{socket, listen, bind, accept, connect, setsockopt, sendto, recvfrom, getsockname, getpeername, recv, send};

use libc::{c_int, c_void, socklen_t, size_t, ssize_t};
use sys::uio::IoVec;


#[cfg(target_os = "linux")]
pub type type_of_cmsg_len = size_t;

#[cfg(not(target_os = "linux"))]
pub type type_of_cmsg_len = socklen_t;

// Private because we don't expose any external functions that operate
// directly on this type; we just use it internally at FFI boundaries.
// Note that in some cases we store pointers in *const fields that the
// kernel will proceed to mutate, so users should be careful about the
// actual mutability of data pointed to by this structure.
#[repr(C)]
pub struct msghdr<'a> {
    pub msg_name: *const c_void,
    pub msg_namelen: socklen_t,
    pub msg_iov: *const IoVec<&'a [u8]>,
    pub msg_iovlen: size_t,
    pub msg_control: *const c_void,
    pub msg_controllen: size_t,
    pub msg_flags: c_int,
}

// As above, private because we don't expose any external functions that
// operate directly on this type, or any external types with a public
// cmsghdr member.
#[repr(C)]
pub struct cmsghdr {
    pub cmsg_len: type_of_cmsg_len,
    pub cmsg_level: c_int,
    pub cmsg_type: c_int,
    pub cmsg_data: [type_of_cmsg_len; 0]
}

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
