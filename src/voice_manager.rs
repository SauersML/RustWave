use crate::voice::Voice;

pub struct VoiceManager {
    pub voices: Vec<Voice>,
}

impl VoiceManager {
    pub fn new(sample_rate: f32, num_voices: usize) -> Self {
        Self {
            voices: (0..num_voices).map(|_| Voice::new(sample_rate)).collect(),
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

    pub fn render_next(&mut self) -> f32 {
        self.voices.iter_mut().map(|v| v.render_next()).sum::<f32>() / self.voices.len() as f32
    }

    // Helper method to find the oldest voice
    fn find_oldest_voice(&mut self) -> Option<&mut Voice> {
        self.voices.iter_mut().min_by_key(|v| v.note)
    }
}