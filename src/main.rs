use rodio::Source;
use core::time::Duration;
use rodio::OutputStream;
use midly::{Smf, MidiMessage};
use std::fs::File;
use std::io::Read;
use midly::TrackEventKind as EventKind;

struct WavetableOscillator {
    sample_rate: u32,
    wave_table: Vec<f32>,
    index: f32,
    index_increment: f32,
}

impl WavetableOscillator {
    fn new(sample_rate: u32, wave_table: Vec<f32>) -> WavetableOscillator {
        return WavetableOscillator {
            sample_rate: sample_rate,
            wave_table: wave_table,
            index: 0.0,
            index_increment: 0.0,
        };
    }

    fn set_frequency(&mut self, frequency: f32) {
        self.index_increment = frequency * self.wave_table.len() as f32 
                               / self.sample_rate as f32;
    }

    fn get_sample(&mut self) -> f32 {
        let sample = self.lerp();
        self.index += self.index_increment;
        self.index %= self.wave_table.len() as f32;
        return sample;
    }

    fn lerp(&self) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % self.wave_table.len();
        
        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        return truncated_index_weight * self.wave_table[truncated_index] 
               + next_index_weight * self.wave_table[next_index];
    }

    fn clone(&self) -> WavetableOscillator {
        return WavetableOscillator {
            sample_rate: self.sample_rate,
            wave_table: self.wave_table.clone(),
            index: self.index,
            index_increment: self.index_increment,
        };
    }
}

impl Iterator for WavetableOscillator {
    type Item = f32;
    
    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.get_sample());
    }
}

impl Source for WavetableOscillator {
    fn channels(&self) -> u16 {
        return 1;
    }

    fn sample_rate(&self) -> u32 {
        return self.sample_rate;
    }   

    fn current_frame_len(&self) -> Option<usize> {
        return None;
    }

    fn total_duration(&self) -> Option<Duration> {
        return None;
    }
}

fn main() {
    let wave_table_size = 64;
    let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
    for n in 0..wave_table_size {
        wave_table.push((2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin());
    }

    let mut oscillator = WavetableOscillator::new(44100, wave_table);

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
                EventKind::Midi { channel: _, message } => {
                    match message {
                        MidiMessage::NoteOn { key, vel: _ } => {
                            let frequency = calculate_frequency(key.into());
                            frequencies.push(frequency);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
    


    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    
    // Play the frequencies
    for frequency in frequencies {
        oscillator.set_frequency(frequency);
        let _ = stream_handle.play_raw(oscillator.clone());
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn calculate_frequency(key: u8) -> f32 {
    let base_frequency = 440.0; // A4
    let key_offset = key as i32 - 69; // MIDI key number of A4 is 69
    let frequency = base_frequency * 2.0f32.powf(key_offset as f32 / 12.0);
    frequency
}