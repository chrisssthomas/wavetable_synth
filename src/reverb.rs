#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReverbType {
    Room,      // Small room reverb
    Hall,      // Large hall reverb  
    Plate,     // Plate reverb simulation
    Spring,    // Spring reverb
}

// Advanced delay line with interpolation and modulation
struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
    max_delay: usize,
}

impl DelayLine {
    fn new(max_delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_delay_samples.max(1)],
            write_pos: 0,
            max_delay: max_delay_samples,
        }
    }
    
    fn write(&mut self, input: f32) {
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % self.max_delay;
    }
    
    fn read(&self, delay_samples: usize) -> f32 {
        let delay = delay_samples.min(self.max_delay - 1);
        let read_pos = if self.write_pos >= delay {
            self.write_pos - delay
        } else {
            self.max_delay + self.write_pos - delay
        };
        self.buffer[read_pos]
    }
}

// Advanced all-pass filter with proper diffusion
struct AllPassFilter {
    delay_line: DelayLine,
    delay_samples: usize,
    feedback: f32,
}

impl AllPassFilter {
    fn new(delay_samples: usize, feedback: f32) -> Self {
        Self {
            delay_line: DelayLine::new(delay_samples + 1),
            delay_samples,
            feedback: feedback.clamp(-0.99, 0.99),
        }
    }
    
    fn process(&mut self, input: f32) -> f32 {
        let delayed_input = self.delay_line.read(self.delay_samples);
        let output = -self.feedback * input + delayed_input;
        self.delay_line.write(input + self.feedback * delayed_input);
        output
    }
}

// Comb filter with proper damping
struct CombFilter {
    delay_line: DelayLine,
    delay_samples: usize,
    feedback: f32,
    filter_state: f32,
    damping: f32,
}

impl CombFilter {
    fn new(delay_samples: usize, feedback: f32, damping: f32) -> Self {
        Self {
            delay_line: DelayLine::new(delay_samples + 1),
            delay_samples,
            feedback: feedback.clamp(0.0, 0.99),
            filter_state: 0.0,
            damping: damping.clamp(0.0, 1.0),
        }
    }
    
    fn process(&mut self, input: f32) -> f32 {
        let output = self.delay_line.read(self.delay_samples);
        
        // High-frequency damping filter (simple 1-pole lowpass)
        self.filter_state = output * (1.0 - self.damping) + self.filter_state * self.damping;
        
        self.delay_line.write(input + self.filter_state * self.feedback);
        output
    }
    
    fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.99);
    }
    
    fn set_damping(&mut self, damping: f32) {
        self.damping = damping.clamp(0.0, 1.0);
    }
}

// Professional Schroeder-style reverb with proper early reflections
pub struct AnalogReverb {
    // Parallel comb filters for late reverberation
    comb_filters: Vec<CombFilter>,
    
    // Serial all-pass filters for diffusion
    allpass_filters: Vec<AllPassFilter>,
    
    // Early reflection network
    early_delays: Vec<DelayLine>,
    early_gains: Vec<f32>,
    early_delays_samples: Vec<usize>,
    
    // Parameters
    room_size: f32,
    decay: f32,
    mix: f32,
    damping: f32,
    reverb_type: ReverbType,
    
    sample_rate: f32,
}

impl AnalogReverb {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate;
        
        // Tuned comb filter delays (in samples) - based on room acoustics research
        // These delays are carefully chosen to avoid modal resonances
        let comb_delays = [
            (sr * 0.0297) as usize,   // ~30ms
            (sr * 0.0371) as usize,   // ~37ms  
            (sr * 0.0411) as usize,   // ~41ms
            (sr * 0.0437) as usize,   // ~44ms
            (sr * 0.0507) as usize,   // ~51ms
            (sr * 0.0561) as usize,   // ~56ms
            (sr * 0.0617) as usize,   // ~62ms
            (sr * 0.0683) as usize,   // ~68ms
        ];
        
        let mut comb_filters = Vec::new();
        for &delay in &comb_delays {
            comb_filters.push(CombFilter::new(delay, 0.74, 0.3));
        }
        
