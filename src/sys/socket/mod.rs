//! Socket interface functions
//!
//! [Further reading](http://man7.org/linux/man-pages/man7/socket.7.html)
use {Error, Errno, Result};
use features;
use fcntl::{fcntl, FD_CLOEXEC, O_NONBLOCK};
use fcntl::FcntlArg::{F_SETFD, F_SETFL};
use libc::{c_void, c_int, socklen_t, size_t, pid_t, uid_t, gid_t};
use std::{mem, ptr, slice};
use std::os::unix::io::RawFd;
use sys::uio::IoVec;

mod addr;
mod consts;
mod ffi;
mod multicast;
pub mod sockopt;

/*
 *
 * ===== Re-exports =====
 *
 */

pub use self::addr::{
    AddressFamily,
    SockAddr,
    InetAddr,
    UnixAddr,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
};
#[cfg(any(target_os = "linux", target_os = "android"))]
pub use ::sys::socket::addr::netlink::NetlinkAddr;

pub use libc::{
    in_addr,
    in6_addr,
    sockaddr,
    sockaddr_in,
    sockaddr_in6,
    sockaddr_un,
    sa_family_t,
};

pub use self::multicast::{
    ip_mreq,
    ipv6_mreq,
};
pub use self::consts::*;

#[cfg(any(not(target_os = "linux"), not(target_arch = "x86")))]
pub use libc::sockaddr_storage;

// Working around rust-lang/rust#23425
#[cfg(all(target_os = "linux", target_arch = "x86"))]
pub struct sockaddr_storage {
    pub ss_family: sa_family_t,
    pub __ss_align: u32,
    pub __ss_pad2: [u8; 120],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum SockType {
    Stream = consts::SOCK_STREAM,
    Datagram = consts::SOCK_DGRAM,
    SeqPacket = consts::SOCK_SEQPACKET,
    Raw = consts::SOCK_RAW,
    Rdm = consts::SOCK_RDM,
}

// Extra flags - Supported by Linux 2.6.27, normalized on other platforms
bitflags!(
    flags SockFlag: c_int {
        const SOCK_NONBLOCK = 0o0004000,
        const SOCK_CLOEXEC  = 0o2000000
    }
);

/// Copy the in-memory representation of src into the byte slice dst,
/// updating the slice to point to the remainder of dst only. Unsafe
/// because it exposes all bytes in src, which may be UB if some of them
/// are uninitialized (including padding).
unsafe fn copy_bytes<'a, 'b, T: ?Sized>(src: &T, dst: &'a mut &'b mut [u8]) {
    let srclen = mem::size_of_val(src);
    let mut tmpdst = &mut [][..];
    mem::swap(&mut tmpdst, dst);
    let (target, mut remainder) = tmpdst.split_at_mut(srclen);
    // Safe because the mutable borrow of dst guarantees that src does not alias it.
    ptr::copy_nonoverlapping(src as *const T as *const u8, target.as_mut_ptr(), srclen);
    mem::swap(dst, &mut remainder);
}


use self::ffi::{cmsghdr, msghdr, type_of_cmsg_len};

/// A structure used to make room in a cmsghdr passed to recvmsg. The
/// size and alignment match that of a cmsghdr followed by a T, but the
/// fields are not accessible, as the actual types will change on a call
/// to recvmsg.
///
/// To make room for multiple messages, nest the type parameter with
/// tuples, e.g.
/// `let cmsg: CmsgSpace<([RawFd; 3], CmsgSpace<[RawFd; 2]>)> = CmsgSpace::new();`
pub struct CmsgSpace<T> {
    _hdr: cmsghdr,
    _data: T,
}

impl<T> CmsgSpace<T> {
    /// Create a CmsgSpace<T>. The structure is used only for space, so
    /// the fields are uninitialized.
    pub fn new() -> Self {
        // Safe because the fields themselves aren't accessible.
        unsafe { mem::uninitialized() }
    }
}

pub struct RecvMsg<'a> {
    // The number of bytes received.
    pub bytes: usize,
    cmsg_buffer: &'a [u8],
    pub address: Option<SockAddr>,
    pub flags: MsgFlags,
}

