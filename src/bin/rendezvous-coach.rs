use chrono::{Duration, prelude::*};
use error_stack::ResultExt;
use rendezvous_coach::error::{AppError, AppResult};
use rendezvous_coach::feature::tts::{Speaker, TTSSpeaker};
use rendezvous_coach::init;

#[derive(Debug)]
struct Plan {
    rendezvous_time: DateTime<Local>,
    trip_duration: Duration,
    alerting_duration: Duration,
}

fn main() -> AppResult<()> {
    init::error_reporting();
    init::tracing();

    let mut speaker = TTSSpeaker::new().change_context(AppError)?;
    speaker.speak("Oggi è una bella giornata!").change_context(AppError)?;
    speaker.speak("Mancano 15 minuti alla partenza").change_context(AppError)?;

    //std::thread::sleep(Duration::seconds(1).to_std().unwrap());

    let plan = Plan {
        rendezvous_time: Local::now(),
        trip_duration: Duration::minutes(15),
        alerting_duration: Duration::minutes(1 * 60 + 30),
    };
    dbg!(&plan);

    Ok(())
}
