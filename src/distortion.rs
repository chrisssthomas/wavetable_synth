use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistortionType {
    Clean,        // Bypass
    Overdrive,    // Tube Screamer style soft clipping
    Distortion,   // Hard clipping with pre-gain
    Fuzz,         // Big Muff style asymmetric clipping
    Tube,         // Tube saturation modeling
}

pub struct AnalogDistortion {
    distortion_type: DistortionType,
    drive: f32,           // 0.0 to 1.0 (maps to different ranges per type)
    tone: f32,            // 0.0 to 1.0 (high frequency emphasis)
    level: f32,           // Output level compensation
    
    // Filter states for tone control
    tone_hp_x1: f32,
    tone_hp_y1: f32,
    tone_lp_x1: f32,
    tone_lp_y1: f32,
    
    // Pre/post filtering for realistic analog behavior
    input_hp_x1: f32,
    input_hp_y1: f32,
    output_lp_x1: f32,
    output_lp_y1: f32,
    
    sample_rate: f32,
}

impl AnalogDistortion {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            distortion_type: DistortionType::Clean,
            drive: 0.5,
            tone: 0.5,
            level: 0.7,
            
            tone_hp_x1: 0.0,
            tone_hp_y1: 0.0,
            tone_lp_x1: 0.0,
            tone_lp_y1: 0.0,
            
            input_hp_x1: 0.0,
            input_hp_y1: 0.0,
            output_lp_x1: 0.0,
            output_lp_y1: 0.0,
            
            sample_rate,
        }
    }

    pub fn set_type(&mut self, dist_type: DistortionType) {
        self.distortion_type = dist_type;
    }

    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(0.0, 1.0);
    }

    pub fn set_tone(&mut self, tone: f32) {
        self.tone = tone.clamp(0.0, 1.0);
    }

    pub fn set_level(&mut self, level: f32) {
        self.level = level.clamp(0.0, 1.0);
    }

    // High-pass filter for input (removes mud, classic analog behavior)
    fn input_highpass(&mut self, input: f32) -> f32 {
        let cutoff = 100.0; // Hz - removes sub-bass
        let rc = 1.0 / (2.0 * PI * cutoff);
        let alpha = rc / (rc + 1.0 / self.sample_rate);
        
        let output = alpha * (self.input_hp_y1 + input - self.input_hp_x1);
        self.input_hp_x1 = input;
        self.input_hp_y1 = output;
        
        output
    }

    // Tube Screamer style overdrive - soft asymmetric clipping
    fn overdrive_processing(&self, input: f32, drive_amount: f32) -> f32 {
        let gain = 1.0 + drive_amount * 20.0; // 1x to 21x gain
        let driven = input * gain;
        
        // Asymmetric soft clipping (Tube Screamer characteristic)
        if driven > 0.0 {
            driven / (1.0 + driven.abs()) // Positive clipping is softer
        } else {
            driven / (1.0 + driven.abs() * 1.5) // Negative clipping is harder
        }
    }

    // Hard distortion - symmetric hard clipping with pre-emphasis
    fn distortion_processing(&self, input: f32, drive_amount: f32) -> f32 {
        let gain = 1.0 + drive_amount * 50.0; // Much more aggressive
        let driven = input * gain;
        
        // Hard symmetric clipping
        driven.clamp(-0.7, 0.7)
    }

    // Big Muff style fuzz - heavily asymmetric with compression
    fn fuzz_processing(&self, input: f32, drive_amount: f32) -> f32 {
        let gain = 1.0 + drive_amount * 100.0; // Extreme gain
        let driven = input * gain;
        
        // Very asymmetric clipping with compression
        if driven > 0.0 {
            let compressed = driven / (1.0 + driven * 2.0);
            compressed.min(0.8)
        } else {
            let compressed = driven / (1.0 + driven.abs() * 0.5);
            compressed.max(-1.0)
        }
    }

    // Tube saturation modeling - smooth saturation curve
    fn tube_processing(&self, input: f32, drive_amount: f32) -> f32 {
        let gain = 1.0 + drive_amount * 8.0; // Moderate gain
        let driven = input * gain;
        
        // Tube-like saturation using tanh
        (driven * 1.5).tanh() * 0.7
    }

    // Tone control - classic Tube Screamer style mid boost
    fn tone_control(&mut self, input: f32) -> f32 {
        // High-pass component (emphasizes highs when tone is high)
        let hp_cutoff = 500.0 + self.tone * 2000.0; // 500Hz to 2.5kHz
        let hp_rc = 1.0 / (2.0 * PI * hp_cutoff);
        let hp_alpha = hp_rc / (hp_rc + 1.0 / self.sample_rate);
        
        let hp_out = hp_alpha * (self.tone_hp_y1 + input - self.tone_hp_x1);
        self.tone_hp_x1 = input;
        self.tone_hp_y1 = hp_out;
        
        // Low-pass component (reduces harshness when tone is low)
        let lp_cutoff = 8000.0 - self.tone * 4000.0; // 4kHz to 8kHz
        let lp_rc = 1.0 / (2.0 * PI * lp_cutoff);
        let lp_alpha = 1.0 / self.sample_rate / (lp_rc + 1.0 / self.sample_rate);
        
        let lp_out = lp_alpha * input + (1.0 - lp_alpha) * self.tone_lp_y1;
        self.tone_lp_x1 = input;
        self.tone_lp_y1 = lp_out;
        
        // Mix HP and LP based on tone control
        let hp_mix = self.tone;
        let lp_mix = 1.0 - self.tone;
        
        hp_out * hp_mix + lp_out * lp_mix
    }

    // Output low-pass filter (removes harsh digital artifacts)
    fn output_lowpass(&mut self, input: f32) -> f32 {
        let cutoff = 12000.0; // Hz - removes digital harshness
        let rc = 1.0 / (2.0 * PI * cutoff);
        let alpha = 1.0 / self.sample_rate / (rc + 1.0 / self.sample_rate);
        
        let output = alpha * input + (1.0 - alpha) * self.output_lp_y1;
        self.output_lp_x1 = input;
        self.output_lp_y1 = output;
        
        output
    }

    pub fn process(&mut self, input: f32) -> f32 {
        if matches!(self.distortion_type, DistortionType::Clean) {
            return input;
        }

        // Input conditioning (classic analog pedal behavior)
        let conditioned = self.input_highpass(input);
        
        // Apply distortion based on type
        let distorted = match self.distortion_type {
            DistortionType::Clean => conditioned,
            DistortionType::Overdrive => self.overdrive_processing(conditioned, self.drive),
            DistortionType::Distortion => self.distortion_processing(conditioned, self.drive),
            DistortionType::Fuzz => self.fuzz_processing(conditioned, self.drive),
            DistortionType::Tube => self.tube_processing(conditioned, self.drive),
        };
        
        // Tone control (classic mid-frequency emphasis)
        let toned = self.tone_control(distorted);
        
        // Output conditioning
        let filtered = self.output_lowpass(toned);
        
        // Level compensation with slight makeup gain
        filtered * self.level * 2.0
    }
}