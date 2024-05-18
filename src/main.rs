#![allow(dead_code)]

mod audio_server;

fn main() {
    let server = audio_server::AudioServer::new();

    {
        // add synths
        let synths = server.synths.clone();
        let mut synths = synths.lock().unwrap();

        synths.push(Box::new(synths::Noise::new()));

        for freq in vec![32.7032, 41.20344, 48.99943, 439.0, 523.2511, 659.2551] {
            synths.push(Box::new(synths::SinWave::new(freq, &server.config)));
        }
    }

    let mut temp = String::new();
    let _ = std::io::stdin().read_line(&mut temp);
}

mod engine {
    use crate::audio_server;

    pub struct Engine {
        audio_server: audio_server::AudioServer,
    }

    impl Engine {}
}

mod synths {
    use crate::audio_server;
    use rand::Rng;

    pub struct Noise {}
    impl Noise {
        pub fn new() -> Self {
            Noise {}
        }
    }

    impl audio_server::Synth for Noise {
        fn write_to_buffer(&mut self, buffer: &mut [f32], garbage: bool) {
            if garbage {
                buffer
                    .iter_mut()
                    .for_each(|v| *v = rand::thread_rng().gen_range(-1.0..1.0));
            } else {
                buffer
                    .iter_mut()
                    .for_each(|v| *v += rand::thread_rng().gen_range(-1.0..1.0));
            }
        }
    }

    pub struct SinWave {
        freq: f32,
        config: cpal::StreamConfig,
        clock: f32,
    }

    impl SinWave {
        pub fn new(freq: f32, config: &cpal::StreamConfig) -> Self {
            Self {
                freq,
                config: config.clone(),
                clock: 0.0,
            }
        }
    }

    impl audio_server::Synth for SinWave {
        fn write_to_buffer(&mut self, buffer: &mut [f32], garbage: bool) {
            let sample_rate = self.config.sample_rate.0 as f32;

            if garbage {
                for value in buffer.iter_mut() {
                    self.clock = (self.clock + 1.0) % (sample_rate / self.freq);

                    *value = (self.clock * std::f32::consts::TAU * self.freq / sample_rate).sin();
                }
            } else {
                for value in buffer.iter_mut() {
                    self.clock = (self.clock + 1.0) % (sample_rate / self.freq);

                    *value += (self.clock * std::f32::consts::TAU * self.freq / sample_rate).sin();
                }
            }
        }
    }
}
