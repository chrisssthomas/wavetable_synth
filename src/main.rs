use core::time::Duration;
use midly::TrackEventKind;
use midly::{MidiMessage, Smf};
use rodio::OutputStream;
use std::fs::File;
use std::io::Read;
use std::thread;

mod wavetable_oscillator;
use wavetable_oscillator::WavetableOscillator;

mod envelope;
use envelope::ADSR;

fn main() {
    let wave_table_size = 64;
    let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
    for n in 0..wave_table_size {
        wave_table.push((2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin());
    }

    let adsr = ADSR::new(0.1, 0.1, 0.8, 0.1); // Set ADSR parameters

    let mut oscillator = WavetableOscillator::new(44100, wave_table, 0.5, adsr);

    // Read the MIDI file
    let mut file = File::open("test.mid").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    // Parse the MIDI file
    let midi = Smf::parse(&buffer).unwrap();

    // Extract the frequency values from the MIDI file
    let mut frequencies: Vec<f32> = Vec::new();
    for (_, track) in midi.tracks.iter().enumerate() {
        for event in track.iter() {
            match event.kind {
                TrackEventKind::Midi {
                    channel: _,
                    message,
                } => match message {
                    MidiMessage::NoteOn { key, vel: _ } => {
                        let frequency = calculate_frequency(key.into());
                        frequencies.push(frequency);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // Play the frequencies
    let mut time = 0.0;
    for frequency in frequencies {
        oscillator.set_frequency(frequency);
        oscillator.set_volume(0.3);
        oscillator.adsr.start(time); // Start the ADSR envelope
        let _ = stream_handle.play_raw(oscillator.clone());
        thread::sleep(Duration::from_secs_f32(0.5)); // Adjust the delay as needed
        time += 0.5; // Increment the time
        oscillator.adsr.stop(time); // Stop the ADSR envelope
    }
}

fn calculate_frequency(key: u8) -> f32 {
    let base_frequency = 440.0; // A4
    let key_offset = key as i32 - 69; // MIDI key number of A4 is 69
    let frequency = base_frequency * 2.0f32.powf(key_offset as f32 / 12.0);
    frequency
}
