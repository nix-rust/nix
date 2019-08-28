use nix::utmpx::*;

/// Test opening and rewinding the DB.
#[test]
fn test_utmpx_open_rewind() {
    let _m = crate::UTMPX_MTX
        .lock()
        .expect("Mutex got poisoned by another test");
    unsafe {
        let mut db = Utmp::open().unwrap();
        db.rewind();
    }
}

/// Test iterating through default DB. Cross-jobs in Travis
/// seems to have an empty DB, so it's disabled there.
#[cfg_attr(travis, ignore)]
#[test]
fn test_iter() {
    let _m = crate::UTMPX_MTX
        .lock()
        .expect("Mutex got poisoned by another test");
    let mut db = unsafe { Utmp::open().unwrap() };

    let mut entries = 0u64;
    let mut found_booted = false;
    for line in db.entries() {
        entries += 1;
        if let Ok(entry) = line {
            if *entry.entry_type() == EntryType::BootTime {
                found_booted |= true;
            }
        }
    }

    // Invariant: the system booted, thus there must be at least one BOOT_TIME entry.
    assert!(entries > 0);
    assert_eq!(found_booted, true);
}
