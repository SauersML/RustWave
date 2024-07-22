use crate::voice::Voice;
use crate::reverb::Reverb;
use crate::chorus::{Chorus, ChorusMode};

pub struct VoiceManager {
    pub voices: Vec<Voice>,
    reverb: Reverb,
    chorus: Chorus,
    max_voices: usize,
}

impl VoiceManager {
    pub fn new(sample_rate: f32, num_voices: usize) -> Self {
        Self {
            voices: (0..num_voices).map(|_| Voice::new(sample_rate)).collect(),
            reverb: Reverb::new(sample_rate),
            chorus: Chorus::new(sample_rate),
            max_voices: num_voices,
        }
    }

    pub fn note_on(&mut self, note: u8) {
        self.note_off(note);

        if let Some(inactive_voice) = self.voices.iter_mut().find(|v| !v.is_active()) {
            inactive_voice.trigger(note);
        } else if let Some(oldest_voice) = self.find_oldest_voice() {
            oldest_voice.trigger(note);
        }
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in self.voices.iter_mut() {
            if voice.note == Some(note) {
                voice.release();
            }
        }
    }

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
        let mut left_output = 0.0;
        let mut right_output = 0.0;
    
        let mut active_voices = 0;
        for voice in &mut self.voices {
            if voice.is_active() {
                let voice_output = voice.render_next();
                left_output += voice_output;
                right_output += voice_output;
                active_voices += 1;
            }
        }
    
        if active_voices > 0 {
            let normalization_factor = 1.0 / (active_voices as f32).sqrt();
            left_output *= normalization_factor;
            right_output *= normalization_factor;
        }
    
        // Apply reverb
        let (reverb_left, reverb_right) = self.reverb.process(left_output, right_output);

        // Mix dry and reverb signals
        let wet_amount = self.reverb.get_wet();
        let left = left_output * (1.0 - wet_amount) + reverb_left * wet_amount;
        let right = right_output * (1.0 - wet_amount) + reverb_right * wet_amount;

        // Apply chorus to the reverb output
        let (chorus_left, chorus_right) = self.chorus.process(left, right);

        // Mix reverb and chorus
        let chorus_mix = 0.8;
        let left = left * (1.0 - chorus_mix) + chorus_left * chorus_mix;
        let right = right * (1.0 - chorus_mix) + chorus_right * chorus_mix;

        (left, right)
    }





    pub fn set_reverb_decay(&mut self, decay: f32) {
        self.reverb.set_decay(decay.clamp(0.0, 0.99));
    }

    pub fn set_reverb_wet(&mut self, wet: f32) {
        self.reverb.set_wet(wet.clamp(0.0, 1.0));
    }

    pub fn set_chorus_mode(&mut self, mode: ChorusMode) {
        self.chorus.set_mode(mode);
    }

    pub fn set_chorus_rate(&mut self, rate: f32) {
        self.chorus.set_rate(rate);
    }

    pub fn set_chorus_depth(&mut self, depth: f32) {
        self.chorus.set_depth(depth);
    }
}