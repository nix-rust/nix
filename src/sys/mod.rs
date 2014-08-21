
#[cfg(target_os = "linux")]
pub mod epoll;

#[cfg(target_os = "macos")]
#[cfg(target_os = "ios")]
pub mod event;

#[cfg(target_os = "linux")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "ios")]
pub mod socket;

#[cfg(target_os = "linux")]
pub mod stat;

#[cfg(target_os = "linux")]
pub mod utsname;
