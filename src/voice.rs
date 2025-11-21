use crate::envelope::ADSR;
use crate::oscillator::{AnalogOscillator, WaveType};
use crate::filter::LowPassFilter;
use crate::distortion::{AnalogDistortion, DistortionType};
use crate::reverb::{AnalogReverb, ReverbType};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct VoiceSettings {
    // Dual oscillator setup
    pub osc1_wave_type: WaveType,
    pub osc2_wave_type: WaveType,
    pub osc1_shape: f32,     // Waveform shaping 0.0-1.0
    pub osc2_shape: f32,     // Waveform shaping 0.0-1.0
    pub osc_mix: f32,        // 0.0 = OSC1 only, 1.0 = OSC2 only, 0.5 = equal mix
    pub osc2_detune: f32,    // Detune OSC2 in cents (-50 to +50)
    
    // ADSR envelope
    pub attack_time: f32,
    pub decay_time: f32,
    pub sustain_level: f32,
    pub release_time: f32,
    
    // Filter
    pub filter_freq: f32,
    pub filter_resonance: f32,
    
    // Distortion
    pub distortion_type: DistortionType,
    pub distortion_drive: f32,
    pub distortion_tone: f32,
    pub distortion_level: f32,
    
    // Reverb
    pub reverb_type: ReverbType,
    pub reverb_size: f32,
    pub reverb_decay: f32,
    pub reverb_mix: f32,
    
    // Master volume
    pub master_volume: f32,  // 0.0 to 1.0
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            osc1_wave_type: WaveType::Sawtooth,
            osc2_wave_type: WaveType::Square,
            osc1_shape: 0.5,        // Moderate shaping by default
            osc2_shape: 0.3,        // Less shaping on osc2
            osc_mix: 0.5,           // Equal mix by default
            osc2_detune: 0.0,       // No detune by default
            attack_time: 0.001,     // Very quick attack to avoid clicks
            decay_time: 0.1,
            sustain_level: 0.7,
            release_time: 0.1,      // Shorter release to avoid trailing artifacts
            filter_freq: 2000.0,
            filter_resonance: 0.3,  // MS-20 style resonance (0.0-0.95)
            
            // Distortion defaults - moderate overdrive
            distortion_type: DistortionType::Overdrive,
            distortion_drive: 0.35,
            distortion_tone: 0.65,
            distortion_level: 0.8,
            
            // Reverb defaults - subtle room ambience
            reverb_type: ReverbType::Room,
            reverb_size: 0.4,
            reverb_decay: 0.5,
            reverb_mix: 0.2,
            
            master_volume: 1.0,     // Maximum volume for audibility
        }
    }
}

pub struct Voice {
    pub osc1: AnalogOscillator,
    pub osc2: AnalogOscillator,
    pub envelope: ADSR,
    pub filter: LowPassFilter,
    pub distortion: AnalogDistortion,
    pub velocity: f32,
    pub start_time: Instant,
    pub is_active: bool,
    pub is_released: bool,
}

