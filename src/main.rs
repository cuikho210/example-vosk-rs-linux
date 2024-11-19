use speech_recognizer::SpeechRecognizer;

mod audio_input;
mod speech_recognizer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();
    tracing::debug!("Init");

    let recognizer = SpeechRecognizer::new();
    recognizer
        .start_recognize()
        .await
        .expect("Cannot start start_recognize");

    tracing::debug!("End");
}
