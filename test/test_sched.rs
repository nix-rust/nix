use nix::sched::{sched_getaffinity, sched_getcpu, sched_setaffinity, CpuSet};
use nix::unistd::Pid;

#[test]
fn test_sched_affinity() {
    // If pid is zero, then the mask of the calling process is returned.
    let initial_affinity = sched_getaffinity(Pid::from_raw(0)).unwrap();
    let mut at_least_one_cpu = false;
    let mut last_valid_cpu = 0;
    for field in 0..CpuSet::count() {
        if initial_affinity.is_set(field).unwrap() {
            at_least_one_cpu = true;
            last_valid_cpu = field;
        }
    }
    assert!(at_least_one_cpu);

    // Now restrict the running CPU
    let mut new_affinity = CpuSet::new();
    new_affinity.set(last_valid_cpu).unwrap();
    sched_setaffinity(Pid::from_raw(0), &new_affinity).unwrap();

    // And now re-check the affinity which should be only the one we set.
    let updated_affinity = sched_getaffinity(Pid::from_raw(0)).unwrap();
    for field in 0..CpuSet::count() {
        // Should be set only for the CPU we set previously
        assert_eq!(
            updated_affinity.is_set(field).unwrap(),
            field == last_valid_cpu
        )
    }

    // Now check that we're also currently running on the CPU in question.
    let cur_cpu = sched_getcpu().unwrap();
    assert_eq!(cur_cpu, last_valid_cpu);

    // Finally, reset the initial CPU set
    sched_setaffinity(Pid::from_raw(0), &initial_affinity).unwrap();
}

#[cfg(all(linux_android, not(target_env = "musl"), not(target_env = "ohos")))]
#[test]
fn test_sched_priority() {
    use nix::sched::{
        sched_get_priority_max, sched_get_priority_min, sched_getparam,
        sched_getscheduler, sched_setscheduler, SchedParam, Scheduler,
    };

    let pid = Pid::from_raw(0);
    let sched = sched_getscheduler(pid).unwrap();
    // default is NORMAL aka OTHER
    assert_eq!(sched, Scheduler::SCHED_NORMAL);

    let priority = sched_getparam(pid).unwrap().sched_priority;
    assert_eq!(priority, 0);

    let max = sched_get_priority_max(Scheduler::SCHED_FIFO).unwrap();
    let _ = sched_get_priority_min(Scheduler::SCHED_FIFO).unwrap();

    // can't set priority unless process has correct capabilities and PREEMPT_RT kernel
    match sched_setscheduler(
        pid,
        Scheduler::SCHED_FIFO,
        SchedParam::from_priority(max),
    ) {
        Ok(_) => {
            assert_eq!(sched_getscheduler(pid).unwrap(), Scheduler::SCHED_FIFO);
            assert_eq!(sched_getparam(pid).unwrap().sched_priority, max);
        }
        Err(nix::errno::Errno::EPERM) => {
            // expected, assert that it didn't change
            assert_eq!(
                sched_getscheduler(pid).unwrap(),
                Scheduler::SCHED_NORMAL
            );
        }
        Err(e) => {
            panic!("unexpected error: {}", e);
        }
    }
}
