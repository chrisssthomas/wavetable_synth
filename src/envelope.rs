pub struct ADSR {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    start_time: f32,
    end_time: f32,
}

impl ADSR {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
            start_time: 0.0,
            end_time: 0.0,
        }
    }

    fn start(&mut self, start_time: f32) {
        self.start_time = start_time;
    }

    fn stop(&mut self, end_time: f32) {
        self.end_time = end_time;
    }

    fn value(&self, time: f32) -> f32 {
        if time < self.start_time {
            0.0
        } else if time < self.start_time + self.attack {
            (time - self.start_time) / self.attack
        } else if time < self.start_time + self.attack + self.decay {
            1.0 + (self.sustain - 1.0) * (time - self.start_time - self.attack) / self.decay
        } else if time < self.end_time {
            self.sustain
        } else if time < self.end_time + self.release {
            self.sustain * (1.0 - (time - self.end_time) / self.release)
        } else {
            0.0
        }
    }

    pub fn clone(&self) -> ADSR {
        return ADSR {
            attack: self.attack,
            decay: self.decay,
            sustain: self.sustain,
            release: self.release,
            start_time: self.start_time,
            end_time: self.end_time,
        };
    }
}
