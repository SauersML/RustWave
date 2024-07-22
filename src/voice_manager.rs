use crate::voice::Voice;
use crate::reverb::Reverb;
use crate::chorus::{Chorus, ChorusMode};

pub struct VoiceManager {
    pub voices: Vec<Voice>,
    reverb: Reverb,
    chorus: Chorus,
}

impl VoiceManager {
    pub fn new(sample_rate: f32, num_voices: usize) -> Self {
        Self {
            voices: (0..num_voices).map(|_| Voice::new(sample_rate)).collect(),
            reverb: Reverb::new(sample_rate, 100.0), // 100ms max delay
            chorus: Chorus::new(sample_rate),
        }
    }

    pub fn note_on(&mut self, note: u8) {
        // First, release any existing voices for this note
        self.note_off(note);

        // Find an inactive voice
        if let Some(inactive_voice) = self.voices.iter_mut().find(|v| !v.is_active()) {
            inactive_voice.trigger(note);
        } else {
            // If no inactive voice, find the oldest one
            if let Some(oldest_voice) = self.find_oldest_voice() {
                oldest_voice.trigger(note);
            }
        }
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in self.voices.iter_mut() {
            if voice.note == Some(note) {
                voice.release();
            }
        }
    }

    // Helper method to find the oldest voice
    fn find_oldest_voice(&mut self) -> Option<&mut Voice> {
        self.voices.iter_mut().min_by_key(|v| v.note)
    }

    pub fn set_filter_cutoff(&mut self, cutoff: f32) {
        for voice in &mut self.voices {
            voice.set_filter_cutoff(cutoff);
        }
    }

    pub fn set_filter_resonance(&mut self, resonance: f32) {
        for voice in &mut self.voices {
            voice.set_filter_resonance(resonance);
        }
    }

    pub fn set_filter_drive(&mut self, drive: f32) {
        for voice in &mut self.voices {
            voice.filter.set_drive(drive);
        }
    }

    pub fn set_filter_saturation(&mut self, saturation: f32) {
        for voice in &mut self.voices {
            voice.filter.set_saturation(saturation);
        }
    }

    pub fn render_next(&mut self) -> (f32, f32) {
        let voice_output = self.voices.iter_mut().map(|v| v.render_next()).sum::<f32>() / self.voices.len() as f32;
        let reverb_output = self.reverb.process(voice_output);
        self.chorus.process(reverb_output)
    }

    pub fn set_chorus_mode(&mut self, mode: ChorusMode) {
        self.chorus.set_mode(mode);
    }

    pub fn set_reverb_decay(&mut self, decay: f32) {
        self.reverb.set_decay(decay);
    }

    pub fn set_chorus_rate(&mut self, rate: f32) {
        self.chorus.set_rate(rate);
    }

    pub fn set_chorus_depth(&mut self, depth: f32) {
        self.chorus.set_depth(depth);
    }
}