use nix::sys::event::{EventFilter, EventFlag, FilterFlag, KEvent};

#[test]
fn test_struct_kevent() {
    use std::mem;

    let udata: intptr_t = 12345;

    let actual = KEvent::new(
        0xdead_beef,
        EventFilter::EVFILT_READ,
        EventFlag::EV_ONESHOT | EventFlag::EV_ADD,
        FilterFlag::NOTE_CHILD | FilterFlag::NOTE_EXIT,
        0x1337,
        udata,
    );
    assert_eq!(0xdead_beef, actual.ident());
    let filter = actual.kevent.filter;
    assert_eq!(libc::EVFILT_READ, filter);
    assert_eq!(libc::EV_ONESHOT | libc::EV_ADD, actual.flags().bits());
    assert_eq!(libc::NOTE_CHILD | libc::NOTE_EXIT, actual.fflags().bits());
    assert_eq!(0x1337, actual.data());
    assert_eq!(udata as type_of_udata, actual.udata() as type_of_udata);
    assert_eq!(mem::size_of::<libc::kevent>(), mem::size_of::<KEvent>());
}

#[test]
fn test_kevent_filter() {
    let udata: intptr_t = 12345;

    let actual = KEvent::new(
        0xdead_beef,
        EventFilter::EVFILT_READ,
        EventFlag::EV_ONESHOT | EventFlag::EV_ADD,
        FilterFlag::NOTE_CHILD | FilterFlag::NOTE_EXIT,
        0x1337,
        udata,
    );
    assert_eq!(EventFilter::EVFILT_READ, actual.filter().unwrap());
}
