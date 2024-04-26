use midir::MidiInput;
use rodio::source::SineWave;
use rodio::Source;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn calculate_frequency(key: u8) -> f32 {
    let a4 = 440.0;
    let a4_key = 69;
    let key_diff = key as i32 - a4_key as i32;
    let frequency = a4 * 2.0_f32.powf(key_diff as f32 / 12.0);
    frequency
}

fn main() -> Result<(), Box<dyn Error>> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let frequencies: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

    let midi_in = MidiInput::new("midir reading input")?;
    let in_ports = midi_in.ports();

    let in_port = match in_ports.first() {
        Some(port) => port,
        None => return Err("no input port available".into()),
    };

    println!("Listening on: {}", midi_in.port_name(in_port)?);

    let frequencies_clone = Arc::clone(&frequencies);
    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |_, message, _| {
        match message {
            [0x90, key, ..] => { // Note On event
                let frequency = calculate_frequency(*key);
                frequencies_clone.lock().unwrap().push(frequency);
            },
            _ => (),
        }
    }, ())?;

    let frequencies_clone = Arc::clone(&frequencies);
    let handle = thread::spawn(move || {
        loop {
            let mut frequencies = frequencies_clone.lock().unwrap();
            if let Some(frequency) = frequencies.pop() {
                let source = SineWave::new(frequency as u32);
                let source_with_duration = source.take_duration(Duration::from_secs_f32(0.5));
                if let Err(e) = stream_handle.play_raw(source_with_duration.convert_samples()) {
                    eprintln!("Error playing frequency: {}", e);
                }
                thread::sleep(Duration::from_secs_f32(0.5)); // Adjust the delay as needed
            }
        }
    });

    loop {
        thread::sleep(Duration::from_secs(1)); // Keep the main thread alive
    }

    Ok(())
}