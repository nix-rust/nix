use nix::sys::socket::{AsSockAddr, SockAddr};
use std::{mem, net};
use std::num::Int;
use std::path::Path;
use std::str::FromStr;

#[test]
pub fn test_inetv4_addr_to_sock_addr() {
    let std: net::SocketAddr = FromStr::from_str("127.0.0.1:3000").unwrap();

    match std.as_sock_addr().unwrap() {
        SockAddr::SockIpV4(addr) => {
            assert_eq!(addr.sin_addr.s_addr, Int::from_be(2130706433));
            assert_eq!(addr.sin_port, 3000);
        }
        _ => panic!("nope"),
    }
}

#[test]
pub fn test_path_to_sock_addr() {
    match Path::new("/foo/bar").as_sock_addr().unwrap() {
        SockAddr::SockUnix(addr) => {
            let expect: &'static [i8] = unsafe { mem::transmute(b"/foo/bar") };
            assert_eq!(&addr.sun_path[..8], expect);
        }
        _ => panic!("nope"),
    }
}
