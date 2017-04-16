use libc::c_int;
use nix::{Error, Result};
use nix::errno::*;
use nix::sys::aio::*;
use nix::sys::signal::*;
use nix::sys::time::{TimeSpec, TimeValLike};
use std::io::{Write, Read, Seek, SeekFrom};
use std::os::unix::io::AsRawFd;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time};
use tempfile::tempfile;

// Helper that polls an AioCb for completion or error
fn poll_aio(mut aiocb: &mut AioCb) -> Result<()> {
    loop {
        let err = aiocb.error();
        if err != Err(Error::from(Errno::EINPROGRESS)) { return err; };
        thread::sleep(time::Duration::from_millis(10));
    }
}

// Tests AioCb.cancel.  We aren't trying to test the OS's implementation, only our
// bindings.  So it's sufficient to check that AioCb.cancel returned any
// AioCancelStat value.
#[test]
fn test_cancel() {
    let mut wbuf = "CDEF".to_string().into_bytes();

    let f = tempfile().unwrap();
    let mut aiocb = AioCb::from_mut_slice( f.as_raw_fd(),
                            0,   //offset
                            &mut wbuf,
                            0,   //priority
                            SigevNotify::SigevNone,
                            LioOpcode::LIO_NOP);
    aiocb.write().unwrap();
    let err = aiocb.error();
    assert!(err == Ok(()) || err == Err(Error::from(Errno::EINPROGRESS)));

    let cancelstat = aiocb.cancel();
    assert!(cancelstat.is_ok());

    // Wait for aiocb to complete, but don't care whether it succeeded
    let _ = poll_aio(&mut aiocb);
    let _ = aiocb.aio_return();
}

// Tests using aio_cancel_all for all outstanding IOs.
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
    aiocb.write().unwrap();
    let err = aiocb.error();
    assert!(err == Ok(()) || err == Err(Error::from(Errno::EINPROGRESS)));

    let cancelstat = aio_cancel_all(f.as_raw_fd());
    assert!(cancelstat.is_ok());

    // Wait for aiocb to complete, but don't care whether it succeeded
    let _ = poll_aio(&mut aiocb);
    let _ = aiocb.aio_return();
}

#[test]
fn test_fsync() {
    const INITIAL: &'static [u8] = b"abcdef123456";
    let mut f = tempfile().unwrap();
    f.write(INITIAL).unwrap();
    let mut aiocb = AioCb::from_fd( f.as_raw_fd(),
                            0,   //priority
                            SigevNotify::SigevNone);
    let err = aiocb.fsync(AioFsyncMode::O_SYNC);
    assert!(err.is_ok());
    poll_aio(&mut aiocb).unwrap();
    aiocb.aio_return().unwrap();
}


#[test]
fn test_aio_suspend() {
    const INITIAL: &'static [u8] = b"abcdef123456";
    const WBUF: &'static [u8] = b"CDEF";
    let timeout = TimeSpec::seconds(10);
    let mut rbuf = vec![0; 4];
    let mut f = tempfile().unwrap();
    f.write(INITIAL).unwrap();

    let mut wcb = AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           &mut WBUF,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_WRITE);

    let mut rcb = AioCb::from_mut_slice( f.as_raw_fd(),
                            8,   //offset
                            &mut rbuf,
                            0,   //priority
                            SigevNotify::SigevNone,
                            LioOpcode::LIO_READ);
    wcb.write().unwrap();
    rcb.read().unwrap();
    loop {
        {
            let cbbuf = [&wcb, &rcb];
            assert!(aio_suspend(&cbbuf[..], Some(timeout)).is_ok());
        }
        if rcb.error() != Err(Error::from(Errno::EINPROGRESS)) &&
           wcb.error() != Err(Error::from(Errno::EINPROGRESS)) {
            break
        }
    }

    assert!(wcb.aio_return().unwrap() as usize == WBUF.len());
    assert!(rcb.aio_return().unwrap() as usize == WBUF.len());
}

// Test a simple aio operation with no completion notification.  We must poll
// for completion
#[test]
fn test_read() {
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
        aiocb.read().unwrap();

        let err = poll_aio(&mut aiocb);
        assert!(err == Ok(()));
        assert!(aiocb.aio_return().unwrap() as usize == EXPECT.len());
    }

    assert!(rbuf == EXPECT);
}

// Test reading into an immutable buffer.  It should fail
#[test]
#[should_panic(expected = "Can't read into an immutable buffer")]
fn test_read_immutable_buffer() {
    let rbuf = vec![0; 4];
    let f = tempfile().unwrap();
    let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           &rbuf,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_NOP);
    aiocb.read().unwrap();
}

// Test a simple aio operation with no completion notification.  We must poll
// for completion.  Unlike test_aio_read, this test uses AioCb::from_slice
#[test]
fn test_write() {
    const INITIAL: &'static [u8] = b"abcdef123456";
    const WBUF: &'static [u8] = b"CDEF"; //"CDEF".to_string().into_bytes();
    let mut rbuf = Vec::new();
    const EXPECT: &'static [u8] = b"abCDEF123456";

    let mut f = tempfile().unwrap();
    f.write(INITIAL).unwrap();
    let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           &WBUF,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_NOP);
    aiocb.write().unwrap();

    let err = poll_aio(&mut aiocb);
    assert!(err == Ok(()));
    assert!(aiocb.aio_return().unwrap() as usize == WBUF.len());

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf).unwrap();
    assert!(len == EXPECT.len());
    assert!(rbuf == EXPECT);
}

lazy_static! {
    pub static ref SIGNALED: AtomicBool = AtomicBool::new(false);
    // protects access to SIGUSR2 handlers, not just SIGNALED
    pub static ref SIGUSR2_MTX: Mutex<()> = Mutex::new(());
}

