use clap::Parser;
use error_stack::ResultExt;
use owo_colors::OwoColorize;
use owo_colors::colors::css::Gray;
use rendezvous_coach::error::{AppError, AppResult};
use rendezvous_coach::feature::coach::{Coach, DefaultItCoach};
use rendezvous_coach::feature::tts::{Speaker, TTSSpeaker};
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

    let mut last_check: Option<Timestamp> = None;

    let cli = Cli::parse();
    let plan = Plan {
        rendezvous_time: Timestamp::parse_today_time(&cli.rendezvous).change_context(AppError)?,
        trip_duration: TimeSpan::parse(&cli.trip).change_context(AppError)?,
    };

    let coach = DefaultItCoach;
    let mut speaker = TTSSpeaker::new().change_context(AppError)?;

    loop {
        let now = Timestamp::now().change_context(AppError)?;
        match last_check {
            Some(last) if last == now => (), // Check at most one time every second
            _ => {
                println!("--------------------------------");
                last_check = Some(now);
                match check_time(&plan, now, &coach, &mut speaker)? {
                    Some(delay) => {
                        let msg = coach.remaining_time_message(&delay);
                        println!("\tProssimo avviso ➡️ {}", msg.fg::<Gray>().underline());
                        std::thread::sleep(delay.into());
                    }
                    None => break,
                }
            }
        }
    }

    Ok(())
}

fn next_delay_every(remaining_time: TimeSpan, minutes: u64) -> AppResult<Option<TimeSpan>> {
    let next_delay_seconds = TimeSpan::of_seconds(remaining_time.seconds());
    let next_delay_minutes = TimeSpan::of_minutes(
        Some(remaining_time.minutes())
            .map(|m| m % minutes)
            .unwrap_or(minutes),
    );
    let next_delay = next_delay_minutes + next_delay_seconds;

    if next_delay.is_zero() {
        Ok(Some(TimeSpan::of_minutes(minutes)))
    } else {
        Ok(Some(next_delay))
    }
}

fn check_time<C: Coach, S: Speaker>(
    plan: &Plan,
    now: Timestamp,
    coach: &C,
    speaker: &mut S,
) -> AppResult<Option<TimeSpan>> {
    let remaining_time: TimeSpan = plan.departure_time().time_span_from(&now);

    if remaining_time.is_zero() {
        time_to_go(&plan, &now, coach, speaker)?;
        Ok(None)
    } else if remaining_time == TimeSpan::new(0, 1, 0) {
        report_time(&plan, &now, &remaining_time, coach, speaker)?;
        Ok(Some(remaining_time))
    } else if remaining_time <= TimeSpan::new(0, 5, 0) {
        report_time(&plan, &now, &remaining_time, coach, speaker)?;
        let next_delay = if remaining_time.seconds() > 0 {
            // Get to the next whole minute
            TimeSpan::of_seconds(remaining_time.seconds())
        } else {
            TimeSpan::of_minutes(1)
        };
        Ok(Some(next_delay))
    } else if remaining_time == TimeSpan::new(0, 15, 0) {
        report_time(&plan, &now, &remaining_time, coach, speaker)?;
        Ok(Some(TimeSpan::of_minutes(5)))
    } else if remaining_time <= TimeSpan::new(0, 15, 0) {
        report_time(&plan, &now, &remaining_time, coach, speaker)?;
        next_delay_every(remaining_time, 5)
    } else if remaining_time == TimeSpan::new(1, 0, 0) {
        report_time(&plan, &now, &remaining_time, coach, speaker)?;
        Ok(Some(TimeSpan::of_minutes(15)))
    } else if remaining_time <= TimeSpan::new(1, 0, 0) {
        report_time(&plan, &now, &remaining_time, coach, speaker)?;
        next_delay_every(remaining_time, 15)
    } else {
        report_time(&plan, &now, &remaining_time, coach, speaker)?;
        next_delay_every(remaining_time, 30)
    }
}

fn report_time<C: Coach, S: Speaker>(
    plan: &Plan,
    now: &Timestamp,
    remaining_time: &TimeSpan,
    coach: &C,
    speaker: &mut S,
) -> AppResult<()> {
    let message = coach.remaining_time_message(remaining_time);
    print_console_message(&message, now, plan);
    speaker.speak(&message).change_context(AppError)
}

fn time_to_go<C: Coach, S: Speaker>(
    plan: &Plan,
    now: &Timestamp,
    coach: &C,
    speaker: &mut S,
) -> AppResult<()> {
    let message = coach.time_to_go();
    print_console_message(&message, now, plan);
    speaker.speak(&message).change_context(AppError)
}

fn print_console_message(message: &str, now: &Timestamp, plan: &Plan) {
    println!("Ora: {}", now);
    println!("Partenza: {}", plan.departure_time());
    println!("📨 {}", message.bright_red());
}

#[cfg(test)]
mod tests {
    use rendezvous_coach::feature::tts::SpeakerResult;

    use super::*;

    struct DummySpeaker;
    impl Speaker for DummySpeaker {
        fn speak(&mut self, _content: &str) -> SpeakerResult<()> {
            Ok(())
        }
    }

    fn assert_check_time(to_departure_time: &str, next_time_span: Option<&str>) {
        let now = Timestamp::parse_today_time("05:00:00").unwrap();
        let remaining_time: TimeSpan = TimeSpan::parse(to_departure_time).unwrap();
        let rendezvous_time = now + remaining_time;

        let plan = Plan {
            rendezvous_time,
            trip_duration: TimeSpan::ZERO,
        };

        let actual_time_delta = check_time(&plan, now, &DefaultItCoach, &mut DummySpeaker).unwrap();

        let expected = next_time_span.map(|s| TimeSpan::parse(s).unwrap());
        assert_eq!(expected, actual_time_delta);
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
