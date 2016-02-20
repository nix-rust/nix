use nix::errno::Errno;
use nix::sys::termios;
use nix::unistd;

#[test]
fn test_tcgetattr() {
    for fd in 0..5 {
        let termios = termios::tcgetattr(fd);
        match unistd::isatty(fd) {
            // If `fd` is a TTY, tcgetattr must succeed.
            Ok(true) => assert!(termios.is_ok()),
            // If it's an invalid file descriptor, tcgetattr should also return
            // the same error
            Err(Errno::EBADF) => {
                assert_eq!(termios.err(), Some(Errno::EBADF));
            },
            // Otherwise it should return any error
            _ => assert!(termios.is_err())
        }
    }
}