impl<'a> RecvMsg<'a> {
    /// Iterate over the valid control messages pointed to by this
    /// msghdr.
    pub fn cmsgs(&self) -> CmsgIterator {
        CmsgIterator(self.cmsg_buffer)
    }
}

pub struct CmsgIterator<'a>(&'a [u8]);

impl<'a> Iterator for CmsgIterator<'a> {
    type Item = ControlMessage<'a>;

    // The implementation loosely follows CMSG_FIRSTHDR / CMSG_NXTHDR,
    // although we handle the invariants in slightly different places to
    // get a better iterator interface.
    fn next(&mut self) -> Option<ControlMessage<'a>> {
        let buf = self.0;
        let sizeof_cmsghdr = mem::size_of::<cmsghdr>();
        if buf.len() < sizeof_cmsghdr {
            return None;
        }
        let cmsg: &cmsghdr = unsafe { mem::transmute(buf.as_ptr()) };

        // This check is only in the glibc implementation of CMSG_NXTHDR
        // (although it claims the kernel header checks this), but such
        // a structure is clearly invalid, either way.
        let cmsg_len = cmsg.cmsg_len as usize;
        if cmsg_len < sizeof_cmsghdr {
            return None;
        }
        let len = cmsg_len - sizeof_cmsghdr;

        // Advance our internal pointer.
        if cmsg_align(cmsg_len) > buf.len() {
            return None;
        }
        self.0 = &buf[cmsg_align(cmsg_len)..];

        match (cmsg.cmsg_level, cmsg.cmsg_type) {
            (SOL_SOCKET, SCM_RIGHTS) => unsafe {
                Some(ControlMessage::ScmRights(
                    slice::from_raw_parts(
                        &cmsg.cmsg_data as *const _ as *const _,
                        len / mem::size_of::<RawFd>())))
            },
            (_, _) => unsafe {
                Some(ControlMessage::Unknown(UnknownCmsg(
                    &cmsg,
                    slice::from_raw_parts(
                        &cmsg.cmsg_data as *const _ as *const _,
                        len))))
            }
        }
    }
}

/// A type-safe wrapper around a single control message. More types may
/// be added to this enum; do not exhaustively pattern-match it.
/// [Further reading](http://man7.org/linux/man-pages/man3/cmsg.3.html)
pub enum ControlMessage<'a> {
    /// A message of type SCM_RIGHTS, containing an array of file
    /// descriptors passed between processes. See the description in the
    /// "Ancillary messages" section of the
    /// [unix(7) man page](http://man7.org/linux/man-pages/man7/unix.7.html).
    ScmRights(&'a [RawFd]),
    #[doc(hidden)]
    Unknown(UnknownCmsg<'a>),
}

// An opaque structure used to prevent cmsghdr from being a public type
#[doc(hidden)]
pub struct UnknownCmsg<'a>(&'a cmsghdr, &'a [u8]);

fn cmsg_align(len: usize) -> usize {
    let round_to = mem::size_of::<type_of_cmsg_len>();
    if len % round_to == 0 {
        len
    } else {
        len + round_to - (len % round_to)
    }
}

impl<'a> ControlMessage<'a> {
    /// The value of CMSG_SPACE on this message.
    fn space(&self) -> usize {
        cmsg_align(self.len())
    }

    /// The value of CMSG_LEN on this message.
    fn len(&self) -> usize {
        mem::size_of::<cmsghdr>() + match *self {
            ControlMessage::ScmRights(fds) => {
                mem::size_of_val(fds)
            },
            ControlMessage::Unknown(UnknownCmsg(_, bytes)) => {
                mem::size_of_val(bytes)
            }
        }
    }

    // Unsafe: start and end of buffer must be size_t-aligned (that is,
    // cmsg_align'd). Updates the provided slice; panics if the buffer
    // is too small.
    unsafe fn encode_into<'b>(&self, buf: &mut &'b mut [u8]) {
        match *self {
            ControlMessage::ScmRights(fds) => {
                let cmsg = cmsghdr {
                    cmsg_len: self.len() as type_of_cmsg_len,
                    cmsg_level: SOL_SOCKET,
                    cmsg_type: SCM_RIGHTS,
                    cmsg_data: [],
                };
                copy_bytes(&cmsg, buf);
                copy_bytes(fds, buf);
            },
            ControlMessage::Unknown(UnknownCmsg(orig_cmsg, bytes)) => {
                copy_bytes(orig_cmsg, buf);
                copy_bytes(bytes, buf);
            }
        }
    }
}


