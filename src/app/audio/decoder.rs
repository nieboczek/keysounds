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

    // linear resampling
    target_sr: u32,
    step: f32,
    pos: f32,
    a: [f32; 2],
    b: [f32; 2],
    a_idx: usize,
    out_ch: usize,
    out_pair: [f32; 2],
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
        if self.out_ch == 0 {
            let target = self.pos.floor() as usize;

            while self.a_idx + 1 < target {
                self.skip_raw_frame()?;
                self.a_idx += 1;
            }

            if self.a_idx < target {
                self.a = self.b;
                self.b = self.read_raw_frame()?;
                self.a_idx += 1;
            }

            let frac = self.pos.fract();
            for i in 0..2 {
                self.out_pair[i] = self.a[i] + (self.b[i] - self.a[i]) * frac;
            }
            self.pos += self.step;
        }

        let sample = self.out_pair[self.out_ch];
        self.out_ch = (self.out_ch + 1) & 1;
        self.counted_samples += 1;
        Some(sample * self.volume)
    }

    fn read_raw_frame(&mut self) -> Option<[f32; 2]> {
        let l = self.next_raw_sample()?;
        let r = if self.spec.channels.count() == 1 {
            l
        } else {
            self.next_raw_sample()?
        };
        Some([l, r])
    }

    fn skip_raw_frame(&mut self) -> Option<()> {
        self.next_raw_sample()?;
        if self.spec.channels.count() == 2 {
            self.next_raw_sample()?;
        }
        Some(())
    }

    pub fn seek(&mut self, pos: Duration) {
        let mut target = pos;
        if let Some(total_duration) = self.total_duration
            && target > total_duration
        {
            target = total_duration;
        }

        let active_channel = self.current_packet_offset % self.spec.channels.count();

        let _ = self.format.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: target.into(),
                track_id: None,
            },
        );

        self.decoder.reset();
        self.current_packet_offset = usize::MAX;
        self.pos = 0.0;
        self.a_idx = 0;
        self.out_ch = 0;
        self.previous_mono_sample = None;

        self.a = self.read_raw_frame().unwrap_or([0.0; 2]);
        self.b = self.read_raw_frame().unwrap_or([0.0; 2]);

        for _ in 0..active_channel {
            self.next_sample();
        }
    }

    pub fn get_pos(&self) -> Duration {
        let secs = self.counted_samples as f64 / self.target_sr as f64 / 2.0;
        Duration::from_secs_f64(secs)
    }

    pub(super) fn total_duration(&self) -> Option<Duration> {
        self.total_duration
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

        let step = spec.rate as f32 / target_sample_rate as f32;

        let mut decoder_obj = AudioDecoder {
            decoder,
            current_packet_offset: 0,
            format: probe_result.format,
            total_duration,
            buffer,
            spec,
            counted_samples: 0,
            volume,
            previous_mono_sample: None,
            target_sr: target_sample_rate,
            step,
            pos: 0.0,
            a: [0.0; 2],
            b: [0.0; 2],
            a_idx: 0,
            out_ch: 0,
            out_pair: [0.0; 2],
        };

        decoder_obj.a = decoder_obj.read_raw_frame().unwrap_or([0.0; 2]);
        decoder_obj.b = decoder_obj.read_raw_frame().unwrap_or([0.0; 2]);

        decoder_obj
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
