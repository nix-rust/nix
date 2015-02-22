use nix::sys::socket::{SockAddr, ToSockAddr, FromSockAddr};
use std::{mem, net};
use std::num::Int;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[test]
pub fn test_inetv4_addr_to_sock_addr() {
    let actual: net::SocketAddr = FromStr::from_str("127.0.0.1:3000").unwrap();
    let addr = actual.to_sock_addr().unwrap();

    match addr {
        SockAddr::SockIpV4(addr) => {
            assert_eq!(addr.sin_addr.s_addr, Int::from_be(2130706433));
            assert_eq!(addr.sin_port, 3000);
        }
        _ => panic!("nope"),
    }

    let inet = FromSockAddr::from_sock_addr(&addr).unwrap();
    assert_eq!(actual, inet);
}

#[test]
pub fn test_path_to_sock_addr() {
    let actual = Path::new("/foo/bar");
    let addr = actual.to_sock_addr().unwrap();

    match addr {
        SockAddr::SockUnix(addr) => {
            let expect: &'static [i8] = unsafe { mem::transmute(b"/foo/bar") };
            assert_eq!(&addr.sun_path[..8], expect);
        }
        _ => panic!("nope"),
    }

    let path: PathBuf = FromSockAddr::from_sock_addr(&addr).unwrap();
    assert_eq!(actual, &*path);
}
