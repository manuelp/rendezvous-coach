use std::num::NonZero;
use std::path::{Path, PathBuf};

use error_stack::{Report, ResultExt};
use indicatif::{ProgressBar, ProgressStyle};
use rodio::{DeviceSinkBuilder, Player, buffer::SamplesBuffer};
use sherpa_rs::tts::{VitsTts, VitsTtsConfig};
use tracing::info;

#[derive(Debug, thiserror::Error)]
#[error("TTS error")]
pub struct SpeakerError;

pub type SpeakerResult<T> = Result<T, Report<SpeakerError>>;

pub trait Speaker {
    fn speak(&mut self, content: &str) -> SpeakerResult<()>;
}

const MODEL_DIR_NAME: &str = "vits-piper-it_IT-paola-medium";
const MODEL_ONNX: &str = "it_IT-paola-medium.onnx";
const MODEL_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/vits-piper-it_IT-paola-medium.tar.bz2";
const MODEL_DOWNLOAD_BYTES: u64 = 67_221_173;

pub struct TTSSpeaker {
    tts: VitsTts,
}

impl TTSSpeaker {
    pub fn new(model_path: Option<&Path>) -> SpeakerResult<Self> {
        let model_dir = match model_path {
            Some(p) => p.to_path_buf(),
            None => {
                let dir = default_model_dir();
                ensure_model(&dir)?;
                dir
            }
        };

        validate_model_dir(&model_dir)?;
        info!("Loading TTS model from {}", model_dir.display());

        let config = VitsTtsConfig {
            model: path_str(model_dir.join(MODEL_ONNX)),
            tokens: path_str(model_dir.join("tokens.txt")),
            data_dir: path_str(model_dir.join("espeak-ng-data")),
            lexicon: String::new(),
            length_scale: 1.0,
            noise_scale: 0.667,
            noise_scale_w: 0.8,
            ..Default::default()
        };

        let tts = VitsTts::new(config);
        Ok(Self { tts })
    }
}

impl Speaker for TTSSpeaker {
    fn speak(&mut self, content: &str) -> SpeakerResult<()> {
        let audio = self
            .tts
            .create(content, 0, 1.0)
            .map_err(|e| Report::new(SpeakerError).attach(e.to_string()))?;

        let samples = audio.samples;
        let sample_rate = audio.sample_rate;

        std::thread::spawn(move || {
            let Ok(mut handle) = DeviceSinkBuilder::open_default_sink() else {
                return;
            };
            handle.log_on_drop(false);
            let player = Player::connect_new(handle.mixer());
            let source = SamplesBuffer::new(
                NonZero::new(1u16).unwrap(),
                NonZero::new(sample_rate).unwrap(),
                samples,
            );
            player.append(source);
            player.sleep_until_end();
        });

        Ok(())
    }
}

fn path_str(p: PathBuf) -> String {
    p.to_string_lossy().into_owned()
}

fn default_model_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| {
            std::env::var("HOME")
                .map(PathBuf::from)
                .unwrap_or_default()
                .join(".local/share")
        })
        .join("rendezvous-coach")
        .join("models")
        .join(MODEL_DIR_NAME)
}

fn validate_model_dir(dir: &Path) -> SpeakerResult<()> {
    for file in [MODEL_ONNX, "tokens.txt"] {
        let p = dir.join(file);
        if !p.exists() {
            return Err(Report::new(SpeakerError)
                .attach(format!("missing model file: {}", p.display())));
        }
    }
    let espeak = dir.join("espeak-ng-data");
    if !espeak.exists() {
        return Err(Report::new(SpeakerError)
            .attach(format!("missing espeak-ng-data dir: {}", espeak.display())));
    }
    Ok(())
}

fn ensure_model(model_dir: &Path) -> SpeakerResult<()> {
    if model_dir.join(MODEL_ONNX).exists() {
        return Ok(());
    }
    let parent = model_dir
        .parent()
        .ok_or_else(|| Report::new(SpeakerError).attach("invalid model dir path"))?;
    std::fs::create_dir_all(parent)
        .change_context(SpeakerError)
        .attach("cannot create model cache dir")?;
    download_model(parent)
}

fn download_model(dest_parent: &Path) -> SpeakerResult<()> {
    eprintln!("First run: downloading TTS model (~67 MB) ...");

    let pb = ProgressBar::new(MODEL_DOWNLOAD_BYTES);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("=>-"),
    );

    let response = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .get(MODEL_URL)
        .call()
        .map_err(|e| Report::new(SpeakerError).attach(e.to_string()))?;

    if let Some(len) = response
        .header("content-length")
        .and_then(|v| v.parse::<u64>().ok())
    {
        pb.set_length(len);
    }

    let reader = pb.wrap_read(response.into_reader());
    let bz = bzip2::read::BzDecoder::new(reader);
    let mut archive = tar::Archive::new(bz);

    archive
        .unpack(dest_parent)
        .change_context(SpeakerError)
        .attach("cannot extract TTS model")?;

    pb.finish_with_message("done");
    Ok(())
}
