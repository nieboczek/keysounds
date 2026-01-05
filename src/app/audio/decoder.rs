use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Async, Resampler, WindowFunction};
use std::{
    ffi::OsStr,
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::Path,
    time::Duration,
};
use symphonia::{
    core::{
        audio::{AudioBufferRef, SampleBuffer, SignalSpec},
        codecs::{CODEC_TYPE_NULL, Decoder},
        errors::Error,
        formats::{FormatOptions, FormatReader, SeekMode, SeekTo},
        io::{MediaSource, MediaSourceStream},
        probe::Hint,
        units,
    },
    default::{get_codecs, get_probe},
};

use crate::app::audio::{BLOCK_FRAMES, BLOCK_SAMPLES, CHANNELS};

// TODO: replace unwraps in this file with actual error handling

pub struct AudioDecoder {
    decoder: Box<dyn Decoder>,
    current_packet_offset: usize,
    format: Box<dyn FormatReader>,
    total_duration: Option<Duration>,
    buffer: SampleBuffer<f32>,
    spec: SignalSpec,
    counted_samples: usize,
    volume: f32,
    previous_mono_sample: Option<f32>,

    resampler: Option<Async<f32>>,
    resampler_out: Vec<f32>,
    resampler_pos: usize,
}

impl AudioDecoder {
    pub fn new(audio_path: &str, sample_rate: u32, volume: f32) -> Self {
        let path = Path::new(audio_path);
        let file = File::open(path).unwrap();
        let byte_len = file.metadata().unwrap().len();
        let buf_reader = BufReader::new(file);

        Self::create_decoder(sample_rate, volume, buf_reader, byte_len, path.extension())
    }

    pub fn next_sample(&mut self) -> Option<f32> {
        let sample = if self.resampler.is_some() {
            if self.resampler_pos >= self.resampler_out.len() {
                self.resample_batch();
                self.resampler_pos = 0;
            }

            let sample = self.resampler_out[self.resampler_pos];
            self.resampler_pos += 1;
            sample
        } else {
            self.next_raw_sample()?
        };

        self.counted_samples += 1;
        Some(sample * self.volume)
    }

