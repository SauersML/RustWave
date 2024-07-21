use crate::oscillator::Oscillator;
use crate::envelope::Envelope;
use crate::filter::Filter;

pub struct Voice {
    pub oscillator: Oscillator,
    pub envelope: Envelope,
    pub filter: Filter,
    pub note: Option<u8>,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            oscillator: Oscillator::new(sample_rate, 440.0),
            envelope: Envelope::new(sample_rate),
            filter: Filter::new(),
            note: None,
        }
    }

    pub fn trigger(&mut self, note: u8) {
        let frequency = Oscillator::note_to_frequency(note);
        self.oscillator.set_frequency(frequency);
        self.envelope.note_on();
        self.note = Some(note);
    }

    pub fn release(&mut self) {
        self.envelope.note_off();
        self.note = None;
    }

    pub fn is_active(&self) -> bool {
        self.note.is_some() || !self.envelope.is_idle()
    }

    pub fn render_next(&mut self) -> f32 {
        let osc_sample = self.oscillator.next_sample();
        let env_sample = self.envelope.next_sample();
        self.filter.process(osc_sample * env_sample)
    }
}