/// Send data in scatter-gather vectors to a socket, possibly accompanied
/// by ancillary data. Optionally direct the message at the given address,
/// as with sendto.
///
/// Allocates if cmsgs is nonempty.
pub fn sendmsg<'a>(fd: RawFd, iov: &[IoVec<&'a [u8]>], cmsgs: &[ControlMessage<'a>], flags: MsgFlags, addr: Option<&'a SockAddr>) -> Result<usize> {
    let mut len = 0;
    let mut capacity = 0;
    for cmsg in cmsgs {
        len += cmsg.len();
        capacity += cmsg.space();
    }
    // Alignment hackery. Note that capacity is guaranteed to be a
    // multiple of size_t. Note also that the resulting vector claims
    // to have length == capacity, so it's presently uninitialized.
    let mut cmsg_buffer = unsafe {
        let mut vec = Vec::<size_t>::with_capacity(capacity / mem::size_of::<size_t>());
        let ptr = vec.as_mut_ptr();
        mem::forget(vec);
        Vec::<u8>::from_raw_parts(ptr as *mut _, capacity, capacity)
    };
    {
        let mut ptr = &mut cmsg_buffer[..];
        for cmsg in cmsgs {
            unsafe { cmsg.encode_into(&mut ptr) };
        }
    }

    let (name, namelen) = match addr {
        Some(addr) => { let (x, y) = unsafe { addr.as_ffi_pair() }; (x as *const _, y) }
        None => (0 as *const _, 0),
    };

    let mhdr = msghdr {
        msg_name: name as *const c_void,
        msg_namelen: namelen,
        msg_iov: iov.as_ptr(),
        msg_iovlen: iov.len() as size_t,
        msg_control: cmsg_buffer.as_ptr() as *const c_void,
        msg_controllen: len as size_t,
        msg_flags: 0,
    };
    let ret = unsafe { ffi::sendmsg(fd, &mhdr, flags.bits()) };

    Errno::result(ret).map(|r| r as usize)
}

/// Receive message in scatter-gather vectors from a socket, and
/// optionally receive ancillary data into the provided buffer.
/// If no ancillary data is desired, use () as the type parameter.
pub fn recvmsg<'a, T>(fd: RawFd, iov: &[IoVec<&mut [u8]>], cmsg_buffer: Option<&'a mut CmsgSpace<T>>, flags: MsgFlags) -> Result<RecvMsg<'a>> {
    let mut address: sockaddr_storage = unsafe { mem::uninitialized() };
    let (msg_control, msg_controllen) = match cmsg_buffer {
        Some(cmsg_buffer) => (cmsg_buffer as *mut _, mem::size_of_val(cmsg_buffer)),
        None => (0 as *mut _, 0),
    };
    let mut mhdr = msghdr {
        msg_name: &mut address as *const _ as *const c_void,
        msg_namelen: mem::size_of::<sockaddr_storage>() as socklen_t,
        msg_iov: iov.as_ptr() as *const IoVec<&[u8]>, // safe cast to add const-ness
        msg_iovlen: iov.len() as size_t,
        msg_control: msg_control as *const c_void,
        msg_controllen: msg_controllen as size_t,
        msg_flags: 0,
    };
    let ret = unsafe { ffi::recvmsg(fd, &mut mhdr, flags.bits()) };

    Ok(unsafe { RecvMsg {
        bytes: try!(Errno::result(ret)) as usize,
        cmsg_buffer: slice::from_raw_parts(mhdr.msg_control as *const u8,
                                           mhdr.msg_controllen as usize),
        address: sockaddr_storage_to_addr(&address,
                                          mhdr.msg_namelen as usize).ok(),
        flags: MsgFlags::from_bits_truncate(mhdr.msg_flags),
    } })
}


