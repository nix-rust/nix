use std::{fmt, ops};
use libc::{time_t, c_long, suseconds_t, timeval, timespec};

const NANOS_PER_SEC: i64 = 1_000_000_000;
const MICROS_PER_SEC: i64 = 1_000_000;
const SECS_PER_MINUTE: i64 = 60;
const SECS_PER_HOUR: i64 = 3600;

#[derive(Clone, Copy)]
pub struct TimeVal(pub timeval);

impl AsRef<timeval> for TimeVal {
    fn as_ref(&self) -> &timeval {
        &self.0
    }
}

impl AsMut<timeval> for TimeVal {
    fn as_mut(&mut self) -> &mut timeval {
        &mut self.0
    }
}

impl TimeVal {
    #[inline]
    pub fn zero() -> TimeVal {
        TimeVal::microseconds(0)
    }

    #[inline]
    pub fn hours(hours: i64) -> TimeVal {
        let secs = hours.checked_mul(SECS_PER_HOUR)
            .expect("TimeVal::hours ouf of bounds");

        TimeVal::seconds(secs)
    }

    #[inline]
    pub fn minutes(minutes: i64) -> TimeVal {
        let secs = minutes.checked_mul(SECS_PER_MINUTE)
            .expect("TimeVal::minutes out of bounds");

        TimeVal::seconds(secs)
    }

    #[inline]
    pub fn seconds(seconds: i64) -> TimeVal {
        assert!(seconds >= time_t::min_value() && seconds <= time_t::max_value(), "TimeVal out of bounds; seconds={}", seconds);
        TimeVal(timeval { tv_sec: seconds as time_t, tv_usec: 0 })
    }

    #[inline]
    pub fn milliseconds(milliseconds: i64) -> TimeVal {
        let microseconds = milliseconds.checked_mul(1_000)
            .expect("TimeVal::milliseconds out of bounds");

        TimeVal::microseconds(microseconds)
    }

    /// Makes a new `TimeVal` with given number of microseconds.
    #[inline]
    pub fn microseconds(microseconds: i64) -> TimeVal {
        let (secs, micros) = div_mod_floor_64(microseconds, MICROS_PER_SEC);
        assert!(secs >= time_t::min_value() && secs <= time_t::max_value(), "TimeVal out of bounds; seconds={}", secs);
        TimeVal(timeval { tv_sec: secs as time_t, tv_usec: micros as suseconds_t })
    }

    pub fn num_hours(&self) -> i64 {
        self.num_seconds() / 3600
    }

    pub fn num_minutes(&self) -> i64 {
        self.num_seconds() / 60
    }

    pub fn num_seconds(&self) -> i64 {
        if self.0.tv_sec < 0 && self.0.tv_usec > 0 {
            (self.0.tv_sec + 1) as i64
        } else {
            self.0.tv_sec as i64
        }
    }

    pub fn num_milliseconds(&self) -> i64 {
        self.num_microseconds() / 1_000
    }

    pub fn num_microseconds(&self) -> i64 {
        let secs = self.num_seconds() * 1_000_000;
        let usec = self.micros_mod_sec();
        secs + usec as i64
    }

    fn micros_mod_sec(&self) -> suseconds_t {
        if self.0.tv_sec < 0 && self.0.tv_usec > 0 {
            self.0.tv_usec - MICROS_PER_SEC as suseconds_t
        } else {
            self.0.tv_usec
        }
    }
}

impl ops::Neg for TimeVal {
    type Output = TimeVal;

    fn neg(self) -> TimeVal {
        TimeVal::microseconds(-self.num_microseconds())
    }
}

impl ops::Add for TimeVal {
    type Output = TimeVal;

    fn add(self, rhs: TimeVal) -> TimeVal {
        TimeVal::microseconds(
            self.num_microseconds() + rhs.num_microseconds())
    }
}

impl ops::Sub for TimeVal {
    type Output = TimeVal;

    fn sub(self, rhs: TimeVal) -> TimeVal {
        TimeVal::microseconds(
            self.num_microseconds() - rhs.num_microseconds())
    }
}

impl ops::Mul<i32> for TimeVal {
    type Output = TimeVal;

    fn mul(self, rhs: i32) -> TimeVal {
        let usec = self.num_microseconds().checked_mul(rhs as i64)
            .expect("TimeVal multiply out of bounds");

        TimeVal::microseconds(usec)
    }
}

impl ops::Div<i32> for TimeVal {
    type Output = TimeVal;

    fn div(self, rhs: i32) -> TimeVal {
        let usec = self.num_microseconds() / rhs as i64;
        TimeVal::microseconds(usec)
    }
}

