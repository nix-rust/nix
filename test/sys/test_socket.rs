use nix::sys::socket::{SockAddr, ToSockAddr, FromSockAddr, getsockname};
use std::{mem, net};
use std::num::Int;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::os::unix::AsRawFd;
use ports::localhost;

#[test]
pub fn test_inetv4_addr_to_sock_addr() {
    let actual: net::SocketAddr = FromStr::from_str("127.0.0.1:3000").unwrap();
    let addr = actual.to_sock_addr().unwrap();

    match addr {
        SockAddr::IpV4(addr) => {
            assert_eq!(addr.sin_addr.s_addr, 0x7f000001.to_be());
            assert_eq!(addr.sin_port, 3000.to_be());
        }
        _ => panic!("nope"),
    }

    assert_eq!(addr.to_str(), "127.0.0.1:3000");

    let inet = FromSockAddr::from_sock_addr(&addr).unwrap();
    assert_eq!(actual, inet);
}

#[test]
pub fn test_path_to_sock_addr() {
    let actual = Path::new("/foo/bar");
    let addr = actual.to_sock_addr().unwrap();

    match addr {
        SockAddr::Unix(addr) => {
            let expect: &'static [i8] = unsafe { mem::transmute(b"/foo/bar") };
            assert_eq!(&addr.sun_path[..8], expect);
        }
        _ => panic!("nope"),
    }

    let path: PathBuf = FromSockAddr::from_sock_addr(&addr).unwrap();
    assert_eq!(actual, &*path);
}

#[test]
pub fn test_getsockname() {
    use std::net::TcpListener;

    let addr = localhost();
    let sock = TcpListener::bind(&*addr).unwrap();
    let res = getsockname(sock.as_raw_fd()).unwrap();

    assert_eq!(addr, res.to_str());
}
