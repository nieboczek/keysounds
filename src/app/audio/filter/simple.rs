use crate::app::audio::AudioFilter;

pub(super) struct Shittify {
    strength: i32,
    cutoff: i32,
}

impl Shittify {
    pub(super) fn new(strength: i32, cutoff: i32) -> Self {
        Shittify { strength, cutoff }
    }
}

impl AudioFilter for Shittify {
    fn filter(&mut self, sample: f32) -> f32 {
        // LOSE 16 BITS
        let sample_i16 = (sample * i16::MAX as f32) as i16;

        // BOOST THE AUDIO 12 TIMES and then CLIP IT A LOT
        let distorted = (sample_i16 as i32 * self.strength).clamp(-self.cutoff, self.cutoff) as i16;

        // QUIETER AUDIO 2 TIMES and cast to f32
        (distorted / 2) as f32 / i16::MAX as f32
    }
}

pub(super) struct BoostBass {
    prev_output: f32,
    sample_rate: f32,
    cutoff: f32,
    gain: f32,
}

impl BoostBass {
    pub(super) fn new(sample_rate: u32, cutoff: f32, gain: f32) -> Self {
        BoostBass {
            prev_output: 0.0,
            sample_rate: sample_rate as f32,
            cutoff,
            gain,
        }
    }
}

impl AudioFilter for BoostBass {
    fn filter(&mut self, sample: f32) -> f32 {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * self.cutoff);
        let dt = 1.0 / self.sample_rate;
        let alpha = dt / (rc + dt);

        let low = self.prev_output + alpha * (sample - self.prev_output);
        self.prev_output = low;

        // Boost lows by mixing them back in
        let boosted = sample + (low * (self.gain - 1.0));

        // Clamp to [-1, 1] to avoid clipping
        boosted.clamp(-1.0, 1.0)
    }
}
