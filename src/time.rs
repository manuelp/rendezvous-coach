use std::ops::{Add, Sub};

use chrono::{TimeDelta, prelude::*};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSpan(u64);

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

#[cfg(test)]
mod tests {
    use super::*;

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
