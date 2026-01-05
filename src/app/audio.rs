use crate::app::{App, Sfx, SfxData};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Stream};
use ringbuf::HeapRb;
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub use decoder::AudioDecoder;
pub use filter::{FilterChain, SampleTransformer};

mod decoder;
mod filter;

pub(super) type KeepAlive = (Stream, Stream, Stream);

const CHANNELS: usize = 2;
const BLOCK_FRAMES: usize = 512;
const BLOCK_SAMPLES: usize = BLOCK_FRAMES * CHANNELS;

const RING_BLOCKS: usize = 8;
const RING_CAPACITY: usize = BLOCK_SAMPLES * RING_BLOCKS;

impl App {
    pub(super) fn play_sfx(&mut self, sfx: Sfx, randomly_triggered: bool) {
        let decoder = AudioDecoder::new(&sfx.path, self.target_sample_rate, sfx.volume);
        let duration = decoder.total_duration().unwrap_or_default();

        let mut guard = self.decoder.lock().unwrap();
        *guard = Some(decoder);

        self.sfx_data = Some(SfxData {
            duration,
            sfx,
            randomly_triggered,
        });
    }

    #[inline]
    pub(super) fn create_streams(
        mic_device: &Device,
        out_device: &Device,
        virtual_out_device: &Device,
        decoder: Arc<Mutex<Option<AudioDecoder>>>,
    ) -> (Arc<Mutex<FilterChain>>, u32, KeepAlive) {
        let mic_config = mic_device.default_input_config().unwrap();
        let out_config = out_device.default_output_config().unwrap();
        let virtual_out_config = virtual_out_device.default_output_config().unwrap();

        let sample_rate = out_config.sample_rate();
        let filter_chain = Arc::new(Mutex::new(FilterChain::new(sample_rate)));

        let mic_rb: HeapRb<f32> = HeapRb::new(RING_CAPACITY);
        let decoder_rb: HeapRb<f32> = HeapRb::new(RING_CAPACITY);
        let decoder_too_rb: HeapRb<f32> = HeapRb::new(RING_CAPACITY);

        let (mut mic_prod, mut mic_cons) = mic_rb.split();
        let (mut decoder_prod, mut decoder_cons) = decoder_rb.split();
        let (mut decoder_too_prod, mut decoder_too_cons) = decoder_too_rb.split();

        let mic_stream = mic_device
            .build_input_stream(
                &mic_config.into(),
                move |data: &[f32], _| {
                    mic_prod.push_slice(data);
                },
                |err| eprintln!("Input stream error: {err}"),
                None,
            )
            .unwrap();

        let out_stream = out_device
            .build_output_stream(
                &out_config.into(),
                move |data: &mut [f32], _| {
                    decoder_cons.pop_slice(data);
                },
                |err| eprintln!("Default output stream error: {err}"),
                None,
            )
            .unwrap();

        let filter_chain_too = Arc::clone(&filter_chain);
        let virtual_out_stream = virtual_out_device
            .build_output_stream(
                &virtual_out_config.into(),
                move |data: &mut [f32], _| {
                    let mut mic_iter = mic_cons.pop_iter();
                    let mut decoder_iter = decoder_too_cons.pop_iter();

                    let mut chain = filter_chain_too.lock().unwrap();
                    for i in 0..data.len() {
                        data[i] = chain.filter(mic_iter.next().unwrap_or_default())
                            + decoder_iter.next().unwrap_or_default();
                    }
                },
                |err| eprintln!("Output stream error: {err}"),
                None,
            )
            .unwrap();

        thread::spawn(move || {
            let mut buf = [0.0f32; BLOCK_SAMPLES];
            loop {
                if decoder_prod.vacant_len() < RING_CAPACITY / 2 {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }

                let mut guard = decoder.lock().unwrap();
                let Some(decoder) = guard.as_mut() else {
                    for i in 0..BLOCK_SAMPLES {
                        buf[i] = 0.0;
                    }

                    decoder_prod.push_slice(&buf);
                    decoder_too_prod.push_slice(&buf);

                    thread::sleep(Duration::from_millis(10));
                    continue;
                };

                let mut eof = false;
                for frame in 0..BLOCK_FRAMES {
                    for ch in 0..CHANNELS {
                        match decoder.next_sample() {
                            Some(sample) => buf[frame * CHANNELS + ch] = sample,
                            None => {
                                eof = true;
                                buf[frame * CHANNELS + ch] = 0.0;
                            }
                        }
                    }
                }

                if eof {
                    // delete decoder
                    *guard = None;
                }
                std::mem::drop(guard);

                decoder_prod.push_slice(&buf);
                decoder_too_prod.push_slice(&buf);
                thread::sleep(Duration::from_millis(5));
            }
        });

        mic_stream.play().unwrap();
        out_stream.play().unwrap();
        virtual_out_stream.play().unwrap();

        (
            filter_chain,
            sample_rate,
            (mic_stream, out_stream, virtual_out_stream),
        )
    }
}
