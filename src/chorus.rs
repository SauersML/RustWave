use std::f32::consts::PI;
use rand::Rng;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Chorus {
    buffer_left: Vec<f32>,
    buffer_right: Vec<f32>,
    index: usize,
    size: usize,
    mode: ChorusMode,
    sample_rate: f32,
    low_pass_filter: LowPassFilter,
    high_pass_filter: HighPassFilter,
    noise_generator: Arc<Mutex<NoiseGenerator>>,
    saturation: Saturation,
    feedback: f32,
    voices: Vec<Voice>,
    rate: f32,
    depth: f32,
    wet_dry_mix: f32,
    prev_delay_left: Vec<f32>,
    prev_delay_right: Vec<f32>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChorusMode {
    Off,
    I,
    II,
    III,
    IV,
}

struct LowPassFilter {
    prev: f32,
    cutoff: f32,
    resonance: f32,
}

struct HighPassFilter {
    prev_input: f32,
    prev_output: f32,
    cutoff: f32,
}

struct NoiseGenerator {
    level: f32,
    prev: f32,
}

struct Saturation {
    drive: f32,
}

struct Voice {
    phase_left: f32,
    phase_right: f32,
    rate_left: f32,
    rate_right: f32,
    depth: f32,
    smooth_depth: f32,
}

impl Chorus {
    pub fn new(sample_rate: f32) -> Self {
        let max_delay_ms = 40.0;
        let size = (sample_rate * max_delay_ms / 1000.0) as usize;
        Self {
            buffer_left: vec![0.0; size],
            buffer_right: vec![0.0; size],
            index: 0,
            size,
            mode: ChorusMode::Off,
            sample_rate,
            low_pass_filter: LowPassFilter::new(sample_rate),
            high_pass_filter: HighPassFilter::new(sample_rate),
            noise_generator: Arc::new(Mutex::new(NoiseGenerator::new())),
            saturation: Saturation::new(),
            feedback: 0.25,
            rate: 0.5,
            depth: 0.5,
            voices: vec![
                Voice::new(0.513, 0.515, 0.7),
                Voice::new(0.75, 0.753, 0.6),
                Voice::new(0.95, 0.953, 0.5),
            ],
            wet_dry_mix: 0.5,
            prev_delay_left: vec![0.0; 3],
            prev_delay_right: vec![0.0; 3],
        }
    }

    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate.clamp(0.1, 10.0);
        for voice in &mut self.voices {
            voice.rate_left = self.rate * (0.9 + rand::thread_rng().gen::<f32>() * 0.2);
            voice.rate_right = self.rate * (0.9 + rand::thread_rng().gen::<f32>() * 0.2);
        }
    }

    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
        for voice in &mut self.voices {
            voice.depth = self.depth * (0.9 + rand::thread_rng().gen::<f32>() * 0.2);
        }
    }

    pub fn set_mode(&mut self, mode: ChorusMode) {
        self.mode = mode;
        match mode {
            ChorusMode::Off => {
                self.voices.clear();
                self.wet_dry_mix = 0.0;
            },
            ChorusMode::I => {
                self.voices = vec![Voice::new(0.513, 0.515, 0.00535)];
                self.wet_dry_mix = 0.5;
            },
            ChorusMode::II => {
                self.voices = vec![Voice::new(0.863, 0.865, 0.00535)];
                self.wet_dry_mix = 0.8;
            },
            ChorusMode::III => {
                self.voices = vec![
                    Voice::new(0.513, 0.515, 0.0037),
                    Voice::new(0.863, 0.865, 0.0037),
                ];
                self.wet_dry_mix = 0.5;
            },
            ChorusMode::IV => {
                self.voices = vec![
                    Voice::new(0.5, 0.502, 0.007),
                    Voice::new(0.75, 0.752, 0.006),
                    Voice::new(1.0, 1.002, 0.005),
                    Voice::new(1.25, 1.252, 0.004),
                ];
                self.wet_dry_mix = 0.6;
            },
        }
        self.prev_delay_left = vec![0.0; self.voices.len()];
        self.prev_delay_right = vec![0.0; self.voices.len()];
    }

    pub fn process(&mut self, input: f32) -> (f32, f32) {
        if self.mode == ChorusMode::Off {
            return (input, input);
        }

        let high_passed = self.high_pass_filter.process(input);
        let filtered_input = self.low_pass_filter.process(high_passed);

        let feedback_left = self.buffer_left[self.index];
        let feedback_right = self.buffer_right[self.index];
        let feedback = (feedback_left + feedback_right) * 0.5;
        let input_with_feedback = filtered_input + (self.feedback * feedback).clamp(-1.0, 1.0);

        self.buffer_left[self.index] = input_with_feedback;
        self.buffer_right[self.index] = input_with_feedback;
        self.index = (self.index + 1) % self.size;

        let (left_output, right_output) = self.calculate_delay_samples(input_with_feedback);

        let noise = self.noise_generator.lock().unwrap().generate();
        let left_output = left_output + noise;
        let right_output = right_output + noise;

        let left_output = self.saturation.process(left_output);
        let right_output = self.saturation.process(right_output);

        let wet_dry_mix = self.wet_dry_mix.clamp(0.0, 1.0);
        let left = (1.0 - wet_dry_mix) * input + wet_dry_mix * left_output;
        let right = (1.0 - wet_dry_mix) * input + wet_dry_mix * right_output;

        (left.clamp(-1.0, 1.0), right.clamp(-1.0, 1.0))
    }

    fn calculate_delay_samples(&mut self, input: f32) -> (f32, f32) {
        let mut left_output = 0.0;
        let mut right_output = 0.0;

        for voice in &mut self.voices {
            voice.phase_left += voice.rate_left / self.sample_rate;
            voice.phase_right += voice.rate_right / self.sample_rate;
            if voice.phase_left >= 1.0 { voice.phase_left -= 1.0; }
            if voice.phase_right >= 1.0 { voice.phase_right -= 1.0; }

            voice.smooth_depth += (voice.depth - voice.smooth_depth) * 0.001;

            let lfo_left = ((2.0 * PI * voice.phase_left).sin() * 0.51 + 0.5) * 0.5 +
                           ((2.0 * PI * voice.phase_left * 1.101).sin() * 0.5 + 0.5) * 0.5;
            let lfo_right = ((2.0 * PI * voice.phase_right).sin() * 0.5 + 0.51) * 0.5 +
                            ((2.0 * PI * voice.phase_right * 1.1).sin() * 0.5 + 0.5) * 0.5;

            let delay_left = (voice.smooth_depth * self.sample_rate * lfo_left).min(self.size as f32 - 1.0);
            let delay_right = (voice.smooth_depth * self.sample_rate * lfo_right).min(self.size as f32 - 1.0);

            let index_left = (self.index as f32 - delay_left + self.size as f32) as usize % self.size;
            let index_right = (self.index as f32 - delay_right + self.size as f32) as usize % self.size;

            let frac_left = delay_left.fract();
            let frac_right = delay_right.fract();

            let sample_left = cubic_interpolate(&[
                self.buffer_left[(index_left + self.size - 1) % self.size],
                self.buffer_left[index_left],
                self.buffer_left[(index_left + 1) % self.size],
                self.buffer_left[(index_left + 2) % self.size],
            ], frac_left);

            let sample_right = cubic_interpolate(&[
                self.buffer_right[(index_right + self.size - 1) % self.size],
                self.buffer_right[index_right],
                self.buffer_right[(index_right + 1) % self.size],
                self.buffer_right[(index_right + 2) % self.size],
            ], frac_right);

            left_output += sample_left;
            right_output += sample_right;
        }

        if !self.voices.is_empty() {
            left_output = left_output / self.voices.len() as f32 + input * 0.5;
            right_output = right_output / self.voices.len() as f32 + input * 0.5;
        } else {
            left_output = input;
            right_output = input;
        }

        (left_output, right_output)
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.99);
    }

    pub fn set_wet_dry_mix(&mut self, mix: f32) {
        self.wet_dry_mix = mix.clamp(0.0, 1.0);
    }

    pub fn set_drive(&mut self, drive: f32) {
        self.saturation.set_drive(drive);
    }
}

