use std::{fmt, ops};
use libc::{time_t, suseconds_t};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TimeVal {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}

const MICROS_PER_SEC: i64 = 1_000_000;
const SECS_PER_MINUTE: i64 = 60;
const SECS_PER_HOUR: i64 = 3600;

#[cfg(target_pointer_width = "64")]
const MAX_SECONDS: i64 = (::std::i64::MAX / MICROS_PER_SEC) - 1;

#[cfg(target_pointer_width = "32")]
const MAX_SECONDS: i64 = ::std::isize::MAX as i64;

const MIN_SECONDS: i64 = -MAX_SECONDS;

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
        assert!(seconds >= MIN_SECONDS && seconds <= MAX_SECONDS, "TimeVal out of bounds; seconds={}", seconds);
        TimeVal { tv_sec: seconds as time_t, tv_usec: 0 }
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
        assert!(secs >= MIN_SECONDS && secs <= MAX_SECONDS, "TimeVal out of bounds");
        TimeVal { tv_sec: secs as time_t, tv_usec: micros as suseconds_t }
    }

    pub fn num_hours(&self) -> i64 {
        self.num_seconds() / 3600
    }

    pub fn num_minutes(&self) -> i64 {
        self.num_seconds() / 60
    }

    pub fn num_seconds(&self) -> i64 {
        if self.tv_sec < 0 && self.tv_usec > 0 {
            (self.tv_sec + 1) as i64
        } else {
            self.tv_sec as i64
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
        if self.tv_sec < 0 && self.tv_usec > 0 {
            self.tv_usec - MICROS_PER_SEC as suseconds_t
        } else {
            self.tv_usec
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
        let (abs, sign) = if self.tv_sec < 0 {
            (-*self, "-")
        } else {
            (*self, "")
        };

        let sec = abs.tv_sec;

        try!(write!(f, "{}", sign));

        if abs.tv_usec == 0 {
            if abs.tv_sec == 1 {
                try!(write!(f, "{} second", sec));
            } else {
                try!(write!(f, "{} seconds", sec));
            }
        } else if abs.tv_usec % 1000 == 0 {
            try!(write!(f, "{}.{:03} seconds", sec, abs.tv_usec / 1000));
        } else {
            try!(write!(f, "{}.{:06} seconds", sec, abs.tv_usec));
        }

        Ok(())
    }
}

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
    use super::TimeVal;

    #[test]
    pub fn test_time_val() {
        assert!(TimeVal::seconds(1) != TimeVal::zero());
        assert_eq!(TimeVal::seconds(1) + TimeVal::seconds(2), TimeVal::seconds(3));
        assert_eq!(TimeVal::minutes(3) + TimeVal::seconds(2),
                   TimeVal::seconds(182));
    }

    #[test]
    pub fn test_time_val_neg() {
        let a = TimeVal::seconds(1) + TimeVal::microseconds(123);
        let b = TimeVal::seconds(-1) + TimeVal::microseconds(-123);

        assert_eq!(a, -b);
    }

    #[test]
    pub fn test_time_val_fmt() {
        assert_eq!(TimeVal::zero().to_string(), "0 seconds");
        assert_eq!(TimeVal::seconds(42).to_string(), "42 seconds");
        assert_eq!(TimeVal::milliseconds(42).to_string(), "0.042 seconds");
        assert_eq!(TimeVal::microseconds(42).to_string(), "0.000042 seconds");
        assert_eq!(TimeVal::seconds(-86401).to_string(), "-86401 seconds");
    }
}
