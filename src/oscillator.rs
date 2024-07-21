use std::sync::atomic::{AtomicU32, Ordering};
use crate::envelope::Envelope;

#[derive(Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

pub struct Oscillator {
    phase: f32,
    frequency: AtomicU32,
    sample_rate: f32,
    volume: AtomicU32,
    waveform: Waveform,
    envelope: Envelope,
}

impl Oscillator {
    pub fn new(sample_rate: f32, frequency: f32) -> Self {
        Self {
            phase: 0.0,
            frequency: AtomicU32::new(frequency.to_bits()),
            sample_rate,
            volume: AtomicU32::new(1.0f32.to_bits()),
            waveform: Waveform::Sawtooth,
            envelope: Envelope::new(sample_rate), // This is correct
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let frequency = f32::from_bits(self.frequency.load(Ordering::Relaxed));
        let volume = f32::from_bits(self.volume.load(Ordering::Relaxed));
        self.phase = (self.phase + frequency / self.sample_rate) % 1.0;

        let raw_sample = match self.waveform {
            Waveform::Sine => (self.phase * 2.0 * std::f32::consts::PI).sin(),
            Waveform::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },
            Waveform::Sawtooth => 2.0 * self.phase - 1.0,
            Waveform::Triangle => 1.0 - 4.0 * (self.phase - 0.25).abs(),
        };
        raw_sample * volume * self.envelope.next_sample()
    }

    pub fn set_frequency(&self, frequency: f32) {
        self.frequency.store(frequency.to_bits(), Ordering::Relaxed);
    }

    pub fn set_volume(&self, volume: f32) {
        self.volume.store(volume.to_bits(), Ordering::Relaxed);
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn note_to_frequency(note: u8) -> f32 {
        220.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
    }

    pub fn note_on(&mut self) {
        self.envelope.note_on();
    }

    pub fn note_off(&mut self) {
        self.envelope.note_off();
    }

    pub fn set_attack(&self, attack: f32) {
        self.envelope.set_attack(attack);
    }

    pub fn set_decay(&self, decay: f32) {
        self.envelope.set_decay(decay);
    }

    pub fn set_sustain(&self, sustain: f32) {
        self.envelope.set_sustain(sustain);
    }

    pub fn set_release(&self, release: f32) {
        self.envelope.set_release(release);
    }
}