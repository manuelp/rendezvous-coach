use error_stack::{Report, ResultExt};
use tracing::info;
use tts::Tts;

pub mod lexicon;

#[derive(Debug, thiserror::Error)]
#[error("TTS error")]
pub struct SpeakerError;

type SpeakerResult<T> = Result<T, Report<SpeakerError>>;

pub trait Speaker {
    fn speak(&mut self, content: &str) -> SpeakerResult<()>;
}

pub struct TTSSpeaker {
    tts: Tts,
}

impl TTSSpeaker {
    pub fn new() -> SpeakerResult<TTSSpeaker> {
        let tts = Tts::default()
            .change_context(SpeakerError)
            .attach("cannot initialize TTS engine")?;

        let features = tts.supported_features();
        info!("TTS features: {:?}", features);

        Ok(TTSSpeaker { tts })
    }
}

impl Speaker for TTSSpeaker {
    fn speak(&mut self, content: &str) -> SpeakerResult<()> {
        self.tts
            .speak(content, true)
            .change_context(SpeakerError)
            .attach("cannot speak")?;
        Ok(())
    }
}
