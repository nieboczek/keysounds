use crate::app::audio::AudioFilter;

pub(super) struct Reverb {
    combs: Vec<Comb>,
    all_pass: AllPass,
    wet: f32,
}

impl Reverb {
    pub(super) fn new(sample_rate: u32, room_size: f32, damping: f32, wet: f32) -> Self {
        let scale = (sample_rate as f32) / 44100.0;

        let comb_sizes = [1116, 1188, 1277, 1356];
        let all_pass_size = 556;

        let feedback = 0.7 + room_size * 0.28;

        let combs = comb_sizes
            .iter()
            .map(|&size| Comb::new((size as f32 * scale) as usize, feedback, damping))
            .collect();

        let all_pass = AllPass::new((all_pass_size as f32 * scale) as usize, 0.5);

        Self {
            combs,
            all_pass,
            wet,
        }
    }
}

impl AudioFilter for Reverb {
    fn filter(&mut self, sample: f32) -> f32 {
        let mut acc = 0.0;

        for comb in &mut self.combs {
            acc += comb.process(sample);
        }

        let out = self.all_pass.process(acc);

        // Dry / wet mix
        (sample * (1.0 - self.wet) + out * self.wet).clamp(-1.0, 1.0)
    }
}

struct Comb {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
    damping: f32,
    filter_store: f32,
}

impl Comb {
    fn new(size: usize, feedback: f32, damping: f32) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
            feedback,
            damping,
            filter_store: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.index];

        self.filter_store = output * (1.0 - self.damping) + self.filter_store * self.damping;

        self.buffer[self.index] = input + self.filter_store * self.feedback;

        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

struct AllPass {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
}

impl AllPass {
    fn new(size: usize, feedback: f32) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
            feedback,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buf_out = self.buffer[self.index];
        let output = -input + buf_out;

        self.buffer[self.index] = input + buf_out * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();

        output
    }
}
