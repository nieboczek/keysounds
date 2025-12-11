use crate::app;

pub struct FilterChain {
    filters: Vec<Box<dyn AudioFilter + Send + Sync>>,
    sample_rate: u32,
}

pub trait AudioFilter {
    fn filter(&mut self, sample: f32) -> f32;
}

impl FilterChain {
    pub(super) fn new(sample_rate: u32) -> FilterChain {
        FilterChain {
            filters: Vec::new(),
            sample_rate,
        }
    }

    pub(super) fn filter(&mut self, mut sample: f32) -> f32 {
        for filter in &mut self.filters {
            sample = filter.filter(sample);
        }
        sample
    }

    pub fn sync_with_config(&mut self, filters: &Vec<app::AudioFilter>) {
        self.filters = filters
            .iter()
            .map(|filter| {
                let transformed: Box<dyn AudioFilter + Send + Sync> = match filter {
                    app::AudioFilter::BoostBass { gain, cutoff } => Box::new(BoostBass {
                        prev_output: 0.0,
                        sample_rate: self.sample_rate as f32,
                        gain: *gain,
                        cutoff: *cutoff,
                    }),
                    app::AudioFilter::Shittify => Box::new(Shittify),
                };
                transformed
            })
            .collect();
    }
}

struct Shittify;

impl AudioFilter for Shittify {
    fn filter(&mut self, sample: f32) -> f32 {
        // LOSE 16 BITS
        let sample_i16 = (sample * i16::MAX as f32) as i16;

        // BOOST THE AUDIO 12 TIMES and then CLIP IT A LOT
        let distorted = (sample_i16 as i32 * 12).clamp(-10000, 10000) as i16;

        // QUIETER AUDIO 2 TIMES and cast to f32
        (distorted / 2) as f32 / i16::MAX as f32
    }
}

struct BoostBass {
    prev_output: f32,
    sample_rate: f32,
    cutoff: f32,
    gain: f32,
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
