use midly::{MidiMessage, Smf, TrackEventKind};
use rodio::{source::SineWave, OutputStream, Source};
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;

// Function to calculate frequency from MIDI note number
fn calculate_frequency(note: u8) -> f32 {
    (2.0f32).powf((note as f32 - 69.0) / 12.0) * 440.0
}

fn main() {
    // Create a new audio stream
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

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
                TrackEventKind::Midi { message, .. } => match message {
                    MidiMessage::NoteOn { key, .. } => {
                        let frequency = calculate_frequency(key.into());
                        frequencies.push(frequency);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    // Play the frequencies
    for frequency in frequencies {
        let source = SineWave::new(frequency as u32);
        let source_with_duration = source.take_duration(Duration::from_secs_f32(0.5));
        stream_handle
            .play_raw(source_with_duration.convert_samples())
            .unwrap();
        thread::sleep(Duration::from_secs_f32(0.5)); // Adjust the delay as needed
    }
}
