use crate::audio_input::AudioInput;
use anyhow::Result;
use cpal::traits::StreamTrait;
use futures::{lock::Mutex, StreamExt};
use std::sync::Arc;
use vosk::{DecodingState, Model, Recognizer};

pub struct SpeechRecognizer {
    audio_input: AudioInput,
    recognizer: Arc<Mutex<Recognizer>>,
}
impl SpeechRecognizer {
    pub fn new() -> Self {
        let audio_input = AudioInput::default();

        let recognizer = {
            let model =
                Model::new("./vosk-model-small-en-us-0.15").expect("Could not create vosk model");

            let recognizer = Recognizer::new(&model, audio_input.config.sample_rate().0 as f32)
                .expect("Could not create the Recognizer");
            Arc::new(Mutex::new(recognizer))
        };

        Self {
            audio_input,
            recognizer,
        }
    }

    pub async fn start_recognize(&self) -> Result<()> {
        let (stream, mut rx) = self.audio_input.new_mono_stream(1);

        if let Err(err) = stream.play() {
            tracing::error!("Cannot start input stream: {}", err);
        }

        let recognizer = self.recognizer.clone();
        // tokio::spawn(async move {
        while let Some(data) = rx.next().await {
            let mut recognizer = recognizer.lock().await;
            match recognizer.accept_waveform(&data) {
                Ok(state) => match state {
                    DecodingState::Running => {}
                    DecodingState::Finalized => {
                        tracing::debug!("result: {:#?}", recognizer.result().single());
                    }
                    DecodingState::Failed => eprintln!("error"),
                },
                Err(err) => tracing::error!("Cannot accept_waveform {}", err),
            }
        }
        // });

        Ok(())
    }
}
