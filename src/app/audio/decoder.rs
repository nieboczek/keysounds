use std::{
    ffi::OsStr,
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::Path,
    time::Duration,
};
use symphonia::{
    core::{
        audio::AudioSpec,
        codecs::{self, CodecParameters, audio::CODEC_ID_NULL_AUDIO},
        errors::Error,
        formats::{FormatReader, SeekMode, SeekTo, TrackType, probe::Hint},
        io::{MediaSource, MediaSourceStream},
        units,
    },
    default::{get_codecs, get_probe},
};

// TODO: replace unwraps in this file with actual error handling

pub struct AudioDecoder {
    decoder: Box<dyn codecs::audio::AudioDecoder>,
    current_packet_offset: usize,
    format: Box<dyn FormatReader>,
    total_duration: Option<Duration>,
    buffer: Vec<f32>,
    spec: AudioSpec,
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
        let r = if self.spec.channels().count() == 1 {
            l
        } else {
            self.next_raw_sample()?
        };
        Some([l, r])
    }

    fn skip_raw_frame(&mut self) -> Option<()> {
        self.next_raw_sample()?;
        if self.spec.channels().count() == 2 {
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

        let active_channel = self.current_packet_offset % self.spec.channels().count();

        let _ = self.format.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: symphonia::core::units::Time::try_new(
                    target.as_secs() as i64,
                    target.subsec_nanos(),
                )
                .unwrap(),
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

    pub fn pos_nanos(&self) -> u64 {
        let secs = self.counted_samples as f64 / self.target_sr as f64 / 2.0;
        (secs * 1_000_000_000.0) as u64
    }

    pub(super) fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }

    fn next_raw_sample(&mut self) -> Option<f32> {
        if self.spec.channels().count() == 1
            && let Some(previous) = self.previous_mono_sample.take()
        {
            return Some(previous);
        }

        if self.current_packet_offset >= self.buffer.len() {
            let decoded = loop {
                let packet = match self.format.next_packet() {
                    Ok(Some(packet)) => packet,
                    Ok(None) => return None,
                    Err(_) => return None,
                };
                let decoded = match self.decoder.decode(&packet) {
                    Ok(decoded) => decoded,
                    Err(Error::DecodeError(_)) => continue,
                    Err(_) => return None,
                };

                if decoded.frames() > 0 {
                    break decoded;
                }
            };

            self.spec = decoded.spec().clone();
            let mut buffer = Vec::new();
            decoded.copy_to_vec_interleaved(&mut buffer);
            self.buffer = buffer;
            self.current_packet_offset = 0;
        }

        let sample = *self.buffer.get(self.current_packet_offset)?;
        self.current_packet_offset += 1;
        self.previous_mono_sample = Some(sample);

        Some(sample)
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

        let mut format = get_probe()
            .probe(&hint, mss, Default::default(), Default::default())
            .unwrap();

        let default_track = match format.default_track(TrackType::Audio) {
            Some(track) => track,
            None => panic!("no audio tracks found"),
        };

        let mut track_id = u32::MAX;
        let track = match format.tracks().iter().find(|track| {
            if let Some(CodecParameters::Audio(audio_params)) = &track.codec_params
                && audio_params.codec != CODEC_ID_NULL_AUDIO
            {
                track_id = track.id;
                return true;
            }
            false
        }) {
            Some(track) => track,
            None => panic!(),
        };

        let audio_params = track
            .codec_params
            .as_ref()
            .and_then(|p| p.audio())
            .expect("expected audio codec parameters");

        let mut decoder = get_codecs()
            .make_audio_decoder(audio_params, &Default::default())
            .unwrap();

        let total_duration =
            default_track
                .time_base
                .zip(default_track.num_frames)
                .map(|(time_base, n_frames)| {
                    let time = time_base
                        .calc_time(units::Timestamp::new(n_frames as i64))
                        .expect("failed to calculate duration");
                    let nanos = time.as_nanos();
                    Duration::new(
                        (nanos / 1_000_000_000) as u64,
                        (nanos % 1_000_000_000) as u32,
                    )
                });

        let decoded = loop {
            let packet = match format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => break decoder.last_decoded(),
                Err(Error::IoError(_)) => break decoder.last_decoded(),
                Err(e) => panic!("{e}"),
            };

            if packet.track_id != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => break decoded,
                Err(Error::DecodeError(_)) => continue,
                Err(_) => panic!(),
            }
        };

        let spec = decoded.spec().clone();
        let mut buffer = Vec::new();
        decoded.copy_to_vec_interleaved(&mut buffer);

        let step = spec.rate() as f32 / target_sample_rate as f32;

        let mut decoder_obj = AudioDecoder {
            decoder,
            current_packet_offset: 0,
            format,
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
