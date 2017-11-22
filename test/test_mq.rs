use libc::c_long;

use std::ffi::CString;
use std::str;

use nix::errno::Errno::*;
use nix::Error::Sys;
use nix::mqueue::{mq_open, mq_close, mq_send, mq_receive, mq_getattr, mq_setattr, mq_unlink, mq_set_nonblock, mq_remove_nonblock};
use nix::mqueue::{MqAttr, MQ_OFlag};
use nix::sys::stat::Mode;

#[test]
fn test_mq_send_and_receive() {
    const MSG_SIZE: c_long =  32;
    let attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name= &CString::new(b"/a_nix_test_queue".as_ref()).unwrap();

    let mqd0 = mq_open(mq_name, MQ_OFlag::O_CREAT | MQ_OFlag::O_WRONLY,
                       Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH,
                       Some(&attr)).unwrap();
    let msg_to_send = "msg_1";
    mq_send(mqd0, msg_to_send.as_bytes(), 1).unwrap();

    let mqd1 = mq_open(mq_name, MQ_OFlag::O_CREAT | MQ_OFlag::O_RDONLY,
                       Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH,
                       Some(&attr)).unwrap();
    let mut buf = [0u8; 32];
    let mut prio = 0u32;
    let len = mq_receive(mqd1, &mut buf, &mut prio).unwrap();
    assert!(prio == 1);

    mq_close(mqd1).unwrap();
    mq_close(mqd0).unwrap();
    assert_eq!(msg_to_send, str::from_utf8(&buf[0..len]).unwrap());
}


#[test]
fn test_mq_getattr() {
    const MSG_SIZE: c_long =  32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name = &CString::new("/attr_test_get_attr".as_bytes().as_ref()).unwrap();
    let mqd = mq_open(mq_name, MQ_OFlag::O_CREAT | MQ_OFlag::O_WRONLY, Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH, Some(&initial_attr)).unwrap();
    let read_attr = mq_getattr(mqd);
    assert!(read_attr.unwrap() == initial_attr);
    mq_close(mqd).unwrap();
}

// FIXME: Fix failures for mips in QEMU
#[test]
#[cfg_attr(any(target_arch = "mips", target_arch = "mips64"), ignore)]
fn test_mq_setattr() {
    const MSG_SIZE: c_long =  32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name = &CString::new("/attr_test_get_attr".as_bytes().as_ref()).unwrap();
    let mqd = mq_open(mq_name, MQ_OFlag::O_CREAT | MQ_OFlag::O_WRONLY, Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH, Some(&initial_attr)).unwrap();

    let new_attr =  MqAttr::new(0, 20, MSG_SIZE * 2, 100);
    let old_attr = mq_setattr(mqd, &new_attr);
    assert!(old_attr.unwrap() == initial_attr);

    let new_attr_get = mq_getattr(mqd);
    // The following tests make sense. No changes here because according to the Linux man page only
    // O_NONBLOCK can be set (see tests below)
    assert!(new_attr_get.unwrap() != new_attr);

    let new_attr_non_blocking =  MqAttr::new(MQ_OFlag::O_NONBLOCK.bits() as c_long, 10, MSG_SIZE, 0);
    mq_setattr(mqd, &new_attr_non_blocking).unwrap();
    let new_attr_get = mq_getattr(mqd);

    // now the O_NONBLOCK flag has been set
    assert!(new_attr_get.unwrap() != initial_attr);
    assert!(new_attr_get.unwrap() == new_attr_non_blocking);
    mq_close(mqd).unwrap();
}

// FIXME: Fix failures for mips in QEMU
#[test]
#[cfg_attr(any(target_arch = "mips", target_arch = "mips64"), ignore)]
fn test_mq_set_nonblocking() {
    const MSG_SIZE: c_long =  32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name = &CString::new("/attr_test_get_attr".as_bytes().as_ref()).unwrap();
    let mqd = mq_open(mq_name, MQ_OFlag::O_CREAT | MQ_OFlag::O_WRONLY, Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH, Some(&initial_attr)).unwrap();
    mq_set_nonblock(mqd).unwrap();
    let new_attr = mq_getattr(mqd);
    assert!(new_attr.unwrap().flags() == MQ_OFlag::O_NONBLOCK.bits() as c_long);
    mq_remove_nonblock(mqd).unwrap();
    let new_attr = mq_getattr(mqd);
    assert!(new_attr.unwrap().flags() == 0);
    mq_close(mqd).unwrap();
}

#[test]
fn test_mq_unlink() {
    const MSG_SIZE: c_long =  32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name_opened = &CString::new("/mq_unlink_test".as_bytes().as_ref()).unwrap();
    let mq_name_not_opened = &CString::new("/mq_unlink_test".as_bytes().as_ref()).unwrap();
    let mqd = mq_open(mq_name_opened, MQ_OFlag::O_CREAT | MQ_OFlag::O_WRONLY, Mode::S_IWUSR | Mode::S_IRUSR | Mode::S_IRGRP | Mode::S_IROTH, Some(&initial_attr)).unwrap();

    let res_unlink = mq_unlink(mq_name_opened);
    assert!(res_unlink == Ok(()) );

    let res_unlink_not_opened = mq_unlink(mq_name_not_opened);
    assert!(res_unlink_not_opened == Err(Sys(ENOENT)) );

    mq_close(mqd).unwrap();
    let res_unlink_after_close = mq_unlink(mq_name_opened);
    assert!(res_unlink_after_close == Err(Sys(ENOENT)) );

}