fn cubic_interpolate(y: &[f32; 4], mu: f32) -> f32 {
    let mu2 = mu * mu;
    let a0 = y[3] - y[2] - y[0] + y[1];
    let a1 = y[0] - y[1] - a0;
    let a2 = y[2] - y[0];
    let a3 = y[1];
    a0 * mu * mu2 + a1 * mu2 + a2 * mu + a3
}

impl LowPassFilter {
    fn new(sample_rate: f32) -> Self {
        Self {
            prev: 0.0,
            cutoff: 8000.0 / sample_rate,
            resonance: 0.5,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let alpha = self.cutoff / (self.cutoff + 1.0);
        let resonance_factor = self.resonance.clamp(0.0, 0.99);
        let output = self.prev + alpha * (input - self.prev + resonance_factor * (self.prev - input));
        self.prev = output;
        output
    }
}

impl HighPassFilter {
    fn new(sample_rate: f32) -> Self {
        Self {
            prev_input: 0.0,
            prev_output: 0.0,
            cutoff: 20.0 / sample_rate,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let alpha = 1.0 / (1.0 + self.cutoff);
        let output = alpha * (self.prev_output + input - self.prev_input);
        self.prev_input = input;
        self.prev_output = output;
        output
    }
}

impl NoiseGenerator {
    fn new() -> Self {
        Self {
            level: 0.0005,
            prev: 0.0,
        }
    }

    fn generate(&mut self) -> f32 {
        let mut rng = rand::thread_rng();
        let new_noise = rng.gen_range(-self.level..self.level);
        let output = (self.prev + new_noise) * 0.5;
        self.prev = new_noise;
        output
    }
}

impl Saturation {
    fn new() -> Self {
        Self {
            drive: 1.2,
        }
    }

    fn process(&self, input: f32) -> f32 {
        (input * self.drive).tanh()
    }

    fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(1.0, 10.0);
    }
}

impl Voice {
    fn new(rate_left: f32, rate_right: f32, depth: f32) -> Self {
        Self {
            phase_left: rand::thread_rng().gen(),
            phase_right: rand::thread_rng().gen(),
            rate_left,
            rate_right,
            depth,
            smooth_depth: depth,
        }
    }
}