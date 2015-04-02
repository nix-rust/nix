use nix::sys::socket::{InetAddr, UnixAddr, getsockname};
use std::{mem, net};
use std::path::Path;
use std::str::FromStr;
use std::os::unix::io::AsRawFd;
use ports::localhost;

#[test]
pub fn test_inetv4_addr_to_sock_addr() {
    let actual: net::SocketAddr = FromStr::from_str("127.0.0.1:3000").unwrap();
    let addr = InetAddr::from_std(&actual);

    match addr {
        InetAddr::V4(addr) => {
            let ip: u32 = 0x7f000001;
            let port: u16 = 3000;

            assert_eq!(addr.sin_addr.s_addr, ip.to_be());
            assert_eq!(addr.sin_port, port.to_be());
        }
        _ => panic!("nope"),
    }

    assert_eq!(addr.to_str(), "127.0.0.1:3000");

    let inet = addr.to_std();
    assert_eq!(actual, inet);
}

#[test]
pub fn test_path_to_sock_addr() {
    let actual = Path::new("/foo/bar");
    let addr = UnixAddr::new(actual).unwrap();

    let expect: &'static [i8] = unsafe { mem::transmute(&b"/foo/bar"[..]) };
    assert_eq!(&addr.0.sun_path[..8], expect);

    assert_eq!(addr.path(), actual);
}

#[test]
pub fn test_getsockname() {
    use std::net::TcpListener;

    let addr = localhost();
    let sock = TcpListener::bind(&*addr).unwrap();
    let res = getsockname(sock.as_raw_fd()).unwrap();

    assert_eq!(addr, res.to_str());
}