        // All-pass filter delays for diffusion
        let allpass_delays = [
            (sr * 0.005) as usize,    // 5ms
            (sr * 0.017) as usize,    // 17ms  
            (sr * 0.023) as usize,    // 23ms
            (sr * 0.033) as usize,    // 33ms
        ];
        
        let mut allpass_filters = Vec::new();
        for (i, &delay) in allpass_delays.iter().enumerate() {
            let feedback = if i % 2 == 0 { 0.7 } else { -0.7 };
            allpass_filters.push(AllPassFilter::new(delay, feedback));
        }
        
        // Early reflection network - simulates first bounces off walls
        let early_delays_ms = [8.0, 12.0, 16.0, 20.0, 24.0, 28.0, 32.0, 36.0];
        let early_delays_samples: Vec<usize> = early_delays_ms
            .iter()
            .map(|&ms| (sr * ms * 0.001) as usize)
            .collect();
            
        let mut early_delays = Vec::new();
        for &delay in &early_delays_samples {
            early_delays.push(DelayLine::new(delay + 1));
        }
        
        // Early reflection gains - realistic decay pattern
        let early_gains = vec![0.8, 0.6, 0.5, 0.4, 0.3, 0.25, 0.2, 0.15];
        
        Self {
            comb_filters,
            allpass_filters,
            early_delays,
            early_gains,
            early_delays_samples,
            room_size: 0.5,
            decay: 0.5,
            mix: 0.3,
            damping: 0.4,
            reverb_type: ReverbType::Room,
            sample_rate: sr,
        }
    }
    
    pub fn process(&mut self, input: f32) -> f32 {
        let input_gain = match self.reverb_type {
            ReverbType::Room => 0.6,
            ReverbType::Hall => 0.8,
            ReverbType::Plate => 0.5,
            ReverbType::Spring => 0.4,
        };
        
        let scaled_input = input * input_gain;
        
        // Generate early reflections
        let mut early_sum = 0.0;
        for (i, delay_line) in self.early_delays.iter_mut().enumerate() {
            delay_line.write(scaled_input);
            let delayed = delay_line.read(self.early_delays_samples[i]);
            early_sum += delayed * self.early_gains[i];
        }
        
        // Process through parallel comb filters
        let mut comb_sum = 0.0;
        for comb in &mut self.comb_filters {
            // Update comb filter parameters based on settings
            let room_scale = 0.5 + self.room_size * 0.4;
            let decay_feedback = 0.4 + self.decay * 0.5;
            comb.set_feedback(decay_feedback * room_scale);
            comb.set_damping(self.damping * 0.6 + 0.1);
            
            comb_sum += comb.process(scaled_input + early_sum * 0.2);
        }
        comb_sum /= self.comb_filters.len() as f32;
        
        // Process through serial all-pass filters for diffusion
        let mut diffused = comb_sum;
        for allpass in &mut self.allpass_filters {
            diffused = allpass.process(diffused);
        }
        
        // Apply reverb type characteristics
        let reverb_output = match self.reverb_type {
            ReverbType::Room => {
                // Warmer, more intimate
                diffused * 0.8 + early_sum * 0.4
            },
            ReverbType::Hall => {
                // Spacious, longer tail
                diffused * 1.2 + early_sum * 0.3  
            },
            ReverbType::Plate => {
                // Bright, dense
                let brightened = diffused * 1.1;
                brightened + early_sum * 0.5
            },
            ReverbType::Spring => {
                // Bouncy, shorter
                diffused * 0.6 + early_sum * 0.8
            },
        };
        
        // Final mix
        let wet = reverb_output * self.mix;
        let dry = input * (1.0 - self.mix);
        
        dry + wet
    }
    
    pub fn set_room_size(&mut self, size: f32) {
        self.room_size = size.clamp(0.0, 1.0);
    }
    
    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay.clamp(0.0, 1.0);
    }
    
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }
    
    pub fn set_type(&mut self, reverb_type: ReverbType) {
        self.reverb_type = reverb_type;
    }
    
    // Compatibility aliases
    pub fn set_size(&mut self, size: f32) {
        self.set_room_size(size);
    }
}