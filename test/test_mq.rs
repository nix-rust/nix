use nix::mq::{mq_open, mq_close, mq_send, mq_receive};
use nix::mq::{O_CREAT, O_WRONLY, O_RDONLY};
use nix::mq::MqAttr;
use nix::sys::stat::{S_IWUSR, S_IRUSR, S_IRGRP, S_IROTH};
use std::ffi::CString;
use libc::{c_long, strlen};

use nix::unistd::{fork, read, write, pipe};
use nix::unistd::Fork::{Child, Parent};
use nix::sys::wait::*;


#[test]
fn mq_send_and_receive() {

    const MSG_SIZE: c_long =  32;

    let attr =  MqAttr { mq_flags: 0, mq_maxmsg: 10, mq_msgsize: MSG_SIZE, mq_curmsgs: 0 };
    let mq_name_in_parent = &CString::new(b"/a_nix_test_queue".as_ref()).unwrap();
    let mqd_in_parent = mq_open(mq_name_in_parent, O_CREAT | O_WRONLY, S_IWUSR | S_IRUSR | S_IRGRP | S_IROTH, &attr).unwrap();
    let msg_to_send = &CString::new("msg_1").unwrap();
    let len = unsafe { strlen(msg_to_send.as_ptr()) as usize };

    mq_send(mqd_in_parent, msg_to_send, 1).unwrap();

    let (reader, writer) = pipe().unwrap();

    let pid = fork();
    match pid {
        Ok(Child) => {
            let mq_name_in_child =  &CString::new(b"/a_nix_test_queue".as_ref()).unwrap();
            let mqd_in_child = mq_open(mq_name_in_child, O_CREAT | O_RDONLY, S_IWUSR | S_IRUSR | S_IRGRP | S_IROTH, &attr).unwrap();
            let mut buf = [0u8; 32];
            let length_msg_received = mq_receive(mqd_in_child, &mut buf, 1).unwrap();
            assert!(length_msg_received == len);
            let message_str = String::from_utf8_lossy(&buf[0 .. len]);
            let expected_str = String::from_utf8_lossy(msg_to_send.as_bytes());
            assert!(message_str == expected_str);
            write(writer, &buf).unwrap();  // pipe result to parent process. Otherwise cargo does not report test failures correctly
            mq_close(mqd_in_child).unwrap();
      }
      Ok(Parent(child_pid)) => {
          mq_close(mqd_in_parent).unwrap();

          // Wait for the child to exit.
          waitpid(child_pid, None).unwrap();
          // Read 1024 bytes.
          let mut read_buf = [0u8; 32];
          read(reader, &mut read_buf).unwrap();
          let message_str = String::from_utf8_lossy(&read_buf);
          assert!(message_str.contains("msg_1"));
      },
      // panic, fork should never fail unless there is a serious problem with the OS
      Err(_) => panic!("Error: Fork Failed")
    }
}



// cargo clean; cargo test -- --nocapture
