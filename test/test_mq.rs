use nix::mqueue::{mq_open, mq_close, mq_send, mq_receive, mq_getattr, mq_setattr, mq_unlink, mq_set_nonblock, mq_remove_nonblock};
use nix::mqueue::{O_CREAT, O_WRONLY, O_RDONLY, O_NONBLOCK};


use nix::mqueue::MqAttr;
use nix::sys::stat::{S_IWUSR, S_IRUSR, S_IRGRP, S_IROTH};
use std::ffi::CString;
use std::str;
use libc::c_long;

use nix::unistd::{fork, read, write, pipe};
use nix::unistd::ForkResult::*;
use nix::sys::wait::*;
use nix::errno::Errno::*;
use nix::Error::Sys;

#[test]
fn test_mq_send_and_receive() {

    const MSG_SIZE: c_long =  32;
    let attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name_in_parent = &CString::new(b"/a_nix_test_queue".as_ref()).unwrap();
    let mqd_in_parent = mq_open(mq_name_in_parent, O_CREAT | O_WRONLY, S_IWUSR | S_IRUSR | S_IRGRP | S_IROTH, Some(&attr)).unwrap();
    let msg_to_send = "msg_1".as_bytes();

    mq_send(mqd_in_parent, msg_to_send, 1).unwrap();

    let (reader, writer) = pipe().unwrap();

    let pid = fork();
    match pid {
        Ok(Child) => {
            let mq_name_in_child =  &CString::new(b"/a_nix_test_queue".as_ref()).unwrap();
            let mqd_in_child = mq_open(mq_name_in_child, O_CREAT | O_RDONLY, S_IWUSR | S_IRUSR | S_IRGRP | S_IROTH, Some(&attr)).unwrap();
            let mut buf = [0u8; 32];
            mq_receive(mqd_in_child, &mut buf, 1).unwrap();
            write(writer, &buf).unwrap();  // pipe result to parent process. Otherwise cargo does not report test failures correctly
            mq_close(mqd_in_child).unwrap();
      }
      Ok(Parent { child }) => {
          mq_close(mqd_in_parent).unwrap();

          // Wait for the child to exit.
          waitpid(child, None).unwrap();
          // Read 1024 bytes.
          let mut read_buf = [0u8; 32];
          read(reader, &mut read_buf).unwrap();
          let message_str = str::from_utf8(&read_buf).unwrap();
          assert_eq!(&message_str[.. message_str.char_indices().nth(5).unwrap().0], "msg_1");
      },
      // panic, fork should never fail unless there is a serious problem with the OS
      Err(_) => panic!("Error: Fork Failed")
    }
}


#[test]
fn test_mq_getattr() {
    const MSG_SIZE: c_long =  32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name = &CString::new("/attr_test_get_attr".as_bytes().as_ref()).unwrap();
    let mqd = mq_open(mq_name, O_CREAT | O_WRONLY, S_IWUSR | S_IRUSR | S_IRGRP | S_IROTH, Some(&initial_attr)).unwrap();
    let read_attr = mq_getattr(mqd);
    assert!(read_attr.unwrap() == initial_attr);
    mq_close(mqd).unwrap();
}

#[test]
fn test_mq_setattr() {
    const MSG_SIZE: c_long =  32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name = &CString::new("/attr_test_get_attr".as_bytes().as_ref()).unwrap();
    let mqd = mq_open(mq_name, O_CREAT | O_WRONLY, S_IWUSR | S_IRUSR | S_IRGRP | S_IROTH, Some(&initial_attr)).unwrap();

    let new_attr =  MqAttr::new(0, 20, MSG_SIZE * 2, 100);
    let old_attr = mq_setattr(mqd, &new_attr);
    assert!(old_attr.unwrap() == initial_attr);

    let new_attr_get = mq_getattr(mqd);
    // The following tests make sense. No changes here because according to the Linux man page only
    // O_NONBLOCK can be set (see tests below)
    assert!(new_attr_get.unwrap() != new_attr);

    let new_attr_non_blocking =  MqAttr::new(O_NONBLOCK.bits() as c_long, 10, MSG_SIZE, 0);
    mq_setattr(mqd, &new_attr_non_blocking).unwrap();
    let new_attr_get = mq_getattr(mqd);

    // now the O_NONBLOCK flag has been set
    assert!(new_attr_get.unwrap() != initial_attr);
    assert!(new_attr_get.unwrap() == new_attr_non_blocking);
    mq_close(mqd).unwrap();
}

#[test]
fn test_mq_set_nonblocking() {
    const MSG_SIZE: c_long =  32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name = &CString::new("/attr_test_get_attr".as_bytes().as_ref()).unwrap();
    let mqd = mq_open(mq_name, O_CREAT | O_WRONLY, S_IWUSR | S_IRUSR | S_IRGRP | S_IROTH, Some(&initial_attr)).unwrap();
    mq_set_nonblock(mqd).unwrap();
    let new_attr = mq_getattr(mqd);
    assert!(new_attr.unwrap().mq_flags == O_NONBLOCK.bits() as c_long);
    mq_remove_nonblock(mqd).unwrap();
    let new_attr = mq_getattr(mqd);
    assert!(new_attr.unwrap().mq_flags == 0);
    mq_close(mqd).unwrap();
}

#[test]
fn test_mq_unlink() {
    const MSG_SIZE: c_long =  32;
    let initial_attr =  MqAttr::new(0, 10, MSG_SIZE, 0);
    let mq_name_opened = &CString::new("/mq_unlink_test".as_bytes().as_ref()).unwrap();
    let mq_name_not_opened = &CString::new("/mq_unlink_test".as_bytes().as_ref()).unwrap();
    let mqd = mq_open(mq_name_opened, O_CREAT | O_WRONLY, S_IWUSR | S_IRUSR | S_IRGRP | S_IROTH, Some(&initial_attr)).unwrap();

    let res_unlink = mq_unlink(mq_name_opened);
    assert!(res_unlink == Ok(()) );

    let res_unlink_not_opened = mq_unlink(mq_name_not_opened);
    assert!(res_unlink_not_opened == Err(Sys(ENOENT)) );

    mq_close(mqd).unwrap();
    let res_unlink_after_close = mq_unlink(mq_name_opened);
    assert!(res_unlink_after_close == Err(Sys(ENOENT)) );

}
