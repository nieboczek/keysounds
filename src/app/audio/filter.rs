use crate::app::config::AudioFilter;

mod reverb;
mod simple;

use self::{
    reverb::Reverb,
    simple::{BassBoost, Shittify},
};

pub struct FilterChain {
    filters: Vec<Box<dyn AudioProcessor>>,
    context: ProcessContext,
}

#[derive(Clone, Copy)]
pub struct ProcessContext {
    pub sample_rate: u32,
    pub channels: usize,
}

pub trait AudioProcessor: Send {
    fn process(&mut self, samples: &mut [f32], context: ProcessContext);
}

impl FilterChain {
    pub(super) fn new(sample_rate: u32, channels: usize) -> FilterChain {
        FilterChain {
            filters: Vec::new(),
            context: ProcessContext {
                sample_rate,
                channels,
            },
        }
    }

    pub(super) fn process(&mut self, samples: &mut [f32]) {
        for filter in &mut self.filters {
            filter.process(samples, self.context);
        }
    }

    pub fn sync(&mut self, filters: impl IntoIterator<Item = AudioFilter>) {
        self.filters.clear();
        self.filters.extend(
            filters
                .into_iter()
                .map(|filter| Self::filter_to_processor(self.context, filter)),
        );
    }

    fn filter_to_processor(
        context: ProcessContext,
        filter: AudioFilter,
    ) -> Box<dyn AudioProcessor> {
        match filter {
            AudioFilter::BassBoost { gain, cutoff } => Box::new(BassBoost::new(
                context.sample_rate,
                context.channels,
                cutoff,
                gain,
            )),
            AudioFilter::Shittify { strength, cutoff } => Box::new(Shittify::new(strength, cutoff)),
            AudioFilter::Reverb {
                room_size,
                damping,
                wet,
            } => Box::new(Reverb::new(
                context.sample_rate,
                context.channels,
                room_size,
                damping,
                wet,
            )),
        }
    }
}
