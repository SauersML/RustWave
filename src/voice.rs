use crate::oscillator::Oscillator;
use crate::envelope::Envelope;
use crate::filter::Filter;

pub struct Voice {
    oscillator: Oscillator,
    envelope: Envelope,
    filter: Filter,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            oscillator: Oscillator::new(),
            envelope: Envelope::new(),
            filter: Filter::new(),
        }
    }

    // TODO: Implement voice processing
}
