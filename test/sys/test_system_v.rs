use nix::{errno::Errno, sys::system_v::*, Result};

use crate::SYSTEMV_MTX;

const IPC_TEST: i32 = 1337;

/// Test struct used to try storing data on the IPC
///
struct IpcStruct {
    pub test_info: i64,
}

#[derive(Debug)]
/// RAII fixture that delete the SystemV IPC and Semaphore on drop
///
struct FixtureSystemV {
    pub id_ipc: i32,
    pub id_sem: i32,
}

impl FixtureSystemV {
    fn setup() -> Result<FixtureSystemV> {
        shmget(
            IPC_TEST,
            std::mem::size_of::<IpcStruct>(),
            vec![ShmgetFlag::IPC_CREAT, ShmgetFlag::IPC_EXCL],
            Permissions::new(0o0777).expect("Octal is smaller than u9"),
        )?;
        semget(
            IPC_TEST,
            1,
            vec![SemgetFlag::IPC_CREAT, SemgetFlag::IPC_EXCL],
            Permissions::new(0o0777).expect("Octal is smaller than u9"),
        )?;
        Ok(Self {
            id_ipc: shmget(
                IPC_TEST,
                0,
                vec![],
                Permissions::new(0o0).expect("Octal is smaller than u9"),
            )
            .expect("IPC exist"),
            id_sem: semget(
                IPC_TEST,
                0,
                vec![],
                Permissions::new(0o0).expect("Octal is smaller than u9"),
            )
            .expect("Sem exist"),
        })
    }
}

impl Drop for FixtureSystemV {
    fn drop(&mut self) {
        let _ = shmctl(
            self.id_ipc,
            ShmctlFlag::IPC_RMID,
            None,
            Permissions::new(0o0).expect("Octal is smaller than u9"),
        )
        .map_err(|_| panic!("Failed to delete the test IPC"));
        let _ = semctl(
            self.id_sem,
            0,
            SemctlCmd::IPC_RMID,
            Permissions::new(0o0).expect("Octal is smaller than u9"),
            None,
        )
        .map_err(|_| panic!("Failed to delete the test semaphore"));
    }
}

#[test]
fn create_ipc() -> Result<()> {
    let _m = SYSTEMV_MTX.lock();

    FixtureSystemV::setup()?;
    Ok(())
}

#[test]
fn create_ipc_already_exist() -> Result<()> {
    let _m = SYSTEMV_MTX.lock();

    // Keep the IPC in scope, so we don't destroy it
    let _ipc = FixtureSystemV::setup()?;
    let expected = Errno::EEXIST;
    let actual = FixtureSystemV::setup().expect_err("Return EExist");

    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn create_ipc_and_get_value() -> Result<()> {
    let _m = SYSTEMV_MTX.lock();

    let ipc = FixtureSystemV::setup()?;
    let mem: *mut IpcStruct = shmat(
        ipc.id_ipc,
        None,
        vec![],
        Permissions::new(0o0).expect("Octal is smaller than u9"),
    )?
    .cast();

    let expected = 0xDEADBEEF;
    unsafe {
        mem.as_mut().unwrap().test_info = expected;
        assert_eq!(expected, mem.read().test_info);
    }

    Ok(())
}
