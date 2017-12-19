mod test_signal;
#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "ios",
          target_os = "netbsd", target_os = "macos", target_os = "linux"))]
mod test_aio;
#[cfg(target_os = "linux")]
mod test_signalfd;
mod test_socket;
mod test_sockopt;
mod test_termios;
mod test_ioctl;
mod test_wait;
mod test_uio;

#[cfg(target_os = "linux")]
mod test_epoll;
mod test_pthread;
#[cfg(any(target_os = "linux", target_os = "android"))]
mod test_ptrace;
#[cfg(any(target_os = "linux", target_os = "android"))]
mod test_prctl;
