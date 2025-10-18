use std::fmt::{Debug, Display};
use std::ops::{Add, Sub};

use chrono::offset::LocalResult;
use chrono::prelude::*;
use chrono::{TimeDelta, TimeZone};
use error_stack::{Report, ResultExt};

#[derive(Debug, thiserror::Error)]
#[error("time error")]
pub struct TimeError;

pub type TimeResult<T> = Result<T, Report<TimeError>>;

// ---------------------- Time span

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSpan(u64);

impl Debug for TimeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nt = NaiveTime::from_num_seconds_from_midnight_opt(self.0 as u32, 0).unwrap();
        write!(f, "{}", nt.format("%H:%M:%S"))
    }
}

impl Display for TimeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<TimeSpan> for std::time::Duration {
    fn from(value: TimeSpan) -> Self {
        std::time::Duration::from_secs(value.0)
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

impl TimeSpan {
    pub const ZERO: Self = Self(0);

    pub fn new(hours: u64, minutes: u64, seconds: u64) -> Self {
        Self(seconds + (minutes * 60) + (hours * 60 * 60))
    }

    pub fn of_seconds(sec: u64) -> Self {
        Self::new(0, 0, sec)
    }

    pub fn of_minutes(min: u64) -> Self {
        Self::new(0, min, 0)
    }

    pub fn of_hours(hour: u64) -> Self {
        Self::new(hour, 0, 0)
    }

    pub fn parse(input: &str) -> TimeResult<TimeSpan> {
        let time = NaiveTime::parse_from_str(input, "%H:%M:%S")
            .or(NaiveTime::parse_from_str(input, "%H:%M"))
            .change_context(TimeError)
            .attach("invalid time")?;
        let delta = time.signed_duration_since(NaiveTime::MIN);
        let span = TimeSpan(delta.abs().num_seconds() as u64);
        Ok(span)
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

// ---------------------- Time
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Time(NaiveTime);

impl Debug for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Time {
    pub fn new(hour: u32, min: u32, sec: u32) -> TimeResult<Self> {
        let naive = NaiveTime::from_hms_opt(hour, min, sec)
            .ok_or(Report::new(TimeError))
            .attach("invalid time")?;
        Ok(Self(naive))
    }

    pub fn parse(input: &str) -> TimeResult<Self> {
        let naive = NaiveTime::parse_from_str(input, "%H:%M:%S")
            .or(NaiveTime::parse_from_str(input, "%H:%M"))
            .change_context(TimeError)
            .attach("invalid time")?;
        Ok(Self(naive))
    }
}

// ---------------------- Timestamp

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Timestamp(DateTime<Local>);

impl Debug for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Sub<&TimeSpan> for &Timestamp {
    type Output = Timestamp;

    fn sub(self, rhs: &TimeSpan) -> Self::Output {
        Timestamp(self.0 - TimeDelta::seconds(rhs.0 as i64))
    }
}

impl Sub<TimeSpan> for Timestamp {
    type Output = Timestamp;

    fn sub(self, rhs: TimeSpan) -> Self::Output {
        &self - &rhs
    }
}

impl Add<TimeSpan> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: TimeSpan) -> Self::Output {
        Timestamp(self.0 + TimeDelta::seconds(rhs.0 as i64))
    }
}

impl Timestamp {
    pub fn now() -> TimeResult<Timestamp> {
        let ts = Local::now()
            .with_nanosecond(0)
            .ok_or(Report::new(TimeError))
            .attach("cannot determine the date and time")?;
        Ok(Timestamp(ts))
    }

    pub fn parse_today_time(input: &str) -> TimeResult<Timestamp> {
        let parsed_time = Time::parse(input)?;
        let now = Timestamp::now()?;
        now.with_time(&parsed_time)
    }

    pub fn new(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> TimeResult<Self> {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or(TimeError)
            .attach("invalid date")?;
        let time = NaiveTime::from_hms_opt(hour, min, sec)
            .ok_or(TimeError)
            .attach("invalid time")?;
        let date_time = NaiveDateTime::new(date, time);
        match Local.from_local_datetime(&date_time) {
            LocalResult::Single(ldt) => Ok(Self(ldt)),
            LocalResult::Ambiguous(_, _) => Err(TimeError).attach("ambiguous time for the date"),
            LocalResult::None => Err(TimeError).attach("invalid date/time"),
        }
    }

    pub fn with_time(&self, time: &Time) -> TimeResult<Timestamp> {
        match self.0.with_time(time.0) {
            LocalResult::Single(new_ts) => Ok(Self(new_ts)),
            LocalResult::Ambiguous(_, _) => Err(TimeError).attach("ambiguous time for today"),
            LocalResult::None => Err(TimeError).attach("invalid time for today"),
        }
    }

    pub fn time_span_from(&self, other: &Timestamp) -> TimeSpan {
        let delta_seconds = (self.0 - other.0).num_seconds();
        if delta_seconds >= 0 {
            TimeSpan::of_seconds(delta_seconds as u64)
        } else {
            TimeSpan::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- TimeSpan

    #[test]
    fn time_span_debug_formatting() {
        let ts = TimeSpan::new(1, 15, 22);

        assert_eq!("01:15:22", format!("{:?}", ts));
    }

    #[test]
    fn time_span_display_formatting() {
        let ts = TimeSpan::new(1, 15, 22);

        assert_eq!("01:15:22", format!("{}", ts));
    }

    #[test]
    fn time_span_should_be_buildable_with_seconds() {
        let built = TimeSpan::of_seconds(5);

        assert_eq!("00:00:05", format!("{built}"));
    }

    #[test]
    fn time_span_should_be_buildable_with_minutes() {
        let built = TimeSpan::of_minutes(5);

        assert_eq!("00:05:00", format!("{built}"));
    }

    #[test]
    fn time_span_should_be_buildable_with_hours() {
        let built = TimeSpan::of_hours(5);

        assert_eq!("05:00:00", format!("{built}"));
    }

    #[test]
    fn time_span_parse_should_parse_correctly_formatted_string() {
        let parsed = TimeSpan::parse("12:15:33").unwrap();

        assert_eq!(TimeSpan::new(12, 15, 33), parsed);
    }

    #[test]
    fn time_span_parse_should_parse_correctly_formatted_string_without_seconds() {
        let parsed = TimeSpan::parse("12:15").unwrap();

        assert_eq!(TimeSpan::new(12, 15, 00), parsed);
    }

    #[test]
    fn time_span_parse_should_not_parse_incorrectly_formatted_string() {
        let result = TimeSpan::parse("12:15-");

        assert!(result.is_err());
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
    fn time_span_seconds_component_1m_12s() {
        let time_span = TimeSpan::new(0, 1, 12);
        assert_eq!(12, time_span.seconds());
    }

    #[test]
    fn time_span_minutes_component_5s() {
        let time_span = TimeSpan::new(0, 0, 5);

        assert_eq!(0, time_span.minutes());
    }

    #[test]
    fn time_span_minutes_component_2m_35s() {
        let time_span = TimeSpan::new(0, 2, 35);
        assert_eq!(2, time_span.minutes());
    }

    #[test]
    fn time_span_hours_component_59m_59s() {
        let time_span = TimeSpan::new(0, 59, 59);
        assert_eq!(0, time_span.hours());
    }

    #[test]
    fn time_span_hours_component_1h_59m_59s() {
        let time_span = TimeSpan::new(1, 59, 59);

        assert_eq!(1, time_span.hours());
    }

    #[test]
    fn time_span_to_standard_duration_15s() {
        let time_span = TimeSpan::of_seconds(15);

        let converted = std::time::Duration::from(time_span);

        assert_eq!(std::time::Duration::from_secs(15), converted);
    }

    #[test]
    fn time_span_to_standard_duration_5m() {
        let time_span = TimeSpan::of_minutes(5);

        let converted = std::time::Duration::from(time_span);

        assert_eq!(std::time::Duration::from_secs(5 * 60), converted);
    }

    // ---- Timestamp

    #[test]
    fn timestamp_debug_should_be_readable() {
        let now = Timestamp::now().unwrap();

        assert_eq!(format!("{:?}", now.0), format!("{:?}", now));
    }

    #[test]
    fn timestamp_display_should_be_readable() {
        let now = Timestamp::now().unwrap();

        assert_eq!(format!("{}", now.0), format!("{}", now));
    }

    #[test]
    fn timestamp_should_be_buildable_manually() {
        let ts = Timestamp::new(2025, 10, 18, 16, 0, 0).unwrap();

        assert!(format!("{ts}").starts_with("2025-10-18 16:00:00"));
    }

    #[test]
    fn time_should_have_a_readable_debug_impl() {
        let time = Time::new(11, 02, 15).unwrap();

        assert_eq!("11:02:15", format!("{:?}", time));
    }

    #[test]
    fn time_should_have_a_readable_display_impl() {
        let time = Time::new(11, 02, 15).unwrap();

        assert_eq!("11:02:15", format!("{}", time));
    }

    #[test]
    fn time_should_be_parsed_from_correct_whole_string() {
        let parsed = Time::parse("10:57:44").unwrap();

        assert_eq!(Time::new(10, 57, 44).unwrap(), parsed);
    }

    #[test]
    fn time_should_be_parsed_from_correct_string_without_seconds() {
        let parsed = Time::parse("11:06").unwrap();

        assert_eq!(Time::new(11, 06, 00).unwrap(), parsed);
    }

    #[test]
    fn time_should_note_be_parsed_from_incorrect_string() {
        let result = Time::parse("11-06-foo");

        assert!(result.is_err());
    }

    #[test]
    fn timestamp_parse_today_time_with_a_valid_string() {
        let res = Timestamp::parse_today_time("16:58:22").unwrap();
        assert_eq!(Local::now().date_naive(), res.0.date_naive());
        assert_eq!(NaiveTime::from_hms_opt(16, 58, 22).unwrap(), res.0.time());
    }

    #[test]
    fn timestamp_parse_today_time_with_an_invalid_string() {
        let res = Timestamp::parse_today_time("16:58:-");

        assert!(res.is_err());
    }

    #[test]
    fn timestamp_set_time_should_set_it() {
        let original = Timestamp::new(2025, 10, 18, 16, 0, 0).unwrap();
        let time = Time::new(1, 2, 3).unwrap();

        let updated = original.with_time(&time).unwrap();

        let expected = Timestamp::new(2025, 10, 18, 1, 2, 3).unwrap();
        assert_eq!(expected, updated);
    }

    #[test]
    fn timestamp_time_span_from_same_timestamp() {
        let original = Timestamp::new(2025, 10, 18, 16, 0, 0).unwrap();

        let result = original.time_span_from(&original);

        assert_eq!(TimeSpan::ZERO, result);
    }

    #[test]
    fn timestamp_time_span_from_previous_timestamp() {
        let original = Timestamp::new(2025, 10, 18, 16, 0, 0).unwrap();
        let previous = Timestamp::new(2025, 10, 18, 15, 30, 11).unwrap();

        let result = original.time_span_from(&previous);

        assert_eq!(TimeSpan::new(0, 29, 49), result);
    }

    #[test]
    fn timestamp_time_span_from_successive_timestamp() {
        let original = Timestamp::new(2025, 10, 18, 15, 30, 11).unwrap();
        let successive = Timestamp::new(2025, 10, 18, 16, 0, 0).unwrap();

        let result = original.time_span_from(&successive);

        assert_eq!(TimeSpan::ZERO, result);
    }

    #[test]
    fn timestamp_subtract_time_span_0() {
        let original = Timestamp::new(2025, 10, 18, 15, 30, 11).unwrap();
        let time_span = TimeSpan::ZERO;

        let res: Timestamp = &original - &time_span;

        assert_eq!(original, res);
    }

    #[test]
    fn timestamp_subtract_time_span_gt_0() {
        let original = Timestamp::new(2025, 10, 18, 15, 30, 11).unwrap();
        let time_span = TimeSpan::new(1, 20, 1);

        let res: Timestamp = &original - &time_span;

        let expected_time = Time::new(14, 10, 10).unwrap();
        let expected = original.with_time(&expected_time).unwrap();
        assert_eq!(expected, res);
    }

    #[test]
    fn timestamp_add_time_span_0() {
        let original = Timestamp::new(2025, 10, 18, 15, 30, 11).unwrap();
        let time_span = TimeSpan::ZERO;

        let res: Timestamp = original + time_span;

        assert_eq!(original, res);
    }

    #[test]
    fn timestamp_add_time_span_gt_0() {
        let original = Timestamp::new(2025, 10, 18, 15, 30, 11).unwrap();
        let time_span = TimeSpan::new(1, 20, 19);

        let res: Timestamp = original + time_span;

        let expected_time = Time::new(16, 50, 30).unwrap();
        let expected = original.with_time(&expected_time).unwrap();
        assert_eq!(expected, res);
    }
}
