use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub trait Synth {
    fn write_to_buffer(&mut self, buffer: &mut [f32], garbage: bool);
}

pub struct AudioServer {
    pub synths: Arc<Mutex<Vec<Box<dyn Synth + Send + Sync>>>>,
    stream: cpal::Stream,
    pub config: cpal::StreamConfig,
}

impl AudioServer {
    pub fn new() -> Self {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("no output device available");

        // get the f32 configuration if available
        let supported_config = device
            .supported_output_configs()
            .expect("error while querying configs")
            .find(|config| config.sample_format() == cpal::SampleFormat::F32)
            .expect("tried to use SampleFormat::F32 but it was not available")
            .with_max_sample_rate();

        let config = supported_config.config();

        let synths: Arc<Mutex<Vec<Box<dyn Synth + Send + Sync>>>> =
            Arc::new(Mutex::new(Vec::new()));

        let synths_copy = synths.clone();

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    let mut synths = synths_copy.lock().unwrap();
                    let mut synth = synths.iter_mut();

                    // initialize
                    if let Some(synth) = synth.next() {
                        synth.write_to_buffer(data, true);
                    }

                    // add
                    for synth in synth {
                        synth.write_to_buffer(data, false);
                    }

                    // normalize
                    data.iter_mut()
                        .for_each(|value| *value /= synths.len() as f32);
                },
                move |err| eprintln!("{err}"),
                None,
            )
            .unwrap();

        stream.play().unwrap();

        AudioServer {
            synths,
            stream,
            config,
        }
    }
}
