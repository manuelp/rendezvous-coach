use chrono::{TimeDelta, prelude::*};

pub fn time_to_duration(time: NaiveTime) -> TimeDelta {
    time.signed_duration_since(NaiveTime::MIN)
}

pub fn time_delta_seconds(delta: &TimeDelta) -> i64 {
    delta.num_seconds() % 60
}

pub fn time_delta_minutes(delta: &TimeDelta) -> i64 {
    (delta.num_seconds() % 3600) / 60
}

pub fn time_delta_hours(delta: &TimeDelta) -> i64 {
    delta.num_seconds() / 3600
}

pub fn time_delta_clamp_seconds(delta: TimeDelta) -> TimeDelta {
    TimeDelta::seconds(delta.num_seconds())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_convert_naive_time_to_time_delta() {
        let naive_time = NaiveTime::from_hms_opt(2, 45, 00).unwrap();

        let delta: TimeDelta = time_to_duration(naive_time);

        let expected_delta = TimeDelta::hours(2) + TimeDelta::minutes(45);
        assert_eq!(expected_delta, delta);
    }

    #[test]
    fn time_delta_seconds_component_5s() {
        let input = TimeDelta::seconds(5);

        let seconds = time_delta_seconds(&input);

        assert_eq!(5, seconds);
    }

    #[test]
    fn time_delta_seconds_component_1m_12s() {
        let input = TimeDelta::minutes(1) + TimeDelta::seconds(12);

        let seconds = time_delta_seconds(&input);

        assert_eq!(12, seconds);
    }

    #[test]
    fn time_delta_minutes_component_5s() {
        let input = TimeDelta::seconds(5);

        let minutes = time_delta_minutes(&input);

        assert_eq!(0, minutes);
    }

    #[test]
    fn time_delta_minutes_component_2m_35s() {
        let input = TimeDelta::minutes(2) + TimeDelta::seconds(35);

        let minutes = time_delta_minutes(&input);

        assert_eq!(2, minutes);
    }

    #[test]
    fn time_delta_hours_component_59m_59s() {
        let input = TimeDelta::minutes(59) + TimeDelta::seconds(59);

        let hours = time_delta_hours(&input);

        assert_eq!(0, hours);
    }

    #[test]
    fn time_delta_hours_component_1h_59m_59s() {
        let input = TimeDelta::hours(1) + TimeDelta::minutes(59) + TimeDelta::seconds(59);

        let hours = time_delta_hours(&input);

        assert_eq!(1, hours);
    }

    #[test]
    fn time_delta_clamp_seconds_should_remove_sub_second_fields() {
        let delta = TimeDelta::milliseconds(12) + TimeDelta::nanoseconds(43);

        let clamped = time_delta_clamp_seconds(delta);

        assert!(clamped.is_zero());
    }
}
