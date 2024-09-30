use nix::sys::pthread::*;

#[cfg(any(target_env = "musl", target_os = "redox"))]
#[test]
fn test_pthread_self() {
    let tid = pthread_self();
    assert!(!tid.is_null());
}

#[cfg(not(any(target_env = "musl", target_os = "redox")))]
#[test]
fn test_pthread_self() {
    let tid = pthread_self();
    assert_ne!(tid, 0);
}

#[test]
#[cfg(not(target_os = "redox"))]
fn test_pthread_kill_none() {
    pthread_kill(pthread_self(), None)
        .expect("Should be able to send signal to my thread.");
}

#[test]
#[cfg(target_os = "linux")]
fn test_pthread_mutex_wrapper() {
    use nix::{
        sys::{
            mman::{mmap_anonymous, MapFlags, ProtFlags},
            pthread::{Mutex, MutexAttr},
        },
        unistd::{fork, ForkResult},
    };
    use std::{cell::UnsafeCell, mem::size_of, num::NonZeroUsize};
    struct MutexWrapper {
        lock: Mutex,
        data: UnsafeCell<u128>,
    }
    impl MutexWrapper {
        fn add(&self) {
            let guard = self.lock.lock().unwrap();
            unsafe { *self.data.get() += 1 };
            // guard.unlock().unwrap();
            guard.try_unlock().unwrap();
        }
    }
    unsafe impl Sync for MutexWrapper {}

    /// Number of forks to spawn that mutate the data, will spawn `2^FORKS` processes.
    const FORKS: usize = 2;
    /// Number of threads each process spawns that mutate the data, will spawn `2^FORKS * THREADS` threads.
    const THREADS: usize = 10;
    /// Number of iterations each thread mutates the data, will perform `2^FORKS * THREADS * ITERATIONS` iterations.
    const ITERATIONS: usize = 100_000;

    let mut mutex_attr = MutexAttr::new().unwrap();
    mutex_attr.set_shared(true).unwrap();

    let mutex_wrapper = unsafe {
        let mut ptr = mmap_anonymous(
            None,
            NonZeroUsize::new_unchecked(size_of::<MutexWrapper>()),
            ProtFlags::PROT_WRITE | ProtFlags::PROT_READ,
            MapFlags::MAP_SHARED | MapFlags::MAP_ANONYMOUS,
        )
        .unwrap()
        .cast::<MutexWrapper>();
        *ptr.as_mut() = MutexWrapper {
            lock: Mutex::new(Some(mutex_attr)).unwrap(),
            data: UnsafeCell::new(0),
        };
        ptr.as_ref()
    };

    let fork_results = (0..FORKS)
        .map(|_| unsafe { fork().unwrap() })
        .collect::<Vec<_>>();

    let handles = (0..THREADS)
        .map(|_| {
            std::thread::spawn(|| {
                for _ in 0..ITERATIONS {
                    mutex_wrapper.add();
                }
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.join().unwrap();
    }

    // The root process will be the parent in all its fork results.
    let mut root = true;
    for fork_result in fork_results {
        if let ForkResult::Parent { child } = fork_result {
            unsafe {
                assert_eq!(
                    libc::waitpid(child.as_raw(), std::ptr::null_mut(), 0),
                    child.as_raw()
                );
            }
        } else {
            root = false;
        }
    }
    if root {
        let steps =
            2u128.pow(FORKS as u32) * (THREADS as u128) * (ITERATIONS as u128);
        assert_eq!(unsafe { *mutex_wrapper.data.get() }, steps);
    }
}
