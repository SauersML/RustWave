// In src/envelope.rs

use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy, PartialEq)]
pub enum EnvelopeStage {
    Attack,
    Decay,
    Sustain,
    Release,
    Idle,
}

pub struct Envelope {
    attack: AtomicU32,
    decay: AtomicU32,
    sustain: AtomicU32,
    release: AtomicU32,
    stage: EnvelopeStage,
    current_level: f32,
    sample_rate: f32,
    time_in_stage: f32,
}

impl Envelope {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            attack: AtomicU32::new(0.1f32.to_bits()),
            decay: AtomicU32::new(0.1f32.to_bits()),
            sustain: AtomicU32::new(0.7f32.to_bits()),
            release: AtomicU32::new(0.2f32.to_bits()),
            stage: EnvelopeStage::Idle,
            current_level: 0.0,
            sample_rate,
            time_in_stage: 0.0,
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        match self.stage {
            EnvelopeStage::Attack => {
                let attack_time = f32::from_bits(self.attack.load(Ordering::Relaxed));
                self.current_level += 1.0 / (attack_time * self.sample_rate);
                if self.current_level >= 1.0 {
                    self.current_level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                    self.time_in_stage = 0.0;
                }
            }
            EnvelopeStage::Decay => {
                let decay_time = f32::from_bits(self.decay.load(Ordering::Relaxed));
                let sustain_level = f32::from_bits(self.sustain.load(Ordering::Relaxed));
                self.current_level -= (1.0 - sustain_level) / (decay_time * self.sample_rate);
                if self.current_level <= sustain_level {
                    self.current_level = sustain_level;
                    self.stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => {
                // Do nothing, maintain the sustain level
            }
            EnvelopeStage::Release => {
                let release_time = f32::from_bits(self.release.load(Ordering::Relaxed));
                self.current_level -= self.current_level / (release_time * self.sample_rate);
                if self.current_level < 0.001 {
                    self.current_level = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
            EnvelopeStage::Idle => {
                self.current_level = 0.0;
            }
        }
        self.time_in_stage += 1.0 / self.sample_rate;
        self.current_level
    }

    pub fn note_on(&mut self) {
        self.stage = EnvelopeStage::Attack;
        self.time_in_stage = 0.0;
    }

    pub fn note_off(&mut self) {
        self.stage = EnvelopeStage::Release;
        self.time_in_stage = 0.0;
    }

    pub fn set_attack(&self, attack: f32) {
        self.attack.store(attack.to_bits(), Ordering::Relaxed);
    }

    pub fn set_decay(&self, decay: f32) {
        self.decay.store(decay.to_bits(), Ordering::Relaxed);
    }

    pub fn set_sustain(&self, sustain: f32) {
        self.sustain.store(sustain.to_bits(), Ordering::Relaxed);
    }

    pub fn set_release(&self, release: f32) {
        self.release.store(release.to_bits(), Ordering::Relaxed);
    }
}