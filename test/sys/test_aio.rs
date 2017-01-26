use libc::c_int;
use nix::{Error, Result};
use nix::errno::*;
use nix::sys::aio::*;
use nix::sys::signal::*;
use nix::sys::time::{TimeSpec, TimeValLike};
use std::io::{Write, Read, Seek, SeekFrom};
use std::os::unix::io::AsRawFd;
use std::{thread, time};
use tempfile::tempfile;

// Helper that polls an AioCb for completion or error
fn poll_aio(mut aiocb: &mut AioCb) -> Result<()> {
    loop {
        let err = aio_error(&mut aiocb);
        if err != Err(Error::from(Errno::EINPROGRESS)) { return err; };
        thread::sleep(time::Duration::from_millis(10));
    }
}

// Tests aio_cancel.  We aren't trying to test the OS's implementation, only our
// bindings.  So it's sufficient to check that aio_cancel returned any
// AioCancelStat value.
#[test]
fn test_aio_cancel() {
    let mut wbuf = "CDEF".to_string().into_bytes();

    let f = tempfile().unwrap();
    let mut aiocb = AioCb::from_mut_slice( f.as_raw_fd(),
                            0,   //offset
                            &mut wbuf,
                            0,   //priority
                            SigevNotify::SigevNone,
                            LioOpcode::LIO_NOP);
    aio_write(&mut aiocb).unwrap();
    let err = aio_error(&mut aiocb);
    assert!(err == Ok(()) || err == Err(Error::from(Errno::EINPROGRESS)));

    let cancelstat = aio_cancel(f.as_raw_fd(), Some(&mut aiocb));
    assert!(cancelstat.is_ok());

    // Wait for aiocb to complete, but don't care whether it succeeded
    let _ = poll_aio(&mut aiocb);
    let _ = aio_return(&mut aiocb);
}

// Tests using aio_cancel for all outstanding IOs.
#[test]
fn test_aio_cancel_all() {
    let mut wbuf = "CDEF".to_string().into_bytes();

    let f = tempfile().unwrap();
    let mut aiocb = AioCb::from_mut_slice( f.as_raw_fd(),
                            0,   //offset
                            &mut wbuf,
                            0,   //priority
                            SigevNotify::SigevNone,
                            LioOpcode::LIO_NOP);
    aio_write(&mut aiocb).unwrap();
    let err = aio_error(&mut aiocb);
    assert!(err == Ok(()) || err == Err(Error::from(Errno::EINPROGRESS)));

    let cancelstat = aio_cancel(f.as_raw_fd(), None);
    assert!(cancelstat.is_ok());

    // Wait for aiocb to complete, but don't care whether it succeeded
    let _ = poll_aio(&mut aiocb);
    let _ = aio_return(&mut aiocb);
}

#[test]
fn test_aio_fsync() {
    const INITIAL: &'static [u8] = b"abcdef123456";
    let mut f = tempfile().unwrap();
    f.write(INITIAL).unwrap();
    let mut aiocb = AioCb::from_fd( f.as_raw_fd(),
                            0,   //priority
                            SigevNotify::SigevNone);
    let err = aio_fsync(AioFsyncMode::O_SYNC, &mut aiocb);
    assert!(err.is_ok());
    poll_aio(&mut aiocb).unwrap();
    aio_return(&mut aiocb).unwrap();
}


#[test]
fn test_aio_suspend() {
    const INITIAL: &'static [u8] = b"abcdef123456";
    const WBUF: &'static [u8] = b"CDEF";
    let timeout = TimeSpec::seconds(10);
    let mut rbuf = vec![0; 4];
    let mut f = tempfile().unwrap();
    f.write(INITIAL).unwrap();

    let mut wcb = unsafe {
        AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           &mut WBUF,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_WRITE)
    };

    let mut rcb = AioCb::from_mut_slice( f.as_raw_fd(),
                            8,   //offset
                            &mut rbuf,
                            0,   //priority
                            SigevNotify::SigevNone,
                            LioOpcode::LIO_READ);
    aio_write(&mut wcb).unwrap();
    aio_read(&mut rcb).unwrap();
    loop {
        {
            let cbbuf = [&wcb, &rcb];
            assert!(aio_suspend(&cbbuf[..], Some(timeout)).is_ok());
        }
        if aio_error(&mut rcb) != Err(Error::from(Errno::EINPROGRESS)) &&
           aio_error(&mut wcb) != Err(Error::from(Errno::EINPROGRESS)) {
            break
        }
    }

    assert!(aio_return(&mut wcb).unwrap() as usize == WBUF.len());
    assert!(aio_return(&mut rcb).unwrap() as usize == WBUF.len());
}

