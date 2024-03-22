#[cfg(any(bsd, target_os = "linux"))]
pub mod sem;
#[cfg(any(bsd, target_os = "linux"))]
pub mod shm;
