use std::f32::consts::PI;

// MS-20 style analog filter simulation
pub struct MS20Filter {
    sample_rate: f32,
    cutoff_frequency: f32,
    resonance: f32,
    
    // MS-20 filter state variables (4-pole ladder design)
    stage1: f32,
    stage2: f32,
    stage3: f32,
    stage4: f32,
    
    // Feedback and drive for analog character
    feedback: f32,
    drive: f32,
    
    // Parameter smoothing to prevent clicks
    target_cutoff: f32,
    target_resonance: f32,
    cutoff_smooth: f32,
    resonance_smooth: f32,
    
    // Non-linear processing for analog warmth
    saturation: f32,
}

impl MS20Filter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            cutoff_frequency: 1000.0,
            resonance: 0.0,
            stage1: 0.0,
            stage2: 0.0,
            stage3: 0.0,
            stage4: 0.0,
            feedback: 0.0,
            drive: 1.0,
            target_cutoff: 1000.0,
            target_resonance: 0.0,
            cutoff_smooth: 0.9995,    // Very smooth cutoff changes
            resonance_smooth: 0.999,  // Smooth resonance changes
            saturation: 0.1,          // Subtle analog saturation
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        // Clamp cutoff to musical range
        self.target_cutoff = cutoff.clamp(20.0, self.sample_rate * 0.45);
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        // MS-20 style resonance: 0.0 to 0.95 (just under self-oscillation)
        self.target_resonance = resonance.clamp(0.0, 0.95);
    }

    // Analog-style soft saturation (tanh approximation)
    fn soft_saturate(&self, input: f32) -> f32 {
        let x = input * self.saturation;
        // Fast tanh approximation for analog warmth
        if x.abs() < 1.0 {
            x * (1.0 - x * x / 3.0)
        } else {
            x.signum() * (1.0 - 1.0 / (1.0 + x.abs()))
        }
    }

    // MS-20 characteristic one-pole filter stage (static method to avoid borrowing issues)
    fn one_pole_stage(input: f32, state: &mut f32, cutoff_factor: f32) -> f32 {
        // One-pole low-pass with analog-style behavior
        let g = cutoff_factor / (1.0 + cutoff_factor);
        let output = *state + g * (input - *state);
        *state = output + (input - output) * 0.01; // Add slight non-linearity
        output
    }

    pub fn process(&mut self, input: f32) -> f32 {
        // Smooth parameter changes
        self.cutoff_frequency = self.cutoff_frequency * self.cutoff_smooth + 
                               self.target_cutoff * (1.0 - self.cutoff_smooth);
        self.resonance = self.resonance * self.resonance_smooth + 
                        self.target_resonance * (1.0 - self.resonance_smooth);

        // Calculate filter coefficients (MS-20 style)
        let omega = 2.0 * PI * self.cutoff_frequency / self.sample_rate;
        let cutoff_factor = omega / (1.0 + omega); // Pre-warping for analog feel

        // MS-20 resonance calculation - more musical than linear
        let reso_factor = self.resonance * self.resonance; // Quadratic resonance response
        let feedback_amount = reso_factor * 3.8; // Just under self-oscillation

        // Apply input drive for analog character
        let driven_input = self.soft_saturate(input * self.drive);

        // Subtract feedback for resonance (MS-20 style)
        let filtered_input = driven_input - self.feedback * feedback_amount;

        // 4-stage ladder filter (simplified MS-20 topology)
        let stage1_out = Self::one_pole_stage(filtered_input, &mut self.stage1, cutoff_factor);
        let stage2_out = Self::one_pole_stage(stage1_out, &mut self.stage2, cutoff_factor * 0.95);
        let stage3_out = Self::one_pole_stage(stage2_out, &mut self.stage3, cutoff_factor * 0.9);
        let stage4_out = Self::one_pole_stage(stage3_out, &mut self.stage4, cutoff_factor * 0.85);

        // MS-20 feedback comes from the 4th stage, with soft saturation
        self.feedback = self.soft_saturate(stage4_out);

        // Mix stages for MS-20 characteristic (emphasize 2nd and 4th stages)
        let output = stage2_out * 0.3 + stage4_out * 0.7;

        // MS-20 style gain compensation: extreme boost for professional output levels
        let gain_compensation = 5.0 + (feedback_amount * 1.0); // Very high base gain
        
        // Final output with proper gain staging
        self.soft_saturate(output * gain_compensation)
    }
}

// Type alias for compatibility with existing code
pub type LowPassFilter = MS20Filter;