impl Voice {
    pub fn new(midi_key: u8, velocity: f32, settings: &VoiceSettings, sample_rate: f32) -> Self {
        let frequency = calculate_frequency(midi_key);
        
        // Setup first oscillator
        let mut osc1 = AnalogOscillator::new(settings.osc1_wave_type, sample_rate);
        osc1.set_frequency(frequency);
        osc1.set_shape(settings.osc1_shape);  // Apply shape warping
        
        // Setup second oscillator with detune
        let detune_factor = 2.0_f32.powf(settings.osc2_detune / 1200.0); // Convert cents to frequency ratio
        let osc2_frequency = frequency * detune_factor;
        let mut osc2 = AnalogOscillator::new(settings.osc2_wave_type, sample_rate);
        osc2.set_frequency(osc2_frequency);
        osc2.set_shape(settings.osc2_shape);  // Apply shape warping
        
        let envelope = ADSR::new(
            settings.attack_time,
            settings.decay_time,
            settings.sustain_level,
            settings.release_time,
        );

        let mut filter = LowPassFilter::new(sample_rate);
        filter.set_cutoff(settings.filter_freq);
        filter.set_resonance(settings.filter_resonance);

        // Setup distortion with current settings
        let mut distortion = AnalogDistortion::new(sample_rate);
        distortion.set_type(settings.distortion_type);
        distortion.set_drive(settings.distortion_drive);
        distortion.set_tone(settings.distortion_tone);
        distortion.set_level(settings.distortion_level);

        // Reset oscillators to avoid phase jumps
        osc1.reset();
        osc2.reset();

        Self {
            osc1,
            osc2,
            envelope,
            filter,
            distortion,
            velocity: velocity / 127.0, // Normalize MIDI velocity
            start_time: Instant::now(),
            is_active: true,
            is_released: false,
        }
    }

    pub fn note_off(&mut self) {
        if !self.is_released {
            // Pass the current time directly (not elapsed time)
            let current_time = self.start_time.elapsed().as_secs_f32();
            self.envelope.stop(current_time);
            self.is_released = true;
        }
    }

    pub fn get_sample(&mut self, settings: &VoiceSettings) -> f32 {
        if !self.is_active {
            return 0.0;
        }

        let elapsed = self.start_time.elapsed().as_secs_f32();
        let envelope_value = self.envelope.value(elapsed);
        
        // Only mark as inactive when envelope value reaches near-zero AND is in release phase
        if self.is_released && envelope_value < 0.001 {
            self.is_active = false;
            return 0.0;
        }

        // Get samples from both oscillators
        let osc1_sample = self.osc1.get_sample();
        let osc2_sample = self.osc2.get_sample();
        
        // Mix oscillators with proper gain staging
        let mixed_sample = osc1_sample * (1.0 - settings.osc_mix) + osc2_sample * settings.osc_mix;
        
        // Apply filter first
        let filtered_sample = self.filter.process(mixed_sample);
        
        // Apply distortion after filter (classic signal chain: OSC -> FILTER -> DISTORTION)
        let distorted_sample = self.distortion.process(filtered_sample);
        
        // Convert MIDI velocity (0-127) to linear gain (0.0-1.0)
        let velocity_gain = self.velocity;
        
        // Master output: envelope × velocity × master_volume × gain boost
        // Reduced gain since distortion provides additional gain
        let final_gain = 25.0; // Reduced from 50.0 since distortion adds gain
        distorted_sample * envelope_value * velocity_gain * settings.master_volume * final_gain
    }

    pub fn start_envelope(&mut self) {
        self.envelope.start(0.0);
    }

    pub fn update_filter_settings(&mut self, filter_freq: f32, filter_resonance: f32) {
        self.filter.set_cutoff(filter_freq);
        self.filter.set_resonance(filter_resonance);
    }

    pub fn update_distortion_settings(&mut self, settings: &VoiceSettings) {
        self.distortion.set_type(settings.distortion_type);
        self.distortion.set_drive(settings.distortion_drive);
        self.distortion.set_tone(settings.distortion_tone);
        self.distortion.set_level(settings.distortion_level);
    }

    pub fn set_settings(&mut self, settings: &VoiceSettings) {
        self.update_filter_settings(settings.filter_freq, settings.filter_resonance);
        self.update_distortion_settings(settings);
        
        // Update ADSR parameters for real-time synthesis programming
        self.envelope.update_attack(settings.attack_time);
        self.envelope.update_decay(settings.decay_time);
        self.envelope.update_sustain(settings.sustain_level);
        self.envelope.update_release(settings.release_time);
        
        // Update oscillator shapes
        self.osc1.set_shape(settings.osc1_shape);
        self.osc2.set_shape(settings.osc2_shape);
        
        // Note: We avoid updating oscillator types on existing voices to prevent audio clicks
    }
}

