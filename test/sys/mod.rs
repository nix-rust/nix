#[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd",
    target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd"))]
mod test_event;
mod test_socket;
mod test_sockopt;
mod test_termios;
mod test_ioctl;
mod test_wait;
mod test_select;
mod test_uio;
