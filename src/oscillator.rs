use std::sync::atomic::{AtomicU32, Ordering};

   pub struct Oscillator {
       phase: f32,
       frequency: AtomicU32,
       sample_rate: f32,
   }

   impl Oscillator {
       pub fn new(sample_rate: f32, frequency: f32) -> Self {
           Self {
               phase: 0.0,
               frequency: AtomicU32::new(frequency.to_bits()),
               sample_rate,
           }
       }

       pub fn next_sample(&mut self) -> f32 {
           let frequency = f32::from_bits(self.frequency.load(Ordering::Relaxed));
           self.phase = (self.phase + frequency / self.sample_rate) % 1.0;
           2.0 * self.phase - 1.0  // Sawtooth wave
       }

       pub fn set_frequency(&self, frequency: f32) {
           self.frequency.store(frequency.to_bits(), Ordering::Relaxed);
       }

       pub fn note_to_frequency(note: u8) -> f32 {
           440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
       }
   }