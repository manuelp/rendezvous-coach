use std::time::Duration;

use clap::Parser;
use error_stack::ResultExt;
use owo_colors::OwoColorize;
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

    let cli = Cli::parse();
    let plan = Plan {
        rendezvous_time: Timestamp::parse_today_time(&cli.rendezvous).change_context(AppError)?,
        trip_duration: TimeSpan::parse(&cli.trip).change_context(AppError)?,
    };

    let coach = DefaultItCoach;
    let mut speaker = TTSSpeaker::new().change_context(AppError)?;

    // Immediately report remaining time
    let now = Timestamp::now().change_context(AppError)?;
    let remaining_time = plan.rendezvous_time.time_span_from(&now);
    let message = coach.remaining_time_message(&remaining_time);
    report_message(&message, &now, &plan, &mut speaker)?;

    // Plan and manage notifications
    let mut notifications = plan.notifications(&coach).change_context(AppError)?;
    while !notifications.is_empty() {
        std::thread::sleep(Duration::from_secs(1));

        let now = Timestamp::now().change_context(AppError)?;
        match notifications.pop_if(|n| n.time == now) {
            Some(n) => report_message(&n.message, &now, &plan, &mut speaker)?,
            None => (),
        }
    }

    Ok(())
}

fn report_message<S: Speaker>(
    message: &str,
    now: &Timestamp,
    plan: &Plan,
    speaker: &mut S,
) -> AppResult<()> {
    println!("------------------------------------------------");
    println!("Ora: {}", now);
    println!("Partenza: {}", plan.departure_time());
    println!("📨 {}", message.bright_red());
    speaker.speak(&message).change_context(AppError)?;
    Ok(())
}