impl fmt::Display for TimeVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (abs, sign) = if self.0.tv_sec < 0 {
            (-*self, "-")
        } else {
            (*self, "")
        };

        let sec = abs.0.tv_sec;

        try!(write!(f, "{}", sign));

        if abs.0.tv_usec == 0 {
            if abs.0.tv_sec == 1 {
                try!(write!(f, "{} second", sec));
            } else {
                try!(write!(f, "{} seconds", sec));
            }
        } else if abs.0.tv_usec % 1_000 == 0 {
            try!(write!(f, "{}.{:03} seconds", sec, abs.0.tv_usec / 1_000));
        } else {
            try!(write!(f, "{}.{:06} seconds", sec, abs.0.tv_usec));
        }

        Ok(())
    }
}

impl PartialEq for TimeVal {
    fn eq(&self, rhs: &TimeVal) -> bool {
        self.0.tv_sec == rhs.0.tv_sec && self.0.tv_usec == rhs.0.tv_usec
    }
}

impl Eq for TimeVal { }

#[derive(Clone, Copy)]
pub struct TimeSpec(pub timespec);

impl AsRef<timespec> for TimeSpec {
    fn as_ref(&self) -> &timespec {
        &self.0
    }
}

impl TimeSpec {
    #[inline]
    pub fn zero() -> TimeSpec {
        TimeSpec::nanoseconds(0)
    }

    #[inline]
    pub fn hours(hours: i64) -> TimeSpec {
        let secs = hours.checked_mul(SECS_PER_HOUR)
            .expect("TimeSpec::hours ouf of bounds");

        TimeSpec::seconds(secs)
    }

    #[inline]
    pub fn minutes(minutes: i64) -> TimeSpec {
        let secs = minutes.checked_mul(SECS_PER_MINUTE)
            .expect("TimeSpec::minutes out of bounds");

        TimeSpec::seconds(secs)
    }

    #[inline]
    pub fn seconds(seconds: i64) -> TimeSpec {
        assert!(seconds >= time_t::min_value() && seconds <= time_t::max_value(), "TimeSpec out of bounds; seconds={}", seconds);
        TimeSpec(timespec { tv_sec: seconds as time_t, tv_nsec: 0 })
    }

    #[inline]
    pub fn milliseconds(milliseconds: i64) -> TimeSpec {
        let nanoseconds = milliseconds.checked_mul(1_000_000)
            .expect("TimeSpec::milliseconds out of bounds");

        TimeSpec::nanoseconds(nanoseconds)
    }

    #[inline]
    pub fn microseconds(microseconds: i64) -> TimeSpec {
        let nanoseconds = microseconds.checked_mul(1_000)
            .expect("TimeSpec::microseconds out of bounds");

        TimeSpec::nanoseconds(nanoseconds)
    }


    /// Makes a new `TimeSpec` with given number of nanoseconds.
    #[inline]
    pub fn nanoseconds(nanoseconds: i64) -> TimeSpec {
        let (secs, nanos) = div_mod_floor_64(nanoseconds, NANOS_PER_SEC);
        assert!(secs >= time_t::min_value() && secs <= time_t::max_value(), "TimeSpec out of bounds; seconds={}", secs);
        TimeSpec(timespec { tv_sec: secs as time_t, tv_nsec: nanos as c_long })
    }

    pub fn num_hours(&self) -> i64 {
        self.num_seconds() / 3600
    }

    pub fn num_minutes(&self) -> i64 {
        self.num_seconds() / 60
    }

    pub fn num_seconds(&self) -> i64 {
        if self.0.tv_sec < 0 && self.0.tv_nsec > 0 {
            (self.0.tv_sec + 1) as i64
        } else {
            self.0.tv_sec as i64
        }
    }

    pub fn num_milliseconds(&self) -> i64 {
        self.num_nanoseconds() / 1_000_000
    }

    pub fn num_microseconds(&self) -> i64 {
        self.num_nanoseconds() / 1_000
    }

    pub fn num_nanoseconds(&self) -> i64 {
        let secs = self.num_seconds() * NANOS_PER_SEC;
        let nsec = self.nanos_mod_sec();
        secs + nsec as i64
    }

    fn nanos_mod_sec(&self) -> suseconds_t {
        if self.0.tv_sec < 0 && self.0.tv_nsec > 0 {
            self.0.tv_nsec - NANOS_PER_SEC as c_long
        } else {
            self.0.tv_nsec
        }
    }
}

impl ops::Neg for TimeSpec {
    type Output = TimeSpec;

    fn neg(self) -> TimeSpec {
        TimeSpec::nanoseconds(-self.num_nanoseconds())
    }
}

impl ops::Add for TimeSpec {
    type Output = TimeSpec;

    fn add(self, rhs: TimeSpec) -> TimeSpec {
        TimeSpec::nanoseconds(
            self.num_nanoseconds() + rhs.num_nanoseconds())
    }
}

impl ops::Sub for TimeSpec {
    type Output = TimeSpec;

    fn sub(self, rhs: TimeSpec) -> TimeSpec {
        TimeSpec::nanoseconds(
            self.num_nanoseconds() - rhs.num_nanoseconds())
    }
}

impl ops::Mul<i32> for TimeSpec {
    type Output = TimeSpec;