/// Create an endpoint for communication
///
/// [Further reading](http://man7.org/linux/man-pages/man2/socket.2.html)
pub fn socket(domain: AddressFamily, ty: SockType, flags: SockFlag, protocol: c_int) -> Result<RawFd> {
    let mut ty = ty as c_int;
    let feat_atomic = features::socket_atomic_cloexec();

    if feat_atomic {
        ty = ty | flags.bits();
    }

    // TODO: Check the kernel version
    let res = try!(Errno::result(unsafe { ffi::socket(domain as c_int, ty, protocol) }));

    if !feat_atomic {
        if flags.contains(SOCK_CLOEXEC) {
            try!(fcntl(res, F_SETFD(FD_CLOEXEC)));
        }

        if flags.contains(SOCK_NONBLOCK) {
            try!(fcntl(res, F_SETFL(O_NONBLOCK)));
        }
    }

    Ok(res)
}

/// Create a pair of connected sockets
///
/// [Further reading](http://man7.org/linux/man-pages/man2/socketpair.2.html)
pub fn socketpair(domain: AddressFamily, ty: SockType, protocol: c_int,
                  flags: SockFlag) -> Result<(RawFd, RawFd)> {
    let mut ty = ty as c_int;
    let feat_atomic = features::socket_atomic_cloexec();

    if feat_atomic {
        ty = ty | flags.bits();
    }
    let mut fds = [-1, -1];
    let res = unsafe {
        ffi::socketpair(domain as c_int, ty, protocol, fds.as_mut_ptr())
    };
    try!(Errno::result(res));

    if !feat_atomic {
        if flags.contains(SOCK_CLOEXEC) {
            try!(fcntl(fds[0], F_SETFD(FD_CLOEXEC)));
            try!(fcntl(fds[1], F_SETFD(FD_CLOEXEC)));
        }

        if flags.contains(SOCK_NONBLOCK) {
            try!(fcntl(fds[0], F_SETFL(O_NONBLOCK)));
            try!(fcntl(fds[1], F_SETFL(O_NONBLOCK)));
        }
    }
    Ok((fds[0], fds[1]))
}

/// Listen for connections on a socket
///
/// [Further reading](http://man7.org/linux/man-pages/man2/listen.2.html)
pub fn listen(sockfd: RawFd, backlog: usize) -> Result<()> {
    let res = unsafe { ffi::listen(sockfd, backlog as c_int) };

    Errno::result(res).map(drop)
}

/// Bind a name to a socket
///
/// [Further reading](http://man7.org/linux/man-pages/man2/bind.2.html)
pub fn bind(fd: RawFd, addr: &SockAddr) -> Result<()> {
    let res = unsafe {
        let (ptr, len) = addr.as_ffi_pair();
        ffi::bind(fd, ptr, len)
    };

    Errno::result(res).map(drop)
}

/// Accept a connection on a socket
///
/// [Further reading](http://man7.org/linux/man-pages/man2/accept.2.html)
pub fn accept(sockfd: RawFd) -> Result<RawFd> {
    let res = unsafe { ffi::accept(sockfd, ptr::null_mut(), ptr::null_mut()) };

    Errno::result(res)
}

/// Accept a connection on a socket
///
/// [Further reading](http://man7.org/linux/man-pages/man2/accept.2.html)
pub fn accept4(sockfd: RawFd, flags: SockFlag) -> Result<RawFd> {
    accept4_polyfill(sockfd, flags)
}

