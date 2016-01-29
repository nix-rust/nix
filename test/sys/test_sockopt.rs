use rand::{thread_rng, Rng};
use nix::sys::socket::{socket, sockopt, getsockopt, setsockopt, AddressFamily, SockType, SockFlag, SockLevel};

#[test]
fn test_so_buf() {
    let fd = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), SockLevel::Udp as i32).unwrap();
    let bufsize: usize = thread_rng().gen_range(4096, 131072);
    setsockopt(fd, sockopt::SndBuf, &bufsize).unwrap();
    let actual = getsockopt(fd, sockopt::SndBuf).unwrap();
    assert!(actual >= bufsize);
    setsockopt(fd, sockopt::RcvBuf, &bufsize).unwrap();
    let actual = getsockopt(fd, sockopt::RcvBuf).unwrap();
    assert!(actual >= bufsize);
}
