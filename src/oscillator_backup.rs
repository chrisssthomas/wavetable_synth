use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveType {
    Sine,
    Sawtooth,
    Square,
    Triangle,
}

pub struct AnalogOscillator {
    wave_type: WaveType,
    frequency: f32,
    phase: f32,
    sample_rate: f32,
    phase_increment: f32,
    // Enhanced anti-aliasing with multiple stages
    dc_blocker_x1: f32,
    dc_blocker_y1: f32,
    // Analog modeling parameters
    drift_phase: f32,
    // Enhanced frequency smoothing
    target_frequency: f32,
    frequency_smoothing: f32,
    // Harmonic content for analog character
    sub_phase: f32,
    // Korg Minilogue-style waveform shaping
    shape: f32,  // 0.0 to 1.0 - warps the waveform character
}

impl AnalogOscillator {
    pub fn new(wave_type: WaveType, sample_rate: f32) -> Self {
        Self {
            wave_type,
            frequency: 440.0,
            phase: 0.0,
            sample_rate,
            phase_increment: 0.0,
            dc_blocker_x1: 0.0,
            dc_blocker_y1: 0.0,
            drift_phase: 0.0,
            target_frequency: 440.0,
            frequency_smoothing: 0.998, // Slightly faster frequency tracking
            sub_phase: 0.0,
            shape: 0.5, // Default neutral shape
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.target_frequency = frequency;
    }

    pub fn set_shape(&mut self, shape: f32) {
        self.shape = shape.clamp(0.0, 1.0);
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.sub_phase = 0.0;
        self.drift_phase = 0.0;
        self.dc_blocker_x1 = 0.0;
        self.dc_blocker_y1 = 0.0;
    }

    // Enhanced PolyBLEP sawtooth with aggressive analog modeling
    fn shaped_sawtooth(&mut self, t: f32) -> f32 {
        let freq_norm = self.frequency / self.sample_rate;
        
        // Base sawtooth with PolyBLEP anti-aliasing
        let mut saw = 2.0 * t - 1.0;
        
        // PolyBLEP correction
        if t < freq_norm {
            let tt = t / freq_norm;
            saw += tt * tt - 2.0 * tt + 1.0;
        } else if t > 1.0 - freq_norm {
            let tt = (t - 1.0) / freq_norm;
            saw += tt * tt + 2.0 * tt + 1.0;
        }
        
        // AGGRESSIVE ANALOG SAW MODELING
        // Multiply base output by 1.8 for more punch
        saw *= 1.8;
        
        // Shape 0.0 = clean saw, Shape 1.0 = extremely aggressive Moog-style saw
        if self.shape > 0.01 {
            // Waveform folding for harsh harmonics - MORE INTENSE
            let fold_amount = 1.0 + self.shape * 4.0;
            saw = (saw * fold_amount).tanh() / (fold_amount * 0.6); // Asymmetric compression
            
            // Step quantization for digital-aggressive sound - STRONGER
            let steps = 2.0 + self.shape * 20.0; // 2 to 22 steps (more extreme)
            saw = (saw * steps).round() / steps;
            
            // Asymmetric distortion for classic analog bite - ENHANCED
            if saw > 0.0 {
                saw = saw.powf(0.5 + self.shape * 0.8); // More aggressive positive peaks
            } else {
                saw = -(-saw).powf(1.4 - self.shape * 0.6); // Sharper negative valleys
            }
            
            // Wave shaping for Moog-style brightness
            let brightness = self.shape * 2.0;
            saw = (saw * brightness).tanh() + saw * (1.0 - brightness);
        }
        
        // Sub-harmonic for THICK analog growl - ENHANCED
        let sub_freq = self.frequency * 0.5;
        self.sub_phase += 2.0 * PI * sub_freq / self.sample_rate;
        if self.sub_phase >= 2.0 * PI {
            self.sub_phase -= 2.0 * PI;
        }
        
        // Multiple sub-harmonics for maximum thickness
        let sub_1 = (self.sub_phase.sin() * (0.12 + self.shape * 0.25)).tanh(); // Fundamental sub
        let sub_2 = ((self.sub_phase * 0.25).sin() * (0.06 + self.shape * 0.12)).tanh(); // Quarter frequency
        
        // Controlled aliasing for digital grit - MORE PRESENT
        let alias_harmonics = if self.shape > 0.3 {
            let alias_amount = (self.shape - 0.3) * 3.0; // Increased intensity
            (self.phase * 7.0).sin() * alias_amount * 0.25 + 
            (self.phase * 11.0).sin() * alias_amount * 0.15 +
            (self.phase * 13.0).sin() * alias_amount * 0.08 // Additional harmonic
        } else {
            0.0
        };
        
        // High-frequency content for classic analog brightness  
        let brightness_harmonics = (self.phase * 3.0).sin() * (0.08 + self.shape * 0.15);
        
        saw + sub_1 + sub_2 + alias_harmonics + brightness_harmonics
    }
        let sub_freq = self.frequency * 0.5;
        self.sub_phase += 2.0 * PI * sub_freq / self.sample_rate;
        if self.sub_phase >= 2.0 * PI {
            self.sub_phase -= 2.0 * PI;
        }
        let sub_harmonic = (self.sub_phase.sin() * (0.08 + self.shape * 0.12)).tanh();
        
        // Add intentional aliasing for digital grit (controlled by shape)
        let alias_harmonics = if self.shape > 0.5 {
            let alias_amount = (self.shape - 0.5) * 2.0;
            (self.phase * 7.0).sin() * alias_amount * 0.15 + 
            (self.phase * 11.0).sin() * alias_amount * 0.08
        } else {
            0.0
        };
        
        saw + sub_harmonic + alias_harmonics
    }

