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
    thermal_drift: f32,
    transistor_mismatch: [f32; 4],
    rng: Xoshiro256PlusPlus,
}

impl LadderFilter {
    pub fn new(sample_rate: f32) -> Self {
        let mut filter = Self {
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
            thermal_drift: 0.0,
            transistor_mismatch: [1.0; 4],
            rng: Xoshiro256PlusPlus::seed_from_u64(0),
        };
        filter.randomize_transistors();
        filter
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
        self.saturation = saturation.clamp(0.00, 2.00);
    }

    fn randomize_transistors(&mut self) {
        for i in 0..4 {
            // Subtle mismatch, within 0.5% of ideal
            self.transistor_mismatch[i] = 1.0 + (self.rng.next_u32() as f32 / u32::MAX as f32 - 0.5) * 0.005;
        }
    }

    fn update_thermal_drift(&mut self) {
        // Simulate slow thermal drift
        self.thermal_drift += (self.rng.next_u32() as f32 / u32::MAX as f32 - 0.5) * 0.0001;
        self.thermal_drift *= 0.9999; // Slow decay towards zero
    }

    pub fn process(&mut self, input: f32) -> f32 {
        self.update_thermal_drift();
        
        let fc = (self.cutoff * (1.0 + self.thermal_drift)) / self.sample_rate;
        let f = fc * 1.16;
        let fb = self.resonance * (1.0 - 0.15 * f * f);

        let mut input_with_feedback = input * self.drive - self.old_y * fb;
        self.old_x = input_with_feedback;

        // Four cascaded one-pole filters (bilinear transform) with transistor-like behavior
        for i in 0..4 {
            if i != 0 {
                input_with_feedback = self.stage[i-1];
            }
            let f_adjusted = f * self.transistor_mismatch[i];
            self.stage[i] = input_with_feedback * f_adjusted + self.delay[i] * (1.0 - f_adjusted);
        }
        self.delay = self.stage;

        // Oversampled nonlinear processing
        for i in 0..3 {
            self.tanhstage[i] = fast_tanh(self.stage[3] * self.saturation);
        }

        self.old_y = (self.tanhstage[0] + self.tanhstage[1] + self.tanhstage[2]) / 3.0;
        self.old_y
    }
}

// Fast approximation of tanh
fn fast_tanh(x: f32) -> f32 {
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

// Xoshiro256++ algorithm for high-quality pseudo-random numbers
struct Xoshiro256PlusPlus {
    s: [u64; 4],
}

impl Xoshiro256PlusPlus {
    fn seed_from_u64(seed: u64) -> Self {
        let mut state = [0; 4];
        let mut splitmix64 = seed;
        for i in 0..4 {
            splitmix64 = splitmix64.wrapping_add(0x9e3779b97f4a7c15);
            let z = (splitmix64 ^ (splitmix64 >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            let z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            state[i] = z ^ (z >> 31);
        }
        Self { s: state }
    }

    fn next_u32(&mut self) -> u32 {
        let result = (self.s[0].wrapping_add(self.s[3])).rotate_left(23).wrapping_add(self.s[0]);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result as u32
    }
}