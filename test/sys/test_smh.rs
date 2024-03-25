use std::ptr;

use nix::errno::Errno;
use nix::sys::shm::*;
use nix::sys::stat::Mode;
use nix::Result;

use crate::SYSTEMV_MTX;

const SHM_TEST: i32 = 1337;

#[derive(Debug, Default)]
/// Test struct used to store some data on the shared memory segment
///
struct TestData {
    data: i64,
}

#[derive(Debug)]
struct FixtureShm {
    shm: Shm<TestData>,
    memory: SharedMemory<TestData>,
}

impl FixtureShm {
    fn setup() -> Result<Self> {
        let shm = Shm::<TestData>::create_and_connect(
            SHM_TEST,
            Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO,
        )?;
        let memory = shm.attach(ptr::null(), ShmatFlag::empty())?;
        Ok(Self { shm, memory })
    }
}

impl Drop for FixtureShm {
    fn drop(&mut self) {
        let _ = self.shm.shmctl(ShmctlFlag::IPC_RMID, None).map_err(|_| {
            panic!("Failed to delete the test shared memory segment")
        });
    }
}

#[test]
fn create_ipc() -> Result<()> {
    let _m = SYSTEMV_MTX.lock();

    FixtureShm::setup()?;
    Ok(())
}

#[test]
fn create_ipc_already_exist() -> Result<()> {
    let _m = SYSTEMV_MTX.lock();

    // Keep the IPC in scope, so we don't destroy it
    let _fixture = FixtureShm::setup()?;
    let expected = Errno::EEXIST;
    let actual = FixtureShm::setup().expect_err("Return EExist");

    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn create_ipc_and_get_value() -> Result<()> {
    let _m = SYSTEMV_MTX.lock();

    let mut fixture = FixtureShm::setup()?;
    let expected = 0xDEADBEEF;
    fixture.memory.data = expected;

    let actual = fixture.shm.attach(ptr::null(), ShmatFlag::empty())?.data;
    assert_eq!(expected, actual);
    Ok(())
}
