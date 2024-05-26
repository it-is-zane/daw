#![allow(dead_code)]

mod audio_server;

fn main() {
    let server = audio_server::AudioServer::new();

    // midi tests
    fn midi_to_freq(note: &u8) -> f32 {
        440f32 * 2f32.powf((*note as f32 - 69f32) / 12f32)
    }

    let midi_in = midir::MidiInput::new("midir reading input").unwrap();
    let port = midi_in.ports().into_iter().last().unwrap();
    let _conn_in = midi_in.connect(
        &port,
        "midir-read-input",
        move |stamp, message, _| match message {
            [_, 0, v] => println!("Modulation Wheel == {v}"),
            // key pressed
            [144, n, v] => {
                let freq = midi_to_freq(n);

                let mut events = server.events.lock().unwrap();
                events.push(audio_server::AudioEvent::new(
                    freq,
                    *v as f32 / (127.0 * 5.0),
                    server.config.clone(),
                ));

                println!("key {n} (freq = {}) pressed {v}", freq);
            }
            // key released
            [128, n, v] => {
                let freq = midi_to_freq(n);

                let mut events = server.events.lock().unwrap();
                for e in events.iter_mut() {
                    if e.freq == freq && !e.released {
                        e.release();
                        println!("key {n} (freq = {}) released {v}", freq);
                        break;
                    }
                }

                // println!("key {n} (freq = {}) released {v}", freq);
            }
            _ => println!("{}: {:?}", stamp, message),
        },
        (),
    );

    // wait for use to close program by pressing enter
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
