use error_stack::ResultExt;
use rendezvous_coach::error::{AppError, AppResult};
use rendezvous_coach::feature::tts::{Speaker, TTSSpeaker};
use rendezvous_coach::init;
use rendezvous_coach::plan::Plan;

fn main() -> AppResult<()> {
    init::error_reporting();
    init::tracing();

    let mut speaker = TTSSpeaker::new().change_context(AppError)?;
    speaker.speak("Oggi è una bella giornata!").change_context(AppError)?;
    speaker.speak("Mancano 15 minuti alla partenza").change_context(AppError)?;

    Ok(())
}