// Test a simple aio operation with no completion notification.  We must poll
// for completion
#[test]
fn test_aio_read() {
    const INITIAL: &'static [u8] = b"abcdef123456";
    let mut rbuf = vec![0; 4];
    const EXPECT: &'static [u8] = b"cdef";
    let mut f = tempfile().unwrap();
    f.write(INITIAL).unwrap();
    {
        let mut aiocb = AioCb::from_mut_slice( f.as_raw_fd(),
                               2,   //offset
                               &mut rbuf,
                               0,   //priority
                               SigevNotify::SigevNone,
                               LioOpcode::LIO_NOP);
        aio_read(&mut aiocb).unwrap();

        let err = poll_aio(&mut aiocb);
        assert!(err == Ok(()));
        assert!(aio_return(&mut aiocb).unwrap() as usize == EXPECT.len());
    }

    assert!(rbuf == EXPECT);
}

// Test a simple aio operation with no completion notification.  We must poll
// for completion.  Unlike test_aio_read, this test uses AioCb::from_slice
#[test]
fn test_aio_write() {
    const INITIAL: &'static [u8] = b"abcdef123456";
    const WBUF: &'static [u8] = b"CDEF"; //"CDEF".to_string().into_bytes();
    let mut rbuf = Vec::new();
    const EXPECT: &'static [u8] = b"abCDEF123456";

    let mut f = tempfile().unwrap();
    f.write(INITIAL).unwrap();
    let mut aiocb = unsafe {
        AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           &WBUF,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_NOP)
    };
    aio_write(&mut aiocb).unwrap();

    let err = poll_aio(&mut aiocb);
    assert!(err == Ok(()));
    assert!(aio_return(&mut aiocb).unwrap() as usize == WBUF.len());

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf).unwrap();
    assert!(len == EXPECT.len());
    assert!(rbuf == EXPECT);
}

// XXX: should be sig_atomic_t, but rust's libc doesn't define that yet
static mut signaled: i32 = 0;

extern fn sigfunc(_: c_int) {
    // It's a pity that Rust can't understand that static mutable sig_atomic_t
    // variables can be safely accessed
    unsafe { signaled = 1 };
}

// Test an aio operation with completion delivered by a signal
#[test]
fn test_aio_write_sigev_signal() {
    let sa = SigAction::new(SigHandler::Handler(sigfunc),
                            SA_RESETHAND,
                            SigSet::empty());
    unsafe {signaled = 0 };
    unsafe { sigaction(Signal::SIGUSR2, &sa) }.unwrap();

    const INITIAL: &'static [u8] = b"abcdef123456";
    const WBUF: &'static [u8] = b"CDEF";
    let mut rbuf = Vec::new();
    const EXPECT: &'static [u8] = b"abCDEF123456";

    let mut f = tempfile().unwrap();
    f.write(INITIAL).unwrap();
    let mut aiocb = unsafe {
        AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           &WBUF,
                           0,   //priority
                           SigevNotify::SigevSignal {
                               signal: Signal::SIGUSR2,
                               si_value: 0  //TODO: validate in sigfunc
                           },
                           LioOpcode::LIO_NOP)
    };
    aio_write(&mut aiocb).unwrap();
    while unsafe { signaled == 0 } {
        thread::sleep(time::Duration::from_millis(10));
    }

    assert!(aio_return(&mut aiocb).unwrap() as usize == WBUF.len());
    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf).unwrap();
    assert!(len == EXPECT.len());
    assert!(rbuf == EXPECT);
}

// Test lio_listio with LIO_WAIT, so all AIO ops should be complete by the time
// lio_listio returns.
#[test]
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn test_lio_listio_wait() {
    const INITIAL: &'static [u8] = b"abcdef123456";
    const WBUF: &'static [u8] = b"CDEF";
    let mut rbuf = vec![0; 4];
    let mut rbuf2 = Vec::new();
    const EXPECT: &'static [u8] = b"abCDEF123456";
    let mut f = tempfile().unwrap();

    f.write(INITIAL).unwrap();

    {
        let mut wcb = unsafe {
            AioCb::from_slice( f.as_raw_fd(),
                               2,   //offset
                               &WBUF,
                               0,   //priority
                               SigevNotify::SigevNone,
                               LioOpcode::LIO_WRITE)
        };

        let mut rcb = AioCb::from_mut_slice( f.as_raw_fd(),
                                8,   //offset
                                &mut rbuf,
                                0,   //priority
                                SigevNotify::SigevNone,
                                LioOpcode::LIO_READ);
        let err = lio_listio(LioMode::LIO_WAIT, &[&mut wcb, &mut rcb], SigevNotify::SigevNone);
        err.expect("lio_listio failed");

        assert!(aio_return(&mut wcb).unwrap() as usize == WBUF.len());
        assert!(aio_return(&mut rcb).unwrap() as usize == WBUF.len());
    }
    assert!(rbuf == b"3456");

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf2).unwrap();
    assert!(len == EXPECT.len());
    assert!(rbuf2 == EXPECT);
}

