use nix::{
    errno::Errno,
    sys::epoll::{Epoll, EpollCreateFlags, EpollEvent, EpollFlags},
};
use std::os::fd::BorrowedFd;

#[test]
pub fn test_epoll_errno() {
    let epoll = Epoll::new(EpollCreateFlags::empty()).unwrap();
    let fd_1 = unsafe { BorrowedFd::borrow_raw(1) };
    let result = epoll.delete(fd_1);
    assert_eq!(result.unwrap_err(), Errno::ENOENT);
}

#[test]
pub fn test_epoll_add_delete() {
    let epoll = Epoll::new(EpollCreateFlags::empty()).unwrap();
    let event = EpollEvent::new(EpollFlags::EPOLLIN | EpollFlags::EPOLLERR, 1);
    let fd_1 = unsafe { BorrowedFd::borrow_raw(1) };

    epoll.add(fd_1, event).unwrap();
    epoll.delete(fd_1).unwrap();
}