extern fn sigfunc(_: c_int) {
    SIGNALED.store(true, Ordering::Relaxed);
}

// Test an aio operation with completion delivered by a signal
#[test]
fn test_write_sigev_signal() {
    let _ = SIGUSR2_MTX.lock().expect("Mutex got poisoned by another test");
    let sa = SigAction::new(SigHandler::Handler(sigfunc),
                            SA_RESETHAND,
                            SigSet::empty());
    SIGNALED.store(false, Ordering::Relaxed);
    unsafe { sigaction(Signal::SIGUSR2, &sa) }.unwrap();

    const INITIAL: &'static [u8] = b"abcdef123456";
    const WBUF: &'static [u8] = b"CDEF";
    let mut rbuf = Vec::new();
    const EXPECT: &'static [u8] = b"abCDEF123456";

    let mut f = tempfile().unwrap();
    f.write(INITIAL).unwrap();
    let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           &WBUF,
                           0,   //priority
                           SigevNotify::SigevSignal {
                               signal: Signal::SIGUSR2,
                               si_value: 0  //TODO: validate in sigfunc
                           },
                           LioOpcode::LIO_NOP);
    aiocb.write().unwrap();
    while SIGNALED.load(Ordering::Relaxed) == false {
        thread::sleep(time::Duration::from_millis(10));
    }

    assert!(aiocb.aio_return().unwrap() as usize == WBUF.len());
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
        let mut wcb = AioCb::from_slice( f.as_raw_fd(),
                               2,   //offset
                               &WBUF,
                               0,   //priority
                               SigevNotify::SigevNone,
                               LioOpcode::LIO_WRITE);

        let mut rcb = AioCb::from_mut_slice( f.as_raw_fd(),
                                8,   //offset
                                &mut rbuf,
                                0,   //priority
                                SigevNotify::SigevNone,
                                LioOpcode::LIO_READ);
        let err = lio_listio(LioMode::LIO_WAIT, &[&mut wcb, &mut rcb], SigevNotify::SigevNone);
        err.expect("lio_listio failed");

        assert!(wcb.aio_return().unwrap() as usize == WBUF.len());
        assert!(rcb.aio_return().unwrap() as usize == WBUF.len());
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
        let mut wcb = AioCb::from_slice( f.as_raw_fd(),
                               2,   //offset
                               &WBUF,
                               0,   //priority
                               SigevNotify::SigevNone,
                               LioOpcode::LIO_WRITE);

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
        assert!(wcb.aio_return().unwrap() as usize == WBUF.len());
        assert!(rcb.aio_return().unwrap() as usize == WBUF.len());
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
    let _ = SIGUSR2_MTX.lock().expect("Mutex got poisoned by another test");
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
        let mut wcb = AioCb::from_slice( f.as_raw_fd(),
                               2,   //offset
                               &WBUF,
                               0,   //priority
                               SigevNotify::SigevNone,
                               LioOpcode::LIO_WRITE);

        let mut rcb = AioCb::from_mut_slice( f.as_raw_fd(),
                                8,   //offset
                                &mut rbuf,
                                0,   //priority
                                SigevNotify::SigevNone,
                                LioOpcode::LIO_READ);
        SIGNALED.store(false, Ordering::Relaxed);
        unsafe { sigaction(Signal::SIGUSR2, &sa) }.unwrap();
        let err = lio_listio(LioMode::LIO_NOWAIT, &[&mut wcb, &mut rcb], sigev_notify);
        err.expect("lio_listio failed");
        while SIGNALED.load(Ordering::Relaxed) == false {
            thread::sleep(time::Duration::from_millis(10));
        }

        assert!(wcb.aio_return().unwrap() as usize == WBUF.len());
        assert!(rcb.aio_return().unwrap() as usize == WBUF.len());
    }
    assert!(rbuf == b"3456");

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf2).unwrap();
    assert!(len == EXPECT.len());
    assert!(rbuf2 == EXPECT);
}

// Try to use lio_listio to read into an immutable buffer.  It should fail
#[test]
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[should_panic(expected = "Can't read into an immutable buffer")]
fn test_lio_listio_read_immutable() {
    let rbuf = vec![0; 4];
    let f = tempfile().unwrap();


    let mut rcb = AioCb::from_slice( f.as_raw_fd(),
                           2,   //offset
                           &rbuf,
                           0,   //priority
                           SigevNotify::SigevNone,
                           LioOpcode::LIO_READ);
    let _ = lio_listio(LioMode::LIO_NOWAIT, &[&mut rcb], SigevNotify::SigevNone);
}

// Test dropping an AioCb that hasn't yet finished.  Behind the scenes, the
// library should wait for the AioCb's completion.
#[test]
fn test_drop() {
    const INITIAL: &'static [u8] = b"abcdef123456";
    const WBUF: &'static [u8] = b"CDEF"; //"CDEF".to_string().into_bytes();
    let mut rbuf = Vec::new();
    const EXPECT: &'static [u8] = b"abCDEF123456";

    let mut f = tempfile().unwrap();
    f.write(INITIAL).unwrap();
    {
        let mut aiocb = AioCb::from_slice( f.as_raw_fd(),
                               2,   //offset
                               &WBUF,
                               0,   //priority
                               SigevNotify::SigevNone,
                               LioOpcode::LIO_NOP);
        aiocb.write().unwrap();
    }

    f.seek(SeekFrom::Start(0)).unwrap();
    let len = f.read_to_end(&mut rbuf).unwrap();
    assert!(len == EXPECT.len());
    assert!(rbuf == EXPECT);
}
