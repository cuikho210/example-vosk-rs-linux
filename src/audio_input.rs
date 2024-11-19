use cpal::{
    traits::{DeviceTrait, HostTrait},
    SizedSample, Stream, SupportedStreamConfig,
};
use dasp_sample::ToSample;
use futures::{channel::mpsc, SinkExt};

pub struct AudioInput {
    input_device: cpal::Device,
    pub config: SupportedStreamConfig,
}
unsafe impl Send for AudioInput {}
impl Default for AudioInput {
    fn default() -> Self {
        let host = cpal::default_host();
        let input_device = host
            .default_input_device()
            .expect("Cannot get input device");
        let config = input_device
            .default_input_config()
            .expect("Failed to load default input config");

        Self {
            config,
            input_device,
        }
    }
}
impl AudioInput {
    pub fn new_mono_stream(&self, buffer_size: usize) -> (Stream, mpsc::Receiver<Vec<i16>>) {
        let (tx, rx) = mpsc::channel::<Vec<i16>>(buffer_size);

        let stream = match self.config.sample_format() {
            cpal::SampleFormat::I8 => self.make_mono_stream::<i8>(tx),
            cpal::SampleFormat::I16 => self.make_mono_stream::<i16>(tx),
            cpal::SampleFormat::I32 => self.make_mono_stream::<i32>(tx),
            cpal::SampleFormat::I64 => self.make_mono_stream::<i64>(tx),
            cpal::SampleFormat::U8 => self.make_mono_stream::<u8>(tx),
            cpal::SampleFormat::U16 => self.make_mono_stream::<u16>(tx),
            cpal::SampleFormat::U32 => self.make_mono_stream::<u32>(tx),
            cpal::SampleFormat::U64 => self.make_mono_stream::<u64>(tx),
            cpal::SampleFormat::F32 => self.make_mono_stream::<f32>(tx),
            cpal::SampleFormat::F64 => self.make_mono_stream::<f64>(tx),
            _ => panic!("[Synth.new_stream] Unsupported format"),
        };

        (stream, rx)
    }

    fn make_mono_stream<T>(&self, tx: mpsc::Sender<Vec<i16>>) -> cpal::Stream
    where
        T: SizedSample + ToSample<i16>,
    {
        let err_fn = |err| tracing::error!("an error occurred on stream: {}", err);
        let channels = self.config.channels();
        let tx = tx.clone();

        let stream = self
            .input_device
            .build_input_stream(
                &self.config.config(),
                move |data: &[T], _| {
                    let mono_samples: Vec<i16> = data
                        .chunks_exact(channels as usize)
                        .map(|frame| {
                            // NOTE: This makes cpal alsa crash
                            // frame
                            //     .into_iter()
                            //     .map(|s| s.to_sample())
                            //     .reduce(|acc, e| (acc + e) / 2)
                            //     .unwrap_or(0)

                            frame[0].to_sample()
                        })
                        .collect();

                    let mut tx = tx.clone();
                    futures::executor::block_on(async move {
                        if let Err(err) = tx.send(mono_samples).await {
                            tracing::error!("Cannot send mono_samples: {}", err);
                        }
                    });
                },
                err_fn,
                None,
            )
            .unwrap();

        stream
    }
}
