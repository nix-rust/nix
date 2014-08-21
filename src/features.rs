pub use self::os::*;

#[cfg(target_os = "linux")]
mod os {
    pub fn atomic_cloexec() -> bool {
        true // TODO: Not on all kernel versions
    }
}

#[cfg(target_os = "macos")]
#[cfg(target_os = "ios")]
mod os {
    pub fn atomic_cloexec() -> bool {
        false
    }
}
