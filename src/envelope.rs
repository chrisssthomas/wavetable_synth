#[derive(Clone)]
pub struct ADSR {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    start_time: f32,
    release_time: Option<f32>, // Use Option to track if release has been triggered
    current_value: f32, // Track current envelope value for smoother transitions
}

impl ADSR {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
            start_time: 0.0,
            release_time: None,
            current_value: 0.0,
        }
    }

    pub fn start(&mut self, start_time: f32) {
        self.start_time = start_time;
        self.release_time = None; // Reset release
        self.current_value = 0.0;
    }

    pub fn stop(&mut self, current_time: f32) {
        if self.release_time.is_none() {
            // Store the release time relative to start_time
            self.release_time = Some(current_time);
            // Calculate current envelope value WITHOUT calling self.value() to avoid recursion
            let elapsed = current_time;
            
            // Calculate what the envelope value should be at this moment
            if elapsed <= self.attack {
                self.current_value = elapsed / self.attack;
            } else if elapsed <= self.attack + self.decay {
                let decay_progress = (elapsed - self.attack) / self.decay;
                self.current_value = 1.0 - (1.0 - self.sustain) * decay_progress;
            } else {
                self.current_value = self.sustain;
            }
        }
    }

    pub fn value(&self, current_time: f32) -> f32 {
        if current_time < 0.0 {
            return 0.0;
        }

        // Use current_time directly since it's already relative time from voice start
        let elapsed = current_time;
        
        // Check if we're in release phase
        if let Some(release_start) = self.release_time {
            if elapsed >= release_start {
                let release_elapsed = elapsed - release_start;
                if release_elapsed >= self.release {
                    return 0.0;
                } else {
                    // Release from current value (smooth transition)
                    return self.current_value * (1.0 - release_elapsed / self.release);
                }
            }
        }
        
        // Attack phase
        if elapsed <= self.attack {
            return elapsed / self.attack;
        }
        
        // Decay phase  
        if elapsed <= self.attack + self.decay {
            let decay_progress = (elapsed - self.attack) / self.decay;
            return 1.0 - (1.0 - self.sustain) * decay_progress;
        }
        
        // Sustain phase (hold at sustain level)
        self.sustain
    }
    
    // Real-time parameter updates for synthesis programming
    pub fn update_attack(&mut self, attack: f32) {
        self.attack = attack;
    }
    
    pub fn update_decay(&mut self, decay: f32) {
        self.decay = decay;
    }
    
    pub fn update_sustain(&mut self, sustain: f32) {
        self.sustain = sustain;
    }
    
    pub fn update_release(&mut self, release: f32) {
        self.release = release;
    }
}
