use nix::time::{clock_getres, clock_gettime, ClockId};

#[test]
pub fn test_clock_getres() {
    assert!(clock_getres(ClockId::CLOCK_REALTIME).is_ok());
}

#[test]
pub fn test_clock_gettime() {
    assert!(clock_gettime(ClockId::CLOCK_REALTIME).is_ok());
}
