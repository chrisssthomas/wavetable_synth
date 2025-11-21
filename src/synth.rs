use crate::voice::{VoiceManager, VoiceSettings};
use rodio::Source;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::VecDeque;

const WAVEFORM_BUFFER_SIZE: usize = 512;

pub struct PolySynth {
    voice_manager: Arc<Mutex<VoiceManager>>,
    sample_rate: u32,
    waveform_buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl PolySynth {
    pub fn new(sample_rate: u32, max_voices: usize) -> Self {
        let voice_manager = Arc::new(Mutex::new(VoiceManager::new(sample_rate as f32, max_voices)));
        
        Self {
            voice_manager,
            sample_rate,
            waveform_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(WAVEFORM_BUFFER_SIZE))),
        }
    }

    pub fn get_waveform_buffer(&self) -> Option<VecDeque<f32>> {
        if let Ok(buffer) = self.waveform_buffer.lock() {
            Some(buffer.clone())
        } else {
            None
        }
    }

    pub fn get_active_voice_count(&self) -> usize {
        if let Ok(vm) = self.voice_manager.lock() {
            vm.active_voice_count()
        } else {
            0
        }
    }

    pub fn note_on(&self, key: u8, velocity: u8) {
        if let Ok(mut vm) = self.voice_manager.lock() {
            vm.note_on(key, velocity);
        }
    }

    pub fn note_off(&self, key: u8) {
        if let Ok(mut vm) = self.voice_manager.lock() {
            vm.note_off(key);
        }
    }

    pub fn update_settings(&self, settings: VoiceSettings) {
        if let Ok(mut vm) = self.voice_manager.lock() {
            vm.update_settings(settings);
        }
    }
}

impl Clone for PolySynth {
    fn clone(&self) -> Self {
        Self {
            voice_manager: Arc::clone(&self.voice_manager),
            sample_rate: self.sample_rate,
            waveform_buffer: Arc::clone(&self.waveform_buffer),
        }
    }
}

impl Iterator for PolySynth {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(mut vm) = self.voice_manager.lock() {
            let sample = vm.get_sample();
            
            // Store sample for waveform display (downsample for GUI)
            static mut SAMPLE_COUNTER: usize = 0;
            unsafe {
                SAMPLE_COUNTER += 1;
                if SAMPLE_COUNTER % 64 == 0 {  // Downsample by factor of 64 for 60fps GUI
                    if let Ok(mut buffer) = self.waveform_buffer.lock() {
                        if buffer.len() >= WAVEFORM_BUFFER_SIZE {
                            buffer.pop_front();
                        }
                        buffer.push_back(sample);
                    }
                }
            }
            
            Some(sample)
        } else {
            Some(0.0)
        }
    }
}

impl Source for PolySynth {
    fn channels(&self) -> u16 {
        1 // Mono output
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn current_frame_len(&self) -> Option<usize> {
        None // Infinite source
    }

    fn total_duration(&self) -> Option<Duration> {
        None // Infinite duration
    }
}