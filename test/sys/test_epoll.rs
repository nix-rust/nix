use nix::sys::epoll::{EpollCreateFlags, EpollOp, EpollEvent};
use nix::sys::epoll::{EPOLLIN, EPOLLERR};
use nix::sys::epoll::{epoll_create1, epoll_ctl};
use nix::{Error, Errno};

#[test]
pub fn test_epoll_errno() {
    let efd = epoll_create1(EpollCreateFlags::empty()).unwrap();
    let result = epoll_ctl(efd, EpollOp::EpollCtlDel, 1, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::Sys(Errno::ENOENT));
}

#[test]
pub fn test_epoll_ctl() {
    let efd = epoll_create1(EpollCreateFlags::empty()).unwrap();
    let mut event = EpollEvent::new(EPOLLIN | EPOLLERR, 1);
    epoll_ctl(efd, EpollOp::EpollCtlAdd, 1, &mut event).unwrap();
    epoll_ctl(efd, EpollOp::EpollCtlDel, 1, None).unwrap();
}
