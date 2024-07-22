use std::f32::consts::PI;

pub struct Reverb {
    left_comb_filters: Vec<CombFilter>,
    right_comb_filters: Vec<CombFilter>,
    left_allpass_filters: Vec<AllpassFilter>,
    right_allpass_filters: Vec<AllpassFilter>,
    wet: f32,
    dry: f32,
}

struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
    damp1: f32,
    damp2: f32,
    filtered: f32,
}

struct AllpassFilter {
    buffer: Vec<f32>,
    index: usize,
    gain: f32,
}

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        let room_size = 0.8;
        let damping = 0.5;
        let wet = 0.33;
        let dry = 1.0 - wet;

        let comb_tunings = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
        let stereo_spread = 23;
        let allpass_tunings = [556, 441, 341, 225];

        let left_comb_filters = comb_tunings
            .iter()
            .map(|&tuning| CombFilter::new(tuning, room_size, damping, sample_rate))
            .collect();

        let right_comb_filters = comb_tunings
            .iter()
            .map(|&tuning| CombFilter::new(tuning + stereo_spread, room_size, damping, sample_rate))
            .collect();

        let left_allpass_filters = allpass_tunings
            .iter()
            .map(|&tuning| AllpassFilter::new(tuning, 0.5, sample_rate))
            .collect();

        let right_allpass_filters = allpass_tunings
            .iter()
            .map(|&tuning| AllpassFilter::new(tuning + stereo_spread, 0.5, sample_rate))
            .collect();

        Self {
            left_comb_filters,
            right_comb_filters,
            left_allpass_filters,
            right_allpass_filters,
            wet,
            dry,
        }
    }

    pub fn process(&mut self, input_left: f32, input_right: f32) -> (f32, f32) {
        let mut output_left = 0.0;
        let mut output_right = 0.0;

        // Process through comb filters in parallel
        for comb in &mut self.left_comb_filters {
            output_left += comb.process(input_left);
        }
        for comb in &mut self.right_comb_filters {
            output_right += comb.process(input_right);
        }

        // Normalize the output of comb filters
        output_left /= self.left_comb_filters.len() as f32;
        output_right /= self.right_comb_filters.len() as f32;

        // Process through allpass filters in series
        for allpass in &mut self.left_allpass_filters {
            output_left = allpass.process(output_left);
        }
        for allpass in &mut self.right_allpass_filters {
            output_right = allpass.process(output_right);
        }

        // Mix dry and wet signals
        let left = input_left * self.dry + output_left * self.wet;
        let right = input_right * self.dry + output_right * self.wet;

        (left, right)
    }

    pub fn set_decay(&mut self, decay: f32) {
        let decay = decay.clamp(0.0, 0.99);
        for comb in self.left_comb_filters.iter_mut().chain(self.right_comb_filters.iter_mut()) {
            comb.set_feedback(decay);
        }
    }

    pub fn set_damping(&mut self, damping: f32) {
        let damping = damping.clamp(0.0, 1.0);
        for comb in self.left_comb_filters.iter_mut().chain(self.right_comb_filters.iter_mut()) {
            comb.set_damping(damping);
        }
    }

    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet.clamp(0.0, 1.0);
        self.dry = 1.0 - self.wet;
    }

    pub fn get_wet(&self) -> f32 {
        self.wet
    }
}

impl CombFilter {
    fn new(delay: usize, room_size: f32, damping: f32, sample_rate: f32) -> Self {
        let buffer = vec![0.0; delay];
        let feedback = room_size;
        let damp1 = damping;
        let damp2 = 1.0 - damping;
        let filtered = 0.0;
        let index = 0;

        Self {
            buffer,
            index,
            feedback,
            damp1,
            damp2,
            filtered,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.index];
        self.filtered = output * self.damp2 + self.filtered * self.damp1;
        self.buffer[self.index] = input + self.filtered * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }

    fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }

    fn set_damping(&mut self, damping: f32) {
        self.damp1 = damping;
        self.damp2 = 1.0 - damping;
    }
}

impl AllpassFilter {
    fn new(delay: usize, gain: f32, sample_rate: f32) -> Self {
        let buffer = vec![0.0; delay];
        let index = 0;

        Self {
            buffer,
            index,
            gain,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        let output = delayed - self.gain * input;
        self.buffer[self.index] = input + self.gain * output;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}