pub struct VoiceManager {
    voices: HashMap<u8, Voice>,
    settings: VoiceSettings,
    reverb: AnalogReverb,  // Global reverb for all voices
    sample_rate: f32,
    max_voices: usize,
}

impl VoiceManager {
    pub fn new(sample_rate: f32, max_voices: usize) -> Self {
        let mut reverb = AnalogReverb::new(sample_rate);
        let default_settings = VoiceSettings::default();
        
        // Initialize reverb with default settings
        reverb.set_type(default_settings.reverb_type);
        reverb.set_size(default_settings.reverb_size);
        reverb.set_decay(default_settings.reverb_decay);
        reverb.set_mix(default_settings.reverb_mix);
        
        Self {
            voices: HashMap::new(),
            settings: default_settings,
            reverb,
            sample_rate,
            max_voices,
        }
    }

    pub fn note_on(&mut self, midi_key: u8, velocity: u8) {
        // Remove old voice if key is already pressed (prevent double triggering)
        if let Some(voice) = self.voices.get_mut(&midi_key) {
            voice.note_off();
        }

        // Voice stealing if at max capacity - be more aggressive
        if self.voices.len() >= self.max_voices {
            // First try to find a released voice that's almost finished
            let oldest_released = self.voices
                .iter()
                .filter(|(_, v)| v.is_released)
                .min_by_key(|(_, v)| v.start_time)
                .map(|(k, _)| *k);
            
            let key_to_remove = oldest_released.or_else(|| {
                // If no released voices, steal the oldest playing voice
                self.voices
                    .iter()
                    .min_by_key(|(_, v)| v.start_time)
                    .map(|(k, _)| *k)
            });
            
            if let Some(key) = key_to_remove {
                self.voices.remove(&key);
            }
        }

        let mut voice = Voice::new(midi_key, velocity as f32 / 127.0, &self.settings, self.sample_rate);
        voice.start_envelope();
        self.voices.insert(midi_key, voice);
    }

    pub fn note_off(&mut self, midi_key: u8) {
        if let Some(voice) = self.voices.get_mut(&midi_key) {
            voice.note_off();
        }
    }

    pub fn get_sample(&mut self) -> f32 {
        // Gentle cleanup - only remove truly inactive voices (envelope near zero)
        self.voices.retain(|_, voice| {
            voice.is_active
        });
        
        // Mix all active voices, passing current settings
        let mixed_sample = self.voices
            .values_mut()
            .map(|voice| voice.get_sample(&self.settings))
            .sum::<f32>();
            
        // Apply gentle soft limiting to prevent harsh clipping
        let soft_limit = 0.95; // Leave some headroom
        let limited_sample = if mixed_sample.abs() > soft_limit {
            mixed_sample.signum() * (soft_limit + (mixed_sample.abs() - soft_limit) * 0.2)
        } else {
            mixed_sample
        };
        
        // Apply global reverb (after all voice processing)
        self.reverb.process(limited_sample)
    }

    pub fn update_settings(&mut self, settings: VoiceSettings) {
        // Update settings for future voices
        self.settings = settings.clone();
        
        // Update reverb parameters
        self.reverb.set_type(settings.reverb_type);
        self.reverb.set_size(settings.reverb_size);
        self.reverb.set_decay(settings.reverb_decay);
        self.reverb.set_mix(settings.reverb_mix);
        
        // Update filter and distortion settings on existing voices (real-time adjustment)
        for voice in self.voices.values_mut() {
            voice.set_settings(&settings);
        }
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.len()
    }
}

// Utility function for frequency calculation
fn calculate_frequency(midi_key: u8) -> f32 {
    let a4 = 440.0;
    let a4_key = 69;
    let key_diff = midi_key as i32 - a4_key as i32;
    a4 * 2.0_f32.powf(key_diff as f32 / 12.0)
}