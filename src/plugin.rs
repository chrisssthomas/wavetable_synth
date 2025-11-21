use vst::prelude::*;
use std::sync::{Arc, Mutex};
use crate::voice::{VoiceManager, VoiceSettings};
use crate::oscillator::WaveType;

// VST Plugin structure
pub struct WavetableSynthVST {
    voice_manager: Arc<Mutex<VoiceManager>>,
    settings: VoiceSettings,
    sample_rate: f32,
    output_buffer: Vec<f32>,
    host: HostCallback,
}

impl WavetableSynthVST {
    fn new(host: HostCallback) -> Self {
        let sample_rate = 44100.0;
        let voice_manager = Arc::new(Mutex::new(VoiceManager::new(sample_rate, 16)));
        
        Self {
            voice_manager,
            settings: VoiceSettings::default(),
            sample_rate,
            output_buffer: Vec::with_capacity(1024),
            host,
        }
    }

    fn handle_midi_event(&mut self, data: [u8; 3]) {
        let status = data[0] & 0xF0;
        let channel = data[0] & 0x0F;
        let data1 = data[1];
        let data2 = data[2];

        match status {
            // Note On
            0x90 if data2 > 0 => {
                if let Ok(mut vm) = self.voice_manager.lock() {
                    vm.note_on(data1, data2);
                }
            }
            // Note Off or Note On with velocity 0
            0x80 | 0x90 => {
                if let Ok(mut vm) = self.voice_manager.lock() {
                    vm.note_off(data1);
                }
            }
            // Control Change
            0xB0 => {
                self.handle_cc(data1, data2);
            }
            _ => {}
        }
    }

    fn handle_cc(&mut self, controller: u8, value: u8) {
        let normalized_value = value as f32 / 127.0;
        
        match controller {
            1 => {
                // Modulation wheel - Filter frequency
                self.settings.filter_freq = 200.0 + normalized_value * 7800.0;
                if let Ok(mut vm) = self.voice_manager.lock() {
                    vm.update_settings(self.settings.clone());
                }
            }
            7 => {
                // Volume
                // Could implement master volume here
            }
            74 => {
                // Filter cutoff (often mapped to this CC)
                self.settings.filter_freq = 200.0 + normalized_value * 7800.0;
                if let Ok(mut vm) = self.voice_manager.lock() {
                    vm.update_settings(self.settings.clone());
                }
            }
            71 => {
                // Filter resonance (MS-20 style: 0.0 to 0.95)
                self.settings.filter_resonance = normalized_value * 0.95;
                if let Ok(mut vm) = self.voice_manager.lock() {
                    vm.update_settings(self.settings.clone());
                }
            }
            _ => {}
        }
    }
}

impl Plugin for WavetableSynthVST {
    fn new(host: HostCallback) -> Self {
        WavetableSynthVST::new(host)
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Wavetable Synth".to_string(),
            vendor: "RustSynth".to_string(),
            unique_id: 0x57617665, // "Wave" in ASCII
            version: 1,
            inputs: 0,
            outputs: 2, // Stereo output
            parameters: 12, // Increased for dual oscillators
            category: Category::Synth,
            f64_precision: false,
            preset_chunks: false,
            midi_inputs: 1,
            midi_outputs: 0,
            silent_when_stopped: true,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
        self.voice_manager = Arc::new(Mutex::new(VoiceManager::new(rate, 16)));
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Attack".to_string(),
            1 => "Decay".to_string(),
            2 => "Sustain".to_string(),
            3 => "Release".to_string(),
            4 => "Filter Freq".to_string(),
            5 => "Filter Res".to_string(),
            6 => "OSC1 Wave".to_string(),
            7 => "OSC2 Wave".to_string(),
            8 => "OSC Mix".to_string(),
            9 => "OSC2 Detune".to_string(),
            10 => "Master Vol".to_string(),
            11 => "Reserved".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.settings.attack_time / 2.0,
            1 => self.settings.decay_time / 2.0,
            2 => self.settings.sustain_level,
            3 => self.settings.release_time / 3.0,
            4 => (self.settings.filter_freq - 200.0) / 7800.0,
            5 => self.settings.filter_resonance / 0.95,
            6 => match self.settings.osc1_wave_type {
                WaveType::Sine => 0.0,
                WaveType::Sawtooth => 0.33,
                WaveType::Square => 0.66,
                WaveType::Triangle => 1.0,
            },
            7 => match self.settings.osc2_wave_type {
                WaveType::Sine => 0.0,
                WaveType::Sawtooth => 0.33,
                WaveType::Square => 0.66,
                WaveType::Triangle => 1.0,
            },
            8 => self.settings.osc_mix,
            9 => (self.settings.osc2_detune + 50.0) / 100.0, // -50 to +50 cents normalized to 0-1
            10 => self.settings.master_volume,
            11 => 0.0, // Reserved
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, value: f32) {
        match index {
            0 => {
                self.settings.attack_time = value * 2.0;
            }
            1 => {
                self.settings.decay_time = value * 2.0;
            }
            2 => {
                self.settings.sustain_level = value;
            }
            3 => {
                self.settings.release_time = value * 3.0;
            }
            4 => {
                self.settings.filter_freq = 200.0 + value * 7800.0;
            }
            5 => {
                self.settings.filter_resonance = value * 0.95;
            }
            6 => {
                self.settings.osc1_wave_type = match value {
                    v if v < 0.25 => WaveType::Sine,
                    v if v < 0.5 => WaveType::Sawtooth,
                    v if v < 0.75 => WaveType::Square,
                    _ => WaveType::Triangle,
                };
            }
            7 => {
                self.settings.osc2_wave_type = match value {
                    v if v < 0.25 => WaveType::Sine,
                    v if v < 0.5 => WaveType::Sawtooth,
                    v if v < 0.75 => WaveType::Square,
                    _ => WaveType::Triangle,
                };
            }
            8 => {
                self.settings.osc_mix = value;
            }
            9 => {
                self.settings.osc2_detune = (value * 100.0) - 50.0; // 0-1 normalized to -50 to +50 cents
            }
            10 => {
                self.settings.master_volume = value;
            }
            _ => {}
        }

        if let Ok(mut vm) = self.voice_manager.lock() {
            vm.update_settings(self.settings.clone());
        }
    }

    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    self.handle_midi_event(ev.data);
                }
                _ => {}
            }
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        
        // Clear outputs
        for output in outputs.iter_mut() {
            for sample in output.iter_mut() {
                *sample = 0.0;
            }
        }

        if let Ok(mut vm) = self.voice_manager.lock() {
            for i in 0..samples {
                let sample = vm.get_sample();
                
                // Apply to all output channels (mono to stereo)
                for output in outputs.iter_mut() {
                    output[i] = sample;
                }
            }
        }
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe,
        }
    }
}

// VST plugin entry point
plugin_main!(WavetableSynthVST);