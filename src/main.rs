use midly::TrackEventKind;
use midly::{MidiMessage, Smf};
use rodio::{OutputStream, Source};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;

mod wavetable_oscillator;
use wavetable_oscillator::WavetableOscillator;

mod envelope;
use envelope::ADSR;
use rodio::buffer::SamplesBuffer;

struct Synthesizer {
    oscillators: VecDeque<WavetableOscillator>, // Use a VecDeque to efficiently add and remove elements
    max_polyphony: usize,
}

impl Synthesizer {
    // create a wavetable for the synth to use
    fn create_wavetable() -> Vec<f32> {
        let wave_table_size = 64;
        let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
        for n in 0..wave_table_size {
            wave_table.push((2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin());
        }
        wave_table
    }

    fn new(max_polyphony: usize /* other parameters */) -> Self {
        let mut oscillators: VecDeque<WavetableOscillator> = VecDeque::new();
        // create 4 oscillators with an empty wave table
        for _ in 0..max_polyphony {
            oscillators.push_back(WavetableOscillator::new(
                44100,                           // Sample rate
                Synthesizer::create_wavetable(), // Wave table
                0.3,                             // Volume
                ADSR::new(0.2, 0.2, 0.8, 0.3),   // ADSR envelope
            ));
        }
        Synthesizer {
            oscillators,
            max_polyphony,
        }
    }

    fn note_on(&mut self, frequency: f32, _velocity: f32, time: f32) {
        // Find the first oscillator that is not playing
        let oscillator_index = self
            .oscillators
            .iter()
            .position(|oscillator| !oscillator.is_playing());

        if let Some(index) = oscillator_index {
            // Start the oscillator and set its frequency
            let oscillator = &mut self.oscillators[index];
            oscillator.adsr.start(time);
            oscillator.set_frequency(frequency);
        } else if self.oscillators.len() < self.max_polyphony {
            // If all oscillators are playing and we haven't reached the polyphony limit,
            // create a new oscillator, start it, and add it to the end of the list
            let mut oscillator = WavetableOscillator::new(
                44100,                           // Sample rate
                Synthesizer::create_wavetable(), // Wave table
                0.3,                             // Volume
                ADSR::new(0.2, 0.2, 0.8, 0.3),   // ADSR envelope
            );
            oscillator.adsr.start(time);
            oscillator.set_frequency(frequency);
            self.oscillators.push_back(oscillator);
        }

        // Remove the oldest oscillator if the polyphony limit is reached
        if self.oscillators.len() > self.max_polyphony {
            self.oscillators.pop_front();
        }
    }

    fn note_off(&mut self, frequency: f32, time: f32) {
        // Find the oscillator with the given frequency and stop it
        for oscillator in self.oscillators.iter_mut() {
            if (oscillator.index_increment * oscillator.wave_table.len() as f32 / 44100.0
                - frequency)
                .abs()
                < 0.1
            {
                oscillator.adsr.stop(time);
            }
        }
    }

    fn next(&mut self) -> Option<f32> {
        let mut sample = 0.0;
        for oscillator in self.oscillators.iter_mut() {
            sample += oscillator.get_sample()
                * oscillator
                    .adsr
                    .value(std::time::Instant::now().elapsed().as_secs_f32());
        }
        Some(sample)
    }
}

fn main() {
    let wave_table_size = 64;
    let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
    for n in 0..wave_table_size {
        wave_table.push((2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin());
    }

    let _adsr = ADSR::new(0.1, 0.1, 0.8, 0.1); // Set ADSR parameters

    let mut synthesizer = Synthesizer::new(4);

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
    for frequency in frequencies.iter() {
        synthesizer.note_on(*frequency, 0.3, time);
        time += 0.5;

        let samples = synthesizer
            .oscillators
            .iter_mut()
            .map(|oscillator| oscillator.get_sample())
            .collect::<Vec<f32>>();
        let buffer = SamplesBuffer::new(1, 44100, samples);
        stream_handle.play_raw(buffer.convert_samples());

        synthesizer.note_off(*frequency, time);
    }
}

fn calculate_frequency(key: u8) -> f32 {
    let base_frequency = 440.0; // A4
    let key_offset = key as i32 - 69; // MIDI key number of A4 is 69
    let frequency = base_frequency * 2.0f32.powf(key_offset as f32 / 12.0);
    frequency
}
