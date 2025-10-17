use chrono::{TimeDelta, prelude::*};
use clap::Parser;
use error_stack::{Report, ResultExt};
use rendezvous_coach::error::{AppError, AppResult};
use rendezvous_coach::feature::coach::{self, Speaker, TTSSpeaker, lexicon};
use rendezvous_coach::init;
use rendezvous_coach::plan::Plan;
use rendezvous_coach::time::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Rendezvous time
    #[arg(short, long, value_name = "HH:MM")]
    rendezvous: String,
    /// Trip duration
    #[arg(short, long, value_name = "HH:MM")]
    trip: String,
}

fn main() -> AppResult<()> {
    init::error_reporting();
    init::tracing();

    let mut last_check: Option<DateTime<Local>> = None;

    let cli = Cli::parse();
    let plan = Plan {
        rendezvous_time: parse_today_time(&cli.rendezvous)?,
        trip_duration: parse_time_span(&cli.trip)?,
    };

    let mut speaker = TTSSpeaker::new().change_context(AppError)?;

    loop {
        let now = now().change_context(AppError)?;
        match last_check {
            Some(last) if last == now => (), // Check at most one time every second
            _ => {
                last_check = Some(now);
                match check_time(&plan, now, &mut speaker)? {
                    Some(delay) => {
                        let std_delay = delay
                            .to_std()
                            .change_context(AppError)
                            .attach("invalid time check delay")?;
                        println!(
                            "\tProssimo avviso ➡️ {}",
                            lexicon::remaining_time_message(&TimeSpan::from(std_delay))
                        );
                        std::thread::sleep(std_delay);
                    }
                    None => break,
                }
            }
        }
    }

    Ok(())
}

fn next_delay_every(remaining_time: TimeSpan, minutes: u64) -> AppResult<Option<TimeDelta>> {
    let next_delay_seconds = TimeDelta::seconds(remaining_time.seconds() as i64);
    let next_delay_minutes = TimeDelta::minutes(
        Some(remaining_time.minutes())
            .map(|m| (m % minutes) as i64)
            .unwrap_or(minutes as i64),
    );
    let next_delay = next_delay_minutes + next_delay_seconds;

    if next_delay.is_zero() {
        Ok(Some(TimeDelta::minutes(minutes as i64)))
    } else {
        Ok(Some(next_delay))
    }
}

fn check_time(
    plan: &Plan,
    now: DateTime<Local>,
    speaker: &mut impl Speaker,
) -> AppResult<Option<TimeDelta>> {
    let remaining_time: TimeSpan = (plan.departure_time() - now).into();

    if remaining_time.is_zero() {
        time_to_go(&plan, &now, speaker)?;
        Ok(None)
    } else if remaining_time == TimeSpan::new(0, 1, 0) {
        report_time(&plan, &now, &remaining_time, speaker)?;
        let next_delay = TimeDelta::from(remaining_time);
        Ok(Some(next_delay))
    } else if remaining_time <= TimeSpan::new(0, 5, 0) {
        report_time(&plan, &now, &remaining_time, speaker)?;
        let next_delay = if remaining_time.seconds() > 0 {
            // Get to the next whole minute
            TimeDelta::seconds(remaining_time.seconds() as i64)
        } else {
            TimeDelta::minutes(1)
        };
        Ok(Some(next_delay))
    } else if remaining_time == TimeSpan::new(0, 15, 0) {
        report_time(&plan, &now, &remaining_time, speaker)?;
        Ok(Some(TimeDelta::minutes(5)))
    } else if remaining_time <= TimeSpan::new(0, 15, 0) {
        report_time(&plan, &now, &remaining_time, speaker)?;
        next_delay_every(remaining_time, 5)
    } else if remaining_time == TimeSpan::new(1, 0, 0) {
        report_time(&plan, &now, &remaining_time, speaker)?;
        Ok(Some(TimeDelta::minutes(15)))
    } else if remaining_time <= TimeSpan::new(1, 0, 0) {
        report_time(&plan, &now, &remaining_time, speaker)?;
        next_delay_every(remaining_time, 15)
    } else {
        report_time(&plan, &now, &remaining_time, speaker)?;
        next_delay_every(remaining_time, 30)
    }
}

fn report_time(
    plan: &Plan,
    now: &DateTime<Local>,
    remaining_time: &TimeSpan,
    speaker: &mut impl Speaker,
) -> AppResult<()> {
    let message = coach::lexicon::remaining_time_message(remaining_time);
    print_console_message(&message, now, plan);
    speaker.speak(&message).change_context(AppError)
}

fn time_to_go(plan: &Plan, now: &DateTime<Local>, speaker: &mut impl Speaker) -> AppResult<()> {
    let message = "Ora di partire!";
    print_console_message(&message, now, plan);
    speaker.speak(message).change_context(AppError)
}

fn print_console_message(message: &str, now: &DateTime<Local>, plan: &Plan) {
    println!("Ora: {}", now.to_rfc3339());
    println!("Partenza: {}", plan.departure_time().to_rfc3339());
    println!("📨 {message}");
    println!("--------------------------------");
}

