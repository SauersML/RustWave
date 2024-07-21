use std::f32::consts::PI;

pub struct AudioEngine {
    sample_rate: f32,
    phase: f32,
    frequency: f32,
}

impl AudioEngine {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            frequency: 440.0,
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let value = (self.phase * 2.0 * PI).sin();
        self.phase = (self.phase + self.frequency / self.sample_rate) % 1.0;
        value
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}