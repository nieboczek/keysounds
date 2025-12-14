use crate::app::config;
use reverb::Reverb;
use simple::{BoostBass, Shittify};

mod reverb;
mod simple;

pub struct FilterChain {
    filters: Vec<Box<dyn AudioFilter>>,
    sample_rate: u32,
}

pub trait AudioFilter: Send + Sync {
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

    pub fn sync_with_vector(&mut self, filters: Vec<config::AudioFilter>) {
        self.filters.clear();
        self.filters.extend(
            filters
                .into_iter()
                .map(|filter| Self::transform_filter(self.sample_rate, filter)),
        );
    }

    fn transform_filter(sample_rate: u32, filter: config::AudioFilter) -> Box<dyn AudioFilter> {
        match filter {
            config::AudioFilter::BoostBass { gain, cutoff } => {
                Box::new(BoostBass::new(sample_rate, cutoff, gain))
            }
            config::AudioFilter::Shittify { strength, cutoff } => {
                Box::new(Shittify::new(strength, cutoff))
            }
            config::AudioFilter::Reverb {
                room_size,
                damping,
                wet,
            } => Box::new(Reverb::new(sample_rate, room_size, damping, wet)),
        }
    }
}
