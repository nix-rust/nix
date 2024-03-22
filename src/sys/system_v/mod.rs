//! Interact with SystemV inter-process communication API

#[cfg(any(bsd, target_os = "linux"))]
pub mod sem;
#[cfg(any(bsd, target_os = "linux"))]
pub mod shm;
