use std::ops::{Add, Sub};
use std::fmt::Debug;

use chrono::{TimeDelta, prelude::*};
use error_stack::{Report, ResultExt};

#[derive(Debug, thiserror::Error)]
#[error("time error")]
pub struct TimeError;

pub type TimeResult<T> = Result<T, Report<TimeError>>;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSpan(u64);

impl Debug for TimeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nt = NaiveTime::from_num_seconds_from_midnight_opt(self.0 as u32, 0).unwrap();
        write!(f, "{}", nt.format("%H:%M:%S"))
    }
}

impl From<std::time::Duration> for TimeSpan {
    fn from(value: std::time::Duration) -> Self {
        Self(value.as_secs())
    }
}

impl From<TimeSpan> for TimeDelta {
    fn from(value: TimeSpan) -> Self {
        TimeDelta::seconds(value.0 as i64)
    }
}

impl From<TimeDelta> for TimeSpan {
    fn from(value: TimeDelta) -> Self {
        Self(value.abs().num_seconds() as u64)
    }
}

impl From<&TimeSpan> for TimeDelta {
    fn from(value: &TimeSpan) -> TimeDelta {
        TimeDelta::seconds(value.0 as i64)
    }
}

impl From<NaiveTime> for TimeSpan {
    fn from(value: NaiveTime) -> Self {
        value.signed_duration_since(NaiveTime::MIN).into()
    }
}

impl TimeSpan {
    pub const ZERO: Self = Self(0);

    pub fn new(hours: u8, minutes: u8, seconds: u8) -> Self {
        Self(seconds as u64 + (minutes as u64 * 60) + (hours as u64 * 60 * 60))
    }

    pub fn is_zero(&self) -> bool {
        self == &TimeSpan::ZERO
    }

    pub fn seconds(&self) -> u64 {
        self.0 % 60
    }

    pub fn minutes(&self) -> u64 {
        (self.0 % 3600) / 60
    }

    pub fn hours(&self) -> u64 {
        self.0 / 3600
    }
}

impl Add<Self> for TimeSpan {
    type Output = TimeSpan;

    fn add(self, rhs: Self) -> Self::Output {
        TimeSpan(self.0 + rhs.0)
    }
}

impl Sub for TimeSpan {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        TimeSpan(self.0 - rhs.0)
    }
}

pub fn normalize(timestamp: DateTime<Local>) -> TimeResult<DateTime<Local>> {
    timestamp.with_nanosecond(0).ok_or(Report::new(TimeError)).attach("invalid timestamp")
}

pub fn now() -> TimeResult<DateTime<Local>> {
    normalize(Local::now())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn normalize_should_remove_nanoseconds() {
        let timestamp = DateTime::from_str("2025-10-16T23:21:18.612202576+02:00").unwrap();

        let normalized = normalize(timestamp).unwrap();
        
        assert_eq!("2025-10-16T23:21:18+02:00", normalized.to_rfc3339());
    }

    #[test]
    fn can_convert_naive_time_to_time_span() {
        let naive_time = NaiveTime::from_hms_opt(2, 45, 0).unwrap();

        let actual: TimeSpan = naive_time.into();

        let expected = TimeSpan::new(2, 45, 0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_convert_time_delta_to_time_span() {
        let delta = TimeDelta::hours(1) + TimeDelta::minutes(59) + TimeDelta::seconds(59);

        let actual: TimeSpan = delta.into();

        let expected = TimeSpan::new(1, 59, 59);
        assert_eq!(expected, actual);
    }

      #[test]
    fn can_convert_negative_time_delta_to_time_span() {
        let delta = TimeDelta::hours(-1) + TimeDelta::minutes(-59) + TimeDelta::seconds(-59);

        let actual: TimeSpan = delta.into();

        let expected = TimeSpan::new(1, 59, 59);
        assert_eq!(expected, actual);
    }


    #[test]
    fn time_span_is_zero() {
        let time_span = TimeSpan::new(0, 0, 0);

        assert!(time_span.is_zero());
        assert_eq!(TimeSpan::ZERO, time_span);
    }

    #[test]
    fn time_span_seconds_component_5s() {
        let time_span = TimeSpan::new(0, 0, 5);

        assert_eq!(5, time_span.seconds());
    }

    #[test]
    fn time_delta_seconds_component_1m_12s() {
        let time_span = TimeSpan::new(0, 1, 12);
        assert_eq!(12, time_span.seconds());
    }

    #[test]
    fn time_delta_minutes_component_5s() {
        let time_span = TimeSpan::new(0, 0, 5);

        assert_eq!(0, time_span.minutes());
    }

    #[test]
    fn time_delta_minutes_component_2m_35s() {
        let time_span = TimeSpan::new(0, 2, 35);
        assert_eq!(2, time_span.minutes());
    }

    #[test]
    fn time_delta_hours_component_59m_59s() {
        let time_span = TimeSpan::new(0, 59, 59);
        assert_eq!(0, time_span.hours());
    }

    #[test]
    fn time_delta_hours_component_1h_59m_59s() {
        let time_span = TimeSpan::new(1, 59, 59);

        assert_eq!(1, time_span.hours());
    }
}
