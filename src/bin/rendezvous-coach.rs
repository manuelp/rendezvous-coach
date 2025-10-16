use chrono::{TimeDelta, prelude::*};
use clap::Parser;
use error_stack::{Report, ResultExt};
use rendezvous_coach::error::{AppError, AppResult};
use rendezvous_coach::feature::tts::{Speaker, TTSSpeaker};
use rendezvous_coach::init;
use rendezvous_coach::plan::Plan;

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

    let cli = Cli::parse();
    let plan = Plan {
        rendezvous_time: parse_today_time(&cli.rendezvous)?,
        trip_duration: parse_time_delta(&cli.trip)?,
    };
    dbg!(&plan);

    Ok(())
}

fn parse_time(input: &str) -> AppResult<NaiveTime> {
    NaiveTime::parse_from_str(input, "%H:%M")
        .change_context(AppError)
        .attach("invalid time")
}

fn time_to_duration(time: NaiveTime) -> TimeDelta {
    time.signed_duration_since(NaiveTime::MIN)
}

fn parse_time_delta(arg: &str) -> AppResult<TimeDelta> {
    parse_time(arg).map(time_to_duration)
}

fn parse_today_time(input: &str) -> AppResult<DateTime<Local>> {
    let time = parse_time(input)?;
    Local::now()
        .with_time(time)
        .single()
        .ok_or(Report::new(AppError))
        .attach("invalid time for the current date")
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn can_convert_naive_time_to_time_delta() {
        let naive_time = NaiveTime::from_hms_opt(2, 45, 00).unwrap();

        let delta: TimeDelta = time_to_duration(naive_time);

        let expected_delta = TimeDelta::hours(2) + TimeDelta::minutes(45);
        assert_eq!(expected_delta, delta);
    }

    #[test]
    fn parse_time_delta_should_parse_correctly_formatted_string() {
        let parsed = parse_time_delta("01:30").unwrap();

        let expected = TimeDelta::hours(1) + TimeDelta::minutes(30);
        assert_eq!(expected, parsed);
        
    }

       #[test]
    fn parse_time_delta_should_note_parse_incorrectly_formatted_string() {
        let parsed = parse_time_delta("0130");
        assert!(parsed.is_err())
    }

    #[test]
    fn parse_today_time_should_parse_correctly_formatted_string() {
        let parsed = parse_today_time("14:45").unwrap();

        let expected_time = NaiveTime::from_hms_opt(14, 45, 00).unwrap();
        let expected = Local::now().with_time(expected_time).unwrap();
        assert_eq!(expected, parsed);
    }
}
