pub struct LadderFilter {
    sample_rate: f32,
    cutoff: f32,
    resonance: f32,
    drive: f32,
    saturation: f32,
    stage: [f32; 4],
    delay: [f32; 4],
    tanhstage: [f32; 3],
    old_x: f32,
    old_y: f32,
}

impl LadderFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            cutoff: 1000.0,
            resonance: 0.0,
            drive: 1.0,
            saturation: 1.0,
            stage: [0.0; 4],
            delay: [0.0; 4],
            tanhstage: [0.0; 3],
            old_x: 0.0,
            old_y: 0.0,
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff.clamp(20.0, self.sample_rate * 0.49);
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.clamp(0.0, 4.0);
    }

    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(0.1, 10.0);
    }

    pub fn set_saturation(&mut self, saturation: f32) {
        self.saturation = saturation.clamp(0.1, 10.0);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let fc = self.cutoff / self.sample_rate;
        let f = fc * 1.16;
        let fb = self.resonance * (1.0 - 0.15 * f * f);
        let mut input_with_feedback = input * self.drive - self.old_y * fb;
        self.old_x = input_with_feedback;
        // Four cascaded one-pole filters (bilinear transform)
        for i in 0..4 {
            if i != 0 {
                input_with_feedback = self.stage[i-1];
            }
            self.stage[i] = input_with_feedback * f + self.delay[i] * (1.0 - f);
        }
        self.delay = self.stage;
        // Oversampled nonlinear processing
        self.tanhstage[0] = fast_tanh(self.stage[3] * self.saturation);
        self.tanhstage[1] = fast_tanh(self.stage[3] * self.saturation);
        self.tanhstage[2] = fast_tanh(self.stage[3] * self.saturation);
        self.old_y = (self.tanhstage[0] + self.tanhstage[1] + self.tanhstage[2]) / 3.0;
        self.old_y
    }
}

// Fast approximation of tanh
fn fast_tanh(x: f32) -> f32 {
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}