    // Enhanced square wave with aggressive PWM and distortion
    fn shaped_square(&self, t: f32) -> f32 {
        let freq_norm = self.frequency / self.sample_rate;
        
        // Extreme pulse width modulation based on shape (0.05 to 0.95 duty cycle)
        let pulse_width = 0.05 + self.shape * 0.9;
        let mut value = if t < pulse_width { 1.5 } else { -1.5 }; // Higher amplitude
        
        // PolyBLEP at rising edge
        if t < freq_norm {
            let tt = t / freq_norm;
            value += 3.0 * (tt - tt * tt); // Stronger edge correction
        } else if t > 1.0 - freq_norm {
            let tt = (t - 1.0) / freq_norm;
            value -= 3.0 * (tt + tt * tt);
        }
        
        // PolyBLEP at falling edge (at pulse_width position)
        let t_fall = if t >= pulse_width { t - pulse_width } else { t - pulse_width + 1.0 };
        if t_fall < freq_norm {
            let tt = t_fall / freq_norm;
            value -= 3.0 * (tt - tt * tt);
        } else if t_fall > 1.0 - freq_norm {
            let tt = (t_fall - 1.0) / freq_norm;
            value += 3.0 * (tt + tt * tt);
        }
        
        // AGGRESSIVE harmonic distortion based on shape
        if self.shape > 0.1 {
            let distortion = (self.shape - 0.1) * 2.5; // More intense distortion
            
            // Multi-stage wave folding for extreme character
            value = (value * (1.0 + distortion * 2.0)).tanh();
            value = (value * (1.0 + distortion)).tanh(); // Second stage
            
            // Add multiple sub-harmonics for MASSIVE growl
            let sub_quarter = (self.phase * 0.25).sin() * distortion * 0.4; // Quarter frequency
            let sub_half = (self.phase * 0.5).sin() * distortion * 0.3;     // Half frequency
            let sub_third = (self.phase * (1.0/3.0)).sin() * distortion * 0.2; // Third frequency
            
            value += sub_quarter + sub_half + sub_third;
            
            // Add bright harmonics for cutting presence
            let harmonics = (self.phase * 3.0).sin() * distortion * 0.25 +
                           (self.phase * 5.0).sin() * distortion * 0.15 +
                           (self.phase * 7.0).sin() * distortion * 0.08;
            
            value += harmonics;
        }
        
        value
    }

