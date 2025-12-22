use crate::app::config::AudioFilter;
use reverb::Reverb;
use simple::{BoostBass, Shittify};

mod reverb;
mod simple;

pub struct FilterChain {
    filters: Vec<Box<dyn SampleTransformer>>,
    sample_rate: u32,
}

pub trait SampleTransformer: Send + Sync {
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

    pub fn sync_with_vector(&mut self, filters: Vec<AudioFilter>) {
        self.filters.clear();
        self.filters.extend(
            filters
                .into_iter()
                .map(|filter| Self::filter_to_transformer(self.sample_rate, filter)),
        );
    }

    fn filter_to_transformer(sample_rate: u32, filter: AudioFilter) -> Box<dyn SampleTransformer> {
        match filter {
            AudioFilter::BoostBass { gain, cutoff } => {
                Box::new(BoostBass::new(sample_rate, cutoff, gain))
            }
            AudioFilter::Shittify { strength, cutoff } => {
                Box::new(Shittify::new(strength, cutoff))
            }
            AudioFilter::Reverb {
                room_size,
                damping,
                wet,
            } => Box::new(Reverb::new(sample_rate, room_size, damping, wet)),
        }
    }
}