fn parse_time(input: &str) -> AppResult<NaiveTime> {
    NaiveTime::parse_from_str(input, "%H:%M")
        .change_context(AppError)
        .attach("invalid time")
}

fn parse_time_span(arg: &str) -> AppResult<TimeSpan> {
    parse_time(arg).map(|nt| nt.into())
}

fn parse_today_time(input: &str) -> AppResult<DateTime<Local>> {
    let time = parse_time(input)?;
    let now = now().change_context(AppError)?;
    now.with_time(time)
        .single()
        .ok_or(Report::new(AppError))
        .attach("invalid time for the current date")
}

#[cfg(test)]
mod tests {
    use rendezvous_coach::feature::coach::SpeakerResult;

    use super::*;

    struct DummySpeaker;
    impl Speaker for DummySpeaker {
        fn speak(&mut self, _content: &str) -> SpeakerResult<()> {
            Ok(())
        }
    }

    fn parse_naive_time(s: &str) -> NaiveTime {
        NaiveTime::parse_from_str(s, "%H:%M:%S").unwrap()
    }

    fn time_delta_from(nt: NaiveTime) -> TimeDelta {
        let time_span = TimeSpan::from(nt);
        TimeDelta::hours(time_span.hours() as i64)
            + TimeDelta::minutes(time_span.minutes() as i64)
            + TimeDelta::seconds(time_span.seconds() as i64)
    }

    fn assert_check_time(to_departure_time: &str, next_time_delta: Option<&str>) {
        let base_timestamp = now().unwrap();
        let now_naive_time = parse_naive_time("05:00:00");
        let now = base_timestamp.with_time(now_naive_time).unwrap();

        let remaining_time: TimeSpan = parse_naive_time(to_departure_time).into();
        let rendezvous_time = now + TimeDelta::from(remaining_time);

        let plan = Plan {
            rendezvous_time,
            trip_duration: TimeSpan::ZERO,
        };

        let actual_time_delta = check_time(&plan, now, &mut DummySpeaker).unwrap();

        let expected = next_time_delta.map(parse_naive_time).map(time_delta_from);
        assert_eq!(expected, actual_time_delta);
    }

    #[test]
    fn parse_time_should_parse_correctly_formatted_string() {
        let parsed = parse_time("14:45").unwrap();

        let expected = NaiveTime::from_hms_opt(14, 45, 00).unwrap();
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_time_should_not_parse_invalid_formatted_string() {
        let parsed = parse_time("1445");

        assert!(parsed.is_err());
    }

    #[test]
    fn parse_time_span_should_parse_correctly_formatted_string() {
        let parsed = parse_time_span("01:30").unwrap();

        let expected = TimeSpan::new(1, 30, 0);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_time_delta_should_note_parse_incorrectly_formatted_string() {
        let parsed = parse_time_span("0130");
        assert!(parsed.is_err())
    }

    #[test]
    fn parse_today_time_should_parse_correctly_formatted_string() {
        let parsed = parse_today_time("14:45").unwrap();

        let expected_time = NaiveTime::from_hms_opt(14, 45, 00).unwrap();
        let expected = Local::now().with_time(expected_time).unwrap();
        assert_eq!(expected, parsed);
    }

    #[test]
    fn check_time_12s() {
        assert_check_time("00:00:12", Some("00:00:12"));
    }

    #[test]
    fn check_time_0s() {
        assert_check_time("00:00:00", None);
    }

    #[test]
    fn check_time_60s() {
        assert_check_time("00:01:00", Some("00:01:00"));
    }

    #[test]
    fn check_time_1m_24s() {
        assert_check_time("00:01:24", Some("00:00:24"));
    }

    #[test]
    fn check_time_5m_2s() {
        assert_check_time("00:05:02", Some("00:00:02"));
    }

    #[test]
    fn check_time_8m_42s() {
        assert_check_time("00:08:42", Some("00:03:42"));
    }

    #[test]
    fn check_time_10m() {
        assert_check_time("00:10:00", Some("00:05:00"));
    }

    #[test]
    fn check_time_14m_55s() {
        assert_check_time("00:14:55", Some("00:04:55"));
    }

    #[test]
    fn check_time_22m_11s() {
        assert_check_time("00:22:11", Some("00:07:11"));
    }

    #[test]
    fn check_time_29m_59s() {
        assert_check_time("00:29:59", Some("00:14:59"));
    }

    #[test]
    fn check_time_30m_59s() {
        assert_check_time("00:30:59", Some("00:00:59"));
    }

    #[test]
    fn check_time_48m_00s() {
        assert_check_time("00:48:00", Some("00:03:00"));
    }

    #[test]
    fn check_time_1h() {
        assert_check_time("01:00:00", Some("00:15:00"));
    }

    #[test]
    fn check_time_1h_48m_34s() {
        assert_check_time("01:48:34", Some("00:18:34"));
    }
}
