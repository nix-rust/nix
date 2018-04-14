use nix::sys::socket::{InetAddr, UnixAddr, getsockname};
use std::slice;
use std::net::{self, Ipv6Addr, SocketAddr, SocketAddrV6};
use std::path::Path;
use std::str::FromStr;
use std::os::unix::io::RawFd;
use libc::c_char;

#[test]
pub fn test_inetv4_addr_to_sock_addr() {
    let actual: net::SocketAddr = FromStr::from_str("127.0.0.1:3000").unwrap();
    let addr = InetAddr::from_std(&actual);

    match addr {
        InetAddr::V4(addr) => {
            let ip: u32 = 0x7f00_0001;
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
pub fn test_inetv6_addr_to_sock_addr() {
    let port: u16 = 3000;
    let flowinfo: u32 = 1;
    let scope_id: u32 = 2;
    let ip: Ipv6Addr = "fe80::1".parse().unwrap();

    let actual = SocketAddr::V6(SocketAddrV6::new(ip, port, flowinfo, scope_id));
    let addr = InetAddr::from_std(&actual);

    match addr {
        InetAddr::V6(addr) => {
            assert_eq!(addr.sin6_port, port.to_be());
            assert_eq!(addr.sin6_flowinfo, flowinfo);
            assert_eq!(addr.sin6_scope_id, scope_id);
        }
        _ => panic!("nope"),
    }

    assert_eq!(actual, addr.to_std());
}

#[test]
pub fn test_path_to_sock_addr() {
    let path = "/foo/bar";
    let actual = Path::new(path);
    let addr = UnixAddr::new(actual).unwrap();

    let expect: &[c_char] = unsafe {
        slice::from_raw_parts(path.as_bytes().as_ptr() as *const c_char, path.len())
    };
    assert_eq!(&addr.0.sun_path[..8], expect);

    assert_eq!(addr.path(), Some(actual));
}

// Test getting/setting abstract addresses (without unix socket creation)
#[cfg(target_os = "linux")]
#[test]
pub fn test_abstract_uds_addr() {
    let empty = String::new();
    let addr = UnixAddr::new_abstract(empty.as_bytes()).unwrap();
    assert_eq!(addr.as_abstract(), Some(empty.as_bytes()));

    let name = String::from("nix\0abstract\0test");
    let addr = UnixAddr::new_abstract(name.as_bytes()).unwrap();
    assert_eq!(addr.as_abstract(), Some(name.as_bytes()));
    assert_eq!(addr.path(), None);

    // Internally, name is null-prefixed (abstract namespace)
    let internal: &[u8] = unsafe {
        slice::from_raw_parts(addr.0.sun_path.as_ptr() as *const u8, addr.1)
    };
    let mut abstract_name = name.clone();
    abstract_name.insert(0, '\0');
    assert_eq!(internal, abstract_name.as_bytes());
}

#[test]
pub fn test_getsockname() {
    use nix::sys::socket::{socket, AddressFamily, SockType, SockFlag};
    use nix::sys::socket::{bind, SockAddr};
    use tempdir::TempDir;

    let tempdir = TempDir::new("test_getsockname").unwrap();
    let sockname = tempdir.path().join("sock");
    let sock = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None)
               .expect("socket failed");
    let sockaddr = SockAddr::new_unix(&sockname).unwrap();
    bind(sock, &sockaddr).expect("bind failed");
    assert_eq!(sockaddr.to_str(),
               getsockname(sock).expect("getsockname failed").to_str());
}

#[test]
pub fn test_socketpair() {
    use nix::unistd::{read, write};
    use nix::sys::socket::{socketpair, AddressFamily, SockType, SockFlag};

    let (fd1, fd2) = socketpair(AddressFamily::Unix, SockType::Stream, None, SockFlag::empty())
                     .unwrap();
    write(fd1, b"hello").unwrap();
    let mut buf = [0;5];
    read(fd2, &mut buf).unwrap();

    assert_eq!(&buf[..], b"hello");
}

#[test]
pub fn test_scm_rights() {
    use nix::sys::uio::IoVec;
    use nix::unistd::{pipe, read, write, close};
    use nix::sys::socket::{socketpair, sendmsg, recvmsg,
                           AddressFamily, SockType, SockFlag,
                           ControlMessage, CmsgSpace, MsgFlags};

    let (fd1, fd2) = socketpair(AddressFamily::Unix, SockType::Stream, None, SockFlag::empty())
                     .unwrap();
    let (r, w) = pipe().unwrap();
    let mut received_r: Option<RawFd> = None;

    {
        let iov = [IoVec::from_slice(b"hello")];
        let fds = [r];
        let cmsg = ControlMessage::ScmRights(&fds);
        assert_eq!(sendmsg(fd1, &iov, &[cmsg], MsgFlags::empty(), None).unwrap(), 5);
        close(r).unwrap();
        close(fd1).unwrap();
    }

    {
        let mut buf = [0u8; 5];
        let iov = [IoVec::from_mut_slice(&mut buf[..])];
        let mut cmsgspace: CmsgSpace<[RawFd; 1]> = CmsgSpace::new();
        let msg = recvmsg(fd2, &iov, Some(&mut cmsgspace), MsgFlags::empty()).unwrap();

        for cmsg in msg.cmsgs() {
            if let ControlMessage::ScmRights(fd) = cmsg {
                assert_eq!(received_r, None);
                assert_eq!(fd.len(), 1);
                received_r = Some(fd[0]);
            } else {
                panic!("unexpected cmsg");
            }
        }
        assert!(!msg.flags.intersects(MsgFlags::MSG_TRUNC | MsgFlags::MSG_CTRUNC));
        close(fd2).unwrap();
    }

    let received_r = received_r.expect("Did not receive passed fd");
    // Ensure that the received file descriptor works
    write(w, b"world").unwrap();
    let mut buf = [0u8; 5];
    read(received_r, &mut buf).unwrap();
    assert_eq!(&buf[..], b"world");
    close(received_r).unwrap();
    close(w).unwrap();
}

// Verify `sendmsg` builds a valid `msghdr` when passing an empty
// `cmsgs` argument.  This should result in a msghdr with a nullptr
// msg_control field and a msg_controllen of 0 when calling into the
// raw `sendmsg`.
#[test]
pub fn test_sendmsg_empty_cmsgs() {
    use nix::sys::uio::IoVec;
    use nix::unistd::close;
    use nix::sys::socket::{socketpair, sendmsg, recvmsg,
                           AddressFamily, SockType, SockFlag,
                           CmsgSpace, MsgFlags};

    let (fd1, fd2) = socketpair(AddressFamily::Unix, SockType::Stream, None, SockFlag::empty())
                     .unwrap();

    {
        let iov = [IoVec::from_slice(b"hello")];
        assert_eq!(sendmsg(fd1, &iov, &[], MsgFlags::empty(), None).unwrap(), 5);
        close(fd1).unwrap();
    }

    {
        let mut buf = [0u8; 5];
        let iov = [IoVec::from_mut_slice(&mut buf[..])];
        let mut cmsgspace: CmsgSpace<[RawFd; 1]> = CmsgSpace::new();
        let msg = recvmsg(fd2, &iov, Some(&mut cmsgspace), MsgFlags::empty()).unwrap();

        for _ in msg.cmsgs() {
            panic!("unexpected cmsg");
        }
        assert!(!msg.flags.intersects(MsgFlags::MSG_TRUNC | MsgFlags::MSG_CTRUNC));
        close(fd2).unwrap();
    }
}

// Test creating and using named unix domain sockets
#[test]
pub fn test_unixdomain() {
    use nix::sys::socket::{AddressFamily, SockType, SockFlag};
    use nix::sys::socket::{bind, socket, connect, listen, accept, SockAddr};
    use nix::unistd::{read, write, close};
    use std::thread;
    use tempdir::TempDir;

    let tempdir = TempDir::new("test_unixdomain").unwrap();
    let sockname = tempdir.path().join("sock");
    let s1 = socket(AddressFamily::Unix, SockType::Stream,
                    SockFlag::empty(), None).expect("socket failed");
    let sockaddr = SockAddr::new_unix(&sockname).unwrap();
    bind(s1, &sockaddr).expect("bind failed");
    listen(s1, 10).expect("listen failed");

    let thr = thread::spawn(move || {
        let s2 = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None)
                 .expect("socket failed");
        connect(s2, &sockaddr).expect("connect failed");
        write(s2, b"hello").expect("write failed");
        close(s2).unwrap();
    });

    let s3 = accept(s1).expect("accept failed");

    let mut buf = [0;5];
    read(s3, &mut buf).unwrap();
    close(s3).unwrap();
    close(s1).unwrap();
    thr.join().unwrap();

    assert_eq!(&buf[..], b"hello");
}

// Test creating and using named system control sockets
#[cfg(any(target_os = "macos", target_os = "ios"))]
#[test]
pub fn test_syscontrol() {
    use nix::Error;
    use nix::errno::Errno;
    use nix::sys::socket::{AddressFamily, socket, SockAddr, SockType, SockFlag, SockProtocol};

    let fd = socket(AddressFamily::System, SockType::Datagram,
                    SockFlag::empty(), SockProtocol::KextControl)
             .expect("socket failed");
    let _sockaddr = SockAddr::new_sys_control(fd, "com.apple.net.utun_control", 0).expect("resolving sys_control name failed");
    assert_eq!(SockAddr::new_sys_control(fd, "foo.bar.lol", 0).err(), Some(Error::Sys(Errno::ENOENT)));

    // requires root privileges
    // connect(fd, &sockaddr).expect("connect failed");
}
