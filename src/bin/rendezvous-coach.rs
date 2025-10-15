use error_stack::ResultExt;
use rendezvous_coach::error::{AppError, AppResult};
use rendezvous_coach::init;
use tts::Tts;

fn main() -> AppResult<()> {
    init::error_reporting();
    init::tracing();

    let mut tts = Tts::default()
        .change_context(AppError)
        .attach("cannot initialize TTS engine")?;
    let features = tts.supported_features();
    println!("Features: {:?}", features);

    tts.speak("Oggi è una bella giornata!", true)
        .change_context(AppError)
        .attach("cannot speak")?;

    Ok(())
}
