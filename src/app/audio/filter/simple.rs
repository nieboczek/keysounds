use crate::app::audio::{AudioProcessor, ProcessContext};

pub(super) struct Shittify {
    strength: i32,
    cutoff: i32,
}

impl Shittify {
    pub(super) fn new(strength: i32, cutoff: i32) -> Self {
        Shittify { strength, cutoff }
    }

    fn transform(&self, sample: f32) -> f32 {
        // DROP 16 BITS
        let sample_i16 = (sample * i16::MAX as f32) as i16;

        // BOOST THE AUDIO strength TIMES and then CLIP IT A LOT
        let distorted = (sample_i16 as i32 * self.strength).clamp(-self.cutoff, self.cutoff) as i16;

        // QUIETER AUDIO 2 TIMES and cast to f32
        (distorted / 2) as f32 / i16::MAX as f32
    }
}

impl AudioProcessor for Shittify {
    fn process(&mut self, samples: &mut [f32], _: ProcessContext) {
        for sample in samples {
            *sample = self.transform(*sample);
        }
    }
}

pub(super) struct BassBoost {
    prev_outputs: Vec<f32>,
    sample_rate: f32,
    cutoff: f32,
    gain: f32,
}

impl BassBoost {
    pub(super) fn new(sample_rate: u32, channels: usize, cutoff: f32, gain: f32) -> Self {
        BassBoost {
            prev_outputs: vec![0.0; channels],
            sample_rate: sample_rate as f32,
            cutoff,
            gain,
        }
    }
}

impl AudioProcessor for BassBoost {
    fn process(&mut self, samples: &mut [f32], context: ProcessContext) {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * self.cutoff);
        let dt = 1.0 / self.sample_rate;
        let alpha = dt / (rc + dt);

        for frame in samples.chunks_exact_mut(context.channels) {
            for (sample, prev_output) in frame.iter_mut().zip(&mut self.prev_outputs) {
                let low = *prev_output + alpha * (*sample - *prev_output);
                *prev_output = low;

                // Boost lows by mixing them back in.
                *sample = (*sample + low * (self.gain - 1.0)).clamp(-1.0, 1.0);
            }
        }
    }
}