    fn mul(self, rhs: i32) -> TimeSpec {
        let nsec = self.num_nanoseconds().checked_mul(rhs as i64)
            .expect("TimeSpec multiply out of bounds");

        TimeSpec::nanoseconds(nsec)
    }
}

impl ops::Div<i32> for TimeSpec {
    type Output = TimeSpec;

    fn div(self, rhs: i32) -> TimeSpec {
        let nsec = self.num_nanoseconds() / rhs as i64;
        TimeSpec::nanoseconds(nsec)
    }
}

impl fmt::Display for TimeSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (abs, sign) = if self.0.tv_sec < 0 {
            (-*self, "-")
        } else {
            (*self, "")
        };

        let sec = abs.0.tv_sec;

        try!(write!(f, "{}", sign));

        if abs.0.tv_nsec == 0 {
            if abs.0.tv_sec == 1 {
                try!(write!(f, "{} second", sec));
            } else {
                try!(write!(f, "{} seconds", sec));
            }
        } else if abs.0.tv_nsec % 1_000_000 == 0 {
            try!(write!(f, "{}.{:03} seconds", sec, abs.0.tv_nsec / 1_000_000));
        } else if abs.0.tv_nsec % 1_000 == 0 {
            try!(write!(f, "{}.{:06} seconds", sec, abs.0.tv_nsec / 1_000));
        } else {
            try!(write!(f, "{}.{:09} seconds", sec, abs.0.tv_nsec));
        }

        Ok(())
    }
}

impl PartialEq for TimeSpec {
    fn eq(&self, rhs: &TimeSpec) -> bool {
        self.0.tv_sec == rhs.0.tv_sec && self.0.tv_nsec == rhs.0.tv_nsec
    }
}

impl Eq for TimeSpec { }

#[inline]
fn div_mod_floor_64(this: i64, other: i64) -> (i64, i64) {
    (div_floor_64(this, other), mod_floor_64(this, other))
}

#[inline]
fn div_floor_64(this: i64, other: i64) -> i64 {
    match div_rem_64(this, other) {
        (d, r) if (r > 0 && other < 0)
               || (r < 0 && other > 0) => d - 1,
        (d, _)                         => d,
    }
}

#[inline]
fn mod_floor_64(this: i64, other: i64) -> i64 {
    match this % other {
        r if (r > 0 && other < 0)
          || (r < 0 && other > 0) => r + other,
        r                         => r,
    }
}

#[inline]
fn div_rem_64(this: i64, other: i64) -> (i64, i64) {
    (this / other, this % other)
}

#[cfg(test)]
mod test {
    use super::{TimeVal, TimeSpec};

    #[test]
    pub fn test_time_val() {
        assert!(TimeVal::seconds(1) != TimeVal::zero());
        assert!(TimeVal::seconds(1) + TimeVal::seconds(2) == TimeVal::seconds(3));
        assert!(TimeVal::minutes(3) + TimeVal::seconds(2) == TimeVal::seconds(182));
    }

    #[test]
    pub fn test_time_val_neg() {
        let a = TimeVal::seconds(1) + TimeVal::microseconds(123);
        let b = TimeVal::seconds(-1) + TimeVal::microseconds(-123);

        assert!(a == -b);
    }

    #[test]
    pub fn test_time_val_fmt() {
        assert_eq!(TimeVal::zero().to_string(), "0 seconds");
        assert_eq!(TimeVal::seconds(42).to_string(), "42 seconds");
        assert_eq!(TimeVal::milliseconds(42).to_string(), "0.042 seconds");
        assert_eq!(TimeVal::microseconds(42).to_string(), "0.000042 seconds");
        assert_eq!(TimeVal::seconds(-86401).to_string(), "-86401 seconds");
    }

    #[test]
    pub fn test_time_spec() {
        assert!(TimeSpec::seconds(1) != TimeSpec::zero());
        assert!(TimeSpec::seconds(1) + TimeSpec::seconds(2) == TimeSpec::seconds(3));
        assert!(TimeSpec::minutes(3) + TimeSpec::seconds(2) == TimeSpec::seconds(182));
    }

    #[test]
    pub fn test_time_spec_neg() {
        let a = TimeSpec::seconds(1) + TimeSpec::nanoseconds(123);
        let b = TimeSpec::seconds(-1) + TimeSpec::nanoseconds(-123);

        assert!(a == -b);
    }

    #[test]
    pub fn test_time_spec_fmt() {
        assert_eq!(TimeSpec::zero().to_string(), "0 seconds");
        assert_eq!(TimeSpec::seconds(42).to_string(), "42 seconds");
        assert_eq!(TimeSpec::milliseconds(42).to_string(), "0.042 seconds");
        assert_eq!(TimeSpec::microseconds(42).to_string(), "0.000042 seconds");
        assert_eq!(TimeSpec::nanoseconds(42).to_string(), "0.000000042 seconds");
        assert_eq!(TimeSpec::seconds(-86401).to_string(), "-86401 seconds");
    }
}
