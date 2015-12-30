use nix::poll::*;
use nix::unistd::{write, pipe};

#[test]
fn test_poll() {
    let (r, w) = pipe().unwrap();
    let mut fds = [PollFd {
        fd: r,
        events: POLLIN,
        revents: EventFlags::empty()
    }];

    let nfds = poll(&mut fds, 100).unwrap();
    assert_eq!(nfds, 0);
    assert!(!fds[0].revents.contains(POLLIN));

    write(w, b".").unwrap();

    let nfds = poll(&mut fds, 100).unwrap();
    assert_eq!(nfds, 1);
    assert!(fds[0].revents.contains(POLLIN));
}