// Test lio_listio with LIO_NOWAIT and no SigEvent, so we must use some other
// mechanism to check for the individual AioCb's completion.
#[test]
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn test_lio_listio_nowait() {
    const INITIAL: &'static [u8] = b"abcdef123456";
    const WBUF: &'static [u8] = b"CDEF";
    let mut rbuf = vec![0; 4];
    let mut rbuf2 = Vec::new();
    const EXPECT: &'static [u8] = b"abCDEF123456";
    let mut f = tempfile().unwrap();

    f.write(INITIAL).unwrap();

    {
        let mut wcb = unsafe {
            AioCb::from_slice( f.as_raw_fd(),
                               2,   //offset
                               &WBUF,
                               0,   //priority
                               SigevNotify::SigevNone,
                               LioOpcode::LIO_WRITE)
        };

        let mut rcb = AioCb::from_mut_slice( f.as_raw_fd(),
                                8,   //offset
                                &mut rbuf,
                                0,   //priority
                                SigevNotify::SigevNone,
                                LioOpcode::LIO_READ);
        let err = lio_listio(LioMode::LIO_NOWAIT, &[&mut wcb, &mut rcb], SigevNotify::SigevNone);
        err.expect("lio_listio failed");

        poll_aio(&mut wcb).unwrap();
        poll_aio(&mut rcb).unwrap();
        assert!(aio_return(&mut wcb).unwrap() as usize == WBUF.len());
        assert!(aio_return(&mut rcb).unwrap() as usize == WBUF.len());
    }
    assert!(rbuf == b"3456");

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf2).unwrap();
    assert!(len == EXPECT.len());
    assert!(rbuf2 == EXPECT);
}

// Test lio_listio with LIO_NOWAIT and a SigEvent to indicate when all AioCb's
// are complete.
#[test]
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn test_lio_listio_signal() {
    const INITIAL: &'static [u8] = b"abcdef123456";
    const WBUF: &'static [u8] = b"CDEF";
    let mut rbuf = vec![0; 4];
    let mut rbuf2 = Vec::new();
    const EXPECT: &'static [u8] = b"abCDEF123456";
    let mut f = tempfile().unwrap();
    let sa = SigAction::new(SigHandler::Handler(sigfunc),
                            SA_RESETHAND,
                            SigSet::empty());
    let sigev_notify = SigevNotify::SigevSignal { signal: Signal::SIGUSR2,
                                                  si_value: 0 };

    f.write(INITIAL).unwrap();

    {
        let mut wcb = unsafe {
            AioCb::from_slice( f.as_raw_fd(),
                               2,   //offset
                               &WBUF,
                               0,   //priority
                               SigevNotify::SigevNone,
                               LioOpcode::LIO_WRITE)
        };

        let mut rcb = AioCb::from_mut_slice( f.as_raw_fd(),
                                8,   //offset
                                &mut rbuf,
                                0,   //priority
                                SigevNotify::SigevNone,
                                LioOpcode::LIO_READ);
        unsafe {signaled = 0 };
        unsafe { sigaction(Signal::SIGUSR2, &sa) }.unwrap();
        let err = lio_listio(LioMode::LIO_NOWAIT, &[&mut wcb, &mut rcb], sigev_notify);
        err.expect("lio_listio failed");
        while unsafe { signaled == 0 } {
            thread::sleep(time::Duration::from_millis(10));
        }

        assert!(aio_return(&mut wcb).unwrap() as usize == WBUF.len());
        assert!(aio_return(&mut rcb).unwrap() as usize == WBUF.len());
    }
    assert!(rbuf == b"3456");

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf2).unwrap();
    assert!(len == EXPECT.len());
    assert!(rbuf2 == EXPECT);
}