    pub fn seek(&mut self, pos: Duration) {
        let mut target = pos;
        if let Some(total_duration) = self.total_duration
            && target > total_duration
        {
            target = total_duration;
        }

        let active_channel = self.current_packet_offset % self.spec.channels.count();

        // we don't really care if it works or not
        let _ = self.format.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: target.into(),
                track_id: None,
            },
        );

        self.decoder.reset();
        self.current_packet_offset = usize::MAX;

        for _ in 0..active_channel {
            self.next_sample();
        }
    }

    pub fn get_pos(&self) -> Duration {
        let secs =
            self.counted_samples as f64 / self.spec.rate as f64 / self.spec.channels.count() as f64;

        Duration::from_secs_f64(secs)
    }

    pub(super) fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }

    fn resample_batch(&mut self) -> Option<()> {
        let mut input = [0.0f32; BLOCK_SAMPLES];

        if self.spec.channels.count() == 1 {
            for frame in 0..BLOCK_SAMPLES {
                input[frame] = self.next_raw_sample()?;
            }
        } else {
            for frame in 0..BLOCK_FRAMES {
                for ch in 0..CHANNELS {
                    // TODO: instead of Try (?), operate on lower sample count
                    // TODO: or even precalculate some count at audio decoder creation
                    input[frame * CHANNELS + ch] = self.next_raw_sample()?;
                }
            }
        }

        let output_frames = self.resampler_out.len() / 2;
        let input = InterleavedSlice::new(&input, CHANNELS, BLOCK_FRAMES).unwrap();
        let mut output =
            InterleavedSlice::new_mut(&mut self.resampler_out, CHANNELS, output_frames).unwrap();

        self.resampler
            .as_mut()?
            .process_into_buffer(&input, &mut output, None)
            .unwrap();

        Some(())
    }

    fn next_raw_sample(&mut self) -> Option<f32> {
        if self.spec.channels.count() == 1 {
            if let Some(previous) = self.previous_mono_sample.take() {
                return Some(previous);
            }
        }

        if self.current_packet_offset >= self.buffer.len() {
            let decoded = loop {
                let packet = self.format.next_packet().ok()?;
                let decoded = match self.decoder.decode(&packet) {
                    Ok(decoded) => decoded,
                    Err(Error::DecodeError(_)) => continue,
                    Err(_) => return None,
                };

                if decoded.frames() > 0 {
                    break decoded;
                }
            };

            decoded.spec().clone_into(&mut self.spec);
            self.buffer = Self::get_buffer(decoded, &self.spec);
            self.current_packet_offset = 0;
        }

        let sample = *self.buffer.samples().get(self.current_packet_offset)?;
        self.current_packet_offset += 1;
        self.previous_mono_sample = Some(sample);

        Some(sample)
    }

    #[inline]
    fn get_buffer(decoded: AudioBufferRef, spec: &SignalSpec) -> SampleBuffer<f32> {
        let duration = units::Duration::from(decoded.capacity() as u64);
        let mut buffer = SampleBuffer::new(duration, *spec);
        buffer.copy_interleaved_ref(decoded);
        buffer
    }

    fn create_decoder(
        target_sample_rate: u32,
        volume: f32,
        buf: BufReader<File>,
        byte_len: u64,
        file_extension: Option<&OsStr>,
    ) -> AudioDecoder {
        let mut hint = Hint::new();
        if let Some(extension) = file_extension {
            hint.with_extension(extension.to_str().unwrap());
        }

        let mss = MediaSourceStream::new(
            Box::new(AudioSource {
                byte_len,
                inner: buf,
            }),
            Default::default(),
        );

        let format_opts = FormatOptions {
            enable_gapless: false,
            ..Default::default()
        };

        let mut probe_result = get_probe()
            .format(&hint, mss, &format_opts, &Default::default())
            .unwrap();

        let default_track = match probe_result.format.default_track() {
            Some(track) => track,
            None => panic!("no audio tracks found"),
        };

        let mut track_id = u32::MAX;
        let track = match probe_result.format.tracks().iter().find(|track| {
            if track.codec_params.codec != CODEC_TYPE_NULL {
                track_id = track.id;
                return true;
            }
            false
        }) {
            Some(track) => track,
            None => panic!(),
        };

        let mut decoder = get_codecs()
            .make(&track.codec_params, &Default::default())
            .unwrap();

        let total_duration = default_track
            .codec_params
            .time_base
            .zip(default_track.codec_params.n_frames)
            .map(|(time_base, n_frames)| time_base.calc_time(n_frames).into());

        let decoded = loop {
            let packet = match probe_result.format.next_packet() {
                Ok(packet) => packet,
                Err(Error::IoError(_)) => break decoder.last_decoded(),
                Err(e) => panic!("{e}"),
            };

            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => break decoded,
                Err(Error::DecodeError(_)) => continue,
                Err(_) => panic!(),
            }
        };

        let spec = *decoded.spec();
        let buffer = Self::get_buffer(decoded, &spec);

        let resampler = if target_sample_rate != spec.rate {
            const WINDOW_FUNCTION: WindowFunction = WindowFunction::BlackmanHarris2;
            const SINC_LEN: usize = 256;
            const OVERSAMPLING_FACTOR: usize = 128;

            let ratio = target_sample_rate as f64 / spec.rate as f64;
            let params = rubato::SincInterpolationParameters {
                sinc_len: SINC_LEN,
                f_cutoff: rubato::calculate_cutoff(SINC_LEN, WINDOW_FUNCTION),
                interpolation: rubato::SincInterpolationType::Quadratic,
                oversampling_factor: OVERSAMPLING_FACTOR,
                window: WINDOW_FUNCTION,
            };

            Some(
                Async::new_sinc(
                    ratio,
                    1.0,
                    &params,
                    BLOCK_FRAMES,
                    CHANNELS,
                    rubato::FixedAsync::Input,
                )
                .unwrap(),
            )
        } else {
            None
        };

        let capacity = if let Some(resampler) = &resampler {
            resampler.output_frames_max() * 2
        } else {
            0
        };

        AudioDecoder {
            decoder,
            current_packet_offset: 0,
            format: probe_result.format,
            total_duration,
            buffer,
            spec,
            counted_samples: 0,
            volume,
            previous_mono_sample: None,
            resampler,
            resampler_out: vec![0.0f32; capacity],
            resampler_pos: 0,
        }
    }
}

struct AudioSource<T: Read + Seek + Send + Sync> {
    inner: T,
    byte_len: u64,
}

impl<T: Read + Seek + Send + Sync> Read for AudioSource<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Read + Seek + Send + Sync> Seek for AudioSource<T> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl<T: Read + Seek + Send + Sync> MediaSource for AudioSource<T> {
    #[inline]
    fn byte_len(&self) -> Option<u64> {
        Some(self.byte_len)
    }

    #[inline]
    fn is_seekable(&self) -> bool {
        true
    }
}
