use crate::voice::Voice;

pub struct VoiceManager {
    voices: Vec<Voice>,
}

impl VoiceManager {
    pub fn new() -> Self {
        Self { voices: Vec::new() }
    }

    // TODO: Implement voice allocation and management
}
