use chrono::{TimeDelta, prelude::*};
use clap::Parser;
use error_stack::{Report, ResultExt};
use rendezvous_coach::error::{AppError, AppResult};
use rendezvous_coach::feature::coach::{self, Speaker, TTSSpeaker};
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

    let cli = Cli::parse();
    let plan = Plan {
        rendezvous_time: parse_today_time(&cli.rendezvous)?,
        trip_duration: parse_time_span(&cli.trip)?,
    };

    let mut speaker = TTSSpeaker::new().change_context(AppError)?;

    loop {
        match check_time(&plan, &mut speaker)? {
            Some(delay) => {
                let std_delay = delay
                    .to_std()
                    .change_context(AppError)
                    .attach("invalid time check delay")?;
                println!("\tWaiting: {std_delay:?}");
                std::thread::sleep(std_delay);
            }
            None => break,
        }
    }

    Ok(())
}

fn check_time(plan: &Plan, speaker: &mut impl Speaker) -> AppResult<Option<TimeDelta>> {
    let now = now().change_context(AppError)?;
    let remaining_time: TimeSpan = (plan.departure_time() - now).into();

    // First, wait off seconds to get to whole minute time
    let off_seconds = remaining_time.seconds() as i64;
    if off_seconds > 0 {
        report_time(&plan, &now, &remaining_time, speaker)?;
        return Ok(Some(TimeDelta::seconds(off_seconds)))
    }

    // Then go for regular checks
    if remaining_time.is_zero() {
        time_to_go(&plan, &now, speaker)?;
        Ok(None)
    } else if remaining_time <= TimeSpan::new(0, 1, 0) {
        report_time(&plan, &now, &remaining_time, speaker)?;
        let next_delay = TimeDelta::from(remaining_time);
        Ok(Some(next_delay))
    } else if remaining_time <= TimeSpan::new(0, 5, 0) {
        report_time(&plan, &now, &remaining_time, speaker)?;
        Ok(Some(TimeDelta::minutes(1)))
    } else if remaining_time <= TimeSpan::new(0, 15, 0) {
        report_time(&plan, &now, &remaining_time, speaker)?;
        Ok(Some(TimeDelta::minutes(5)))
    } else if remaining_time <= TimeSpan::new(1, 0, 0) {
        report_time(&plan, &now, &remaining_time, speaker)?;
        Ok(Some(TimeDelta::minutes(15)))
    } else {
        Ok(Some(TimeDelta::minutes(30)))
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
    println!("-> {message}");
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
}