    // Shaped triangle with angular distortion
    fn shaped_triangle(&self, t: f32) -> f32 {
        // Base triangle wave
        let tri = if t < 0.25 {
            4.0 * t
        } else if t < 0.75 {
            2.0 - 4.0 * t
        } else {
            4.0 * t - 4.0
        };
        
        // Shape makes triangle more angular/stepped
        let shaped_tri = if self.shape > 0.1 {
            let sharpness = 1.0 + self.shape * 3.0; // 1.0 to 4.0
            tri.signum() * tri.abs().powf(1.0 / sharpness)
        } else {
            tri
        };
        
        // Add harmonic distortion for character
        let harmonic_3 = (3.0 * self.phase).sin() * (0.05 + self.shape * 0.15);
        let harmonic_5 = (5.0 * self.phase).sin() * (0.02 + self.shape * 0.08);
        
        shaped_tri + harmonic_3 + harmonic_5
    }

    // Enhanced sine with shape-controlled harmonic distortion
    fn shaped_sine(&self, _t: f32) -> f32 {
        let base_sine = self.phase.sin();
        
        if self.shape < 0.1 {
            // Pure sine
            base_sine
        } else {
            // Add controlled harmonic distortion
            let distortion = self.shape;
            let harmonics = (2.0 * self.phase).sin() * distortion * 0.3 +
                           (3.0 * self.phase).sin() * distortion * 0.15 +
                           (4.0 * self.phase).sin() * distortion * 0.08;
            
            // Soft saturation for tube-like warmth
            (base_sine + harmonics).tanh() * 0.8
        }
    }

    pub fn get_sample(&mut self) -> f32 {
        // Smooth frequency changes to avoid clicks
        self.frequency = self.frequency * self.frequency_smoothing + 
                        self.target_frequency * (1.0 - self.frequency_smoothing);
        self.phase_increment = 2.0 * PI * self.frequency / self.sample_rate;

        let t = self.phase / (2.0 * PI);
        
        let sample = match self.wave_type {
            WaveType::Sine => {
                self.shaped_sine(t)
            },
            WaveType::Sawtooth => {
                self.shaped_sawtooth(t)
            },
            WaveType::Square => {
                self.shaped_square(t)
            },
            WaveType::Triangle => {
                self.shaped_triangle(t)
            }
        };

        // Enhanced DC blocking filter
        let dc_blocked = sample - self.dc_blocker_x1 + 0.998 * self.dc_blocker_y1;
        self.dc_blocker_x1 = sample;
        self.dc_blocker_y1 = dc_blocked;

        self.phase += self.phase_increment;
        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        // AGGRESSIVE OUTPUT STAGE for maximum analog character
        // Much higher output level with enhanced saturation
        let output = dc_blocked * 4.5; // Increased from 3.5 for more drive
        
        // Multi-stage analog-style saturation for MASSIVE character
        let saturated = if output.abs() > 0.1 {
            // First stage: Soft compression
            let compressed = output.signum() * (1.0 - (-output.abs() * 2.0).exp()) * 1.5;
            
            // Second stage: Tube-style asymmetric saturation  
            let tube_saturation = if compressed > 0.0 {
                compressed * (1.0 + compressed * 0.3) // Positive peaks get enhanced harmonics
            } else {
                compressed * (1.0 - compressed * 0.1) // Negative valleys stay clean
            };
            
            // Third stage: Final limiting with harmonic preservation
            if tube_saturation > 1.2 {
                1.2 - (tube_saturation - 1.2) * 0.3
            } else if tube_saturation < -1.1 {
                -1.1 - (tube_saturation + 1.1) * 0.2  
            } else {
                tube_saturation
            }
        } else {
            output * 1.2 // Boost quiet signals for better dynamics
        };
        
        // Final clamp to prevent DA converter overload
        saturated.clamp(-1.5, 1.5)
    }
}