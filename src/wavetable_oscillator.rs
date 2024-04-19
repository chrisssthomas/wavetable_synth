use crate::envelope::ADSR;
use std::time::Duration; // Add the missing envelope module to the crate root.

use rodio::Source;

pub struct WavetableOscillator {
    sample_rate: u32,
    wave_table: Vec<f32>,
    index: f32,
    index_increment: f32,
    volume: f32,
    adsr: ADSR,
}

impl WavetableOscillator {
    pub fn new(
        sample_rate: u32,
        wave_table: Vec<f32>,
        volume: f32,
        adsr: ADSR,
    ) -> WavetableOscillator {
        return WavetableOscillator {
            sample_rate,
            wave_table,
            index: 0.0,
            index_increment: 0.0,
            volume,
            adsr,
        };
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.index_increment = frequency * self.wave_table.len() as f32 / self.sample_rate as f32;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    fn get_sample(&mut self) -> f32 {
        let sample = self.lerp();
        self.index += self.index_increment;
        self.index %= self.wave_table.len() as f32;
        return sample * self.volume;
    }

    fn lerp(&self) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % self.wave_table.len();

        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        return truncated_index_weight * self.wave_table[truncated_index]
            + next_index_weight * self.wave_table[next_index];
    }

    pub fn clone(&self) -> WavetableOscillator {
        return WavetableOscillator {
            sample_rate: self.sample_rate,
            wave_table: self.wave_table.clone(),
            index: self.index,
            index_increment: self.index_increment,
            volume: self.volume,
            adsr: self.adsr.clone(),
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
