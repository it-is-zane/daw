use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub trait Synth {
    fn write_to_buffer(&mut self, buffer: &mut [f32], garbage: bool);
}

pub struct AudioServer {
    pub events: Arc<Mutex<Vec<AudioEvent>>>,
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

        let events = Arc::new(Mutex::new(Vec::<AudioEvent>::new()));
        let events_clone = events.clone();

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    let mut events = events_clone.lock().unwrap();

                    // remove finished events
                    if let Some(index) = events
                        .iter()
                        .enumerate()
                        .find_map(|(i, e)| e.done.then_some(i))
                    {
                        events.swap_remove(index);
                    }

                    let mut events = events.iter_mut();

                    // initialize
                    if let Some(event) = events.next() {
                        event.write_to_buffer(data, true);
                    } else {
                        data.fill(0.0);
                    }

                    // add
                    for event in events {
                        event.write_to_buffer(data, false);
                    }
                },
                move |err| eprintln!("{err}"),
                None,
            )
            .unwrap();

        stream.play().unwrap();

        AudioServer {
            events,
            stream,
            config,
        }
    }
}

pub struct AudioEvent {
    pub freq: f32,
    amp: f32,
    config: cpal::StreamConfig,
    clock: f32,
    pub released: bool,
    pub done: bool,
}

impl AudioEvent {
    pub fn new(freq: f32, amp: f32, config: cpal::StreamConfig) -> Self {
        Self {
            freq,
            amp,
            config,
            clock: 0.0,
            released: false,
            done: false,
        }
    }

    fn write_to_buffer(&mut self, buffer: &mut [f32], garbage: bool) {
        let time_constant = std::f32::consts::TAU / self.config.sample_rate.0 as f32;

        let mut value_calc = || {
            if self.released {
                self.amp -= time_constant * 0.1;

                if self.amp <= 0.0 {
                    self.amp = 0.0;
                    self.done = true;
                }
            }

            self.clock = (self.clock + 1.0) % (self.config.sample_rate.0 as f32 / self.freq);
            self.amp * (self.clock * self.freq * time_constant).sin()
        };

        if garbage {
            for value in buffer.iter_mut() {
                *value = value_calc();
            }
        } else {
            for value in buffer.iter_mut() {
                *value += value_calc();
            }
        }
    }

    pub fn release(&mut self) {
        self.released = true;
    }
}