#[inline]
fn accept4_polyfill(sockfd: RawFd, flags: SockFlag) -> Result<RawFd> {
    let res = try!(Errno::result(unsafe { ffi::accept(sockfd, ptr::null_mut(), ptr::null_mut()) }));

    if flags.contains(SOCK_CLOEXEC) {
        try!(fcntl(res, F_SETFD(FD_CLOEXEC)));
    }

    if flags.contains(SOCK_NONBLOCK) {
        try!(fcntl(res, F_SETFL(O_NONBLOCK)));
    }

    Ok(res)
}

/// Initiate a connection on a socket
///
/// [Further reading](http://man7.org/linux/man-pages/man2/connect.2.html)
pub fn connect(fd: RawFd, addr: &SockAddr) -> Result<()> {
    let res = unsafe {
        let (ptr, len) = addr.as_ffi_pair();
        ffi::connect(fd, ptr, len)
    };

    Errno::result(res).map(drop)
}

/// Receive data from a connection-oriented socket. Returns the number of
/// bytes read
///
/// [Further reading](http://man7.org/linux/man-pages/man2/recv.2.html)
pub fn recv(sockfd: RawFd, buf: &mut [u8], flags: MsgFlags) -> Result<usize> {
    unsafe {
        let ret = ffi::recv(
            sockfd,
            buf.as_ptr() as *mut c_void,
            buf.len() as size_t,
            flags.bits());

        Errno::result(ret).map(|r| r as usize)
    }
}

/// Receive data from a connectionless or connection-oriented socket. Returns
/// the number of bytes read and the socket address of the sender.
///
/// [Further reading](http://man7.org/linux/man-pages/man2/recvmsg.2.html)
pub fn recvfrom(sockfd: RawFd, buf: &mut [u8]) -> Result<(usize, SockAddr)> {
    unsafe {
        let addr: sockaddr_storage = mem::zeroed();
        let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

        let ret = try!(Errno::result(ffi::recvfrom(
            sockfd,
            buf.as_ptr() as *mut c_void,
            buf.len() as size_t,
            0,
            mem::transmute(&addr),
            &mut len as *mut socklen_t)));

        sockaddr_storage_to_addr(&addr, len as usize)
            .map(|addr| (ret as usize, addr))
    }
}

pub fn sendto(fd: RawFd, buf: &[u8], addr: &SockAddr, flags: MsgFlags) -> Result<usize> {
    let ret = unsafe {
        let (ptr, len) = addr.as_ffi_pair();
        ffi::sendto(fd, buf.as_ptr() as *const c_void, buf.len() as size_t, flags.bits(), ptr, len)
    };

    Errno::result(ret).map(|r| r as usize)
}

