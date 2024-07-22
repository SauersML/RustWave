// src/reverb.rs

use std::sync::atomic::{AtomicU32, Ordering};

pub struct Reverb {
    buffer: Vec<f32>,
    index: usize,
    size: usize,
    decay: AtomicU32,
}

impl Reverb {
    pub fn new(sample_rate: f32, max_delay_ms: f32) -> Self {
        let size = (sample_rate * max_delay_ms / 1000.0) as usize;
        Self {
            buffer: vec![0.0; size],
            index: 0,
            size,
            decay: AtomicU32::new(0.5f32.to_bits()),
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let decay = f32::from_bits(self.decay.load(Ordering::Relaxed));
        
        let output = self.buffer[self.index];
        self.buffer[self.index] = input + output * decay;
        self.index = (self.index + 1) % self.size;
        
        input + output * 0.5 // Mix dry and wet signals
    }

    pub fn set_decay(&self, decay: f32) {
        self.decay.store(decay.to_bits(), Ordering::Relaxed);
    }
}