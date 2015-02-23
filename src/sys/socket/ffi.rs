use libc::{c_int, c_void, socklen_t};
pub use libc::{socket, listen, bind, accept, connect, setsockopt, sendto, recvfrom, getsockname, getpeername};

extern {
    pub fn getsockopt(
        sockfd: c_int,
        level: c_int,
        optname: c_int,
        optval: *mut c_void,
        optlen: *mut socklen_t) -> c_int;
}
