use nix::sys::epoll::{EpollCreateFlags, EpollOp, EpollEvent};
use nix::sys::epoll::{EPOLLIN, EPOLLERR};
use nix::sys::epoll::{epoll_create1, epoll_ctl};
use nix::{Error, Errno, Result};

#[test]
pub fn test_epoll_errno() {
    let efd = epoll_create1(EpollCreateFlags::empty()).expect("epoll_create1 failed");
    let result = epoll_ctl(efd, EpollOp::EpollCtlDel, 1, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::Sys(Errno::ENOENT));

    let result = epoll_ctl(efd, EpollOp::EpollCtlAdd, 1, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::Sys(Errno::EINVAL));
}

#[test]
pub fn test_epoll_ctl() {
    let ok : Result<()> = Ok(());
    let notok: Result<()> = Err(Error::Sys(Errno::UnknownErrno));
    println!("In test_epoll_ctl, ok is {:?}, notok is {:?}", ok, notok);

    let efd = epoll_create1(EpollCreateFlags::empty()).expect("epoll_create1 failed");
    let mut event = EpollEvent::new(EPOLLIN | EPOLLERR, 1);
    let epoll_ctl_res = epoll_ctl(efd, EpollOp::EpollCtlAdd, 1, &mut event);
    println!("In test_epoll_ctl, result was {:?}", epoll_ctl_res);
    epoll_ctl_res.expect("epoll_ctl 1 failed");
    epoll_ctl(efd, EpollOp::EpollCtlDel, 1, None).expect("epoll_ctl 2 failed");
}