/// Send data to a connection-oriented socket. Returns the number of bytes read
///
/// [Further reading](http://man7.org/linux/man-pages/man2/send.2.html)
pub fn send(fd: RawFd, buf: &[u8], flags: MsgFlags) -> Result<usize> {
    let ret = unsafe {
        ffi::send(fd, buf.as_ptr() as *const c_void, buf.len() as size_t, flags.bits())
    };

    Errno::result(ret).map(|r| r as usize)
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct linger {
    pub l_onoff: c_int,
    pub l_linger: c_int
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ucred {
    pid: pid_t,
    uid: uid_t,
    gid: gid_t,
}

/*
 *
 * ===== Socket Options =====
 *
 */

/// The protocol level at which to get / set socket options. Used as an
/// argument to `getsockopt` and `setsockopt`.
///
/// [Further reading](http://man7.org/linux/man-pages/man2/setsockopt.2.html)
#[repr(i32)]
pub enum SockLevel {
    Socket = SOL_SOCKET,
    Tcp = IPPROTO_TCP,
    Ip = IPPROTO_IP,
    Ipv6 = IPPROTO_IPV6,
    Udp = IPPROTO_UDP,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    Netlink = SOL_NETLINK,
}

/// Represents a socket option that can be accessed or set. Used as an argument
/// to `getsockopt`
pub trait GetSockOpt : Copy {
    type Val;

    #[doc(hidden)]
    fn get(&self, fd: RawFd) -> Result<Self::Val>;
}

/// Represents a socket option that can be accessed or set. Used as an argument
/// to `setsockopt`
pub trait SetSockOpt : Copy {
    type Val;

    #[doc(hidden)]
    fn set(&self, fd: RawFd, val: &Self::Val) -> Result<()>;
}

/// Get the current value for the requested socket option
///
/// [Further reading](http://man7.org/linux/man-pages/man2/getsockopt.2.html)
pub fn getsockopt<O: GetSockOpt>(fd: RawFd, opt: O) -> Result<O::Val> {
    opt.get(fd)
}

/// Sets the value for the requested socket option
///
/// [Further reading](http://man7.org/linux/man-pages/man2/setsockopt.2.html)
pub fn setsockopt<O: SetSockOpt>(fd: RawFd, opt: O, val: &O::Val) -> Result<()> {
    opt.set(fd, val)
}

/// Get the address of the peer connected to the socket `fd`.
///
/// [Further reading](http://man7.org/linux/man-pages/man2/getpeername.2.html)
pub fn getpeername(fd: RawFd) -> Result<SockAddr> {
    unsafe {
        let addr: sockaddr_storage = mem::uninitialized();
        let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

        let ret = ffi::getpeername(fd, mem::transmute(&addr), &mut len);

        try!(Errno::result(ret));

        sockaddr_storage_to_addr(&addr, len as usize)
    }
}

/// Get the current address to which the socket `fd` is bound.
///
/// [Further reading](http://man7.org/linux/man-pages/man2/getsockname.2.html)
pub fn getsockname(fd: RawFd) -> Result<SockAddr> {
    unsafe {
        let addr: sockaddr_storage = mem::uninitialized();
        let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

        let ret = ffi::getsockname(fd, mem::transmute(&addr), &mut len);

        try!(Errno::result(ret));

        sockaddr_storage_to_addr(&addr, len as usize)
    }
}

pub unsafe fn sockaddr_storage_to_addr(
    addr: &sockaddr_storage,
    len: usize) -> Result<SockAddr> {

    if len < mem::size_of_val(&addr.ss_family) {
        return Err(Error::Sys(Errno::ENOTCONN));
    }

    match addr.ss_family as c_int {
        consts::AF_INET => {
            assert!(len as usize == mem::size_of::<sockaddr_in>());
            let ret = *(addr as *const _ as *const sockaddr_in);
            Ok(SockAddr::Inet(InetAddr::V4(ret)))
        }
        consts::AF_INET6 => {
            assert!(len as usize == mem::size_of::<sockaddr_in6>());
            Ok(SockAddr::Inet(InetAddr::V6((*(addr as *const _ as *const sockaddr_in6)))))
        }
        consts::AF_UNIX => {
            Ok(SockAddr::Unix(UnixAddr(*(addr as *const _ as *const sockaddr_un), len)))
        }
        af => panic!("unexpected address family {}", af),
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shutdown {
    /// Further receptions will be disallowed.
    Read,
    /// Further  transmissions will be disallowed.
    Write,
    /// Further receptions and transmissions will be disallowed.
    Both,
}

/// Shut down part of a full-duplex connection.
///
/// [Further reading](http://man7.org/linux/man-pages/man2/shutdown.2.html)
pub fn shutdown(df: RawFd, how: Shutdown) -> Result<()> {
    unsafe {
        use libc::shutdown;

        let how = match how {
            Shutdown::Read  => consts::SHUT_RD,
            Shutdown::Write => consts::SHUT_WR,
            Shutdown::Both  => consts::SHUT_RDWR,
        };

        Errno::result(shutdown(df, how)).map(drop)
    }
}

#[test]
pub fn test_struct_sizes() {
    use nixtest;
    nixtest::assert_size_of::<sockaddr_storage>("sockaddr_storage");
}
