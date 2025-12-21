use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer, SignalSpec};
use symphonia::core::codecs::{CODEC_TYPE_NULL, Decoder};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::probe::Hint;
use symphonia::core::units;
use symphonia::default::{get_codecs, get_probe};

// TODO: replace unwraps in this file with actual error handling
// TODO(13-12-2025): this outputs static noise :skull:

pub struct AudioDecoder {
    decoder: Box<dyn Decoder>,
    current_packet_offset: usize,
    format: Box<dyn FormatReader>,
    total_duration: Option<Duration>,
    buffer: SampleBuffer<f32>,
    spec: SignalSpec,
}

impl AudioDecoder {
    pub fn new(audio_path: &str) -> Self {
        let path = Path::new(audio_path);
        let file = File::open(path).unwrap();
        let byte_len = file.metadata().unwrap().len();
        let buf_reader = BufReader::new(file);

        Self::create_decoder(buf_reader, byte_len, path.extension())
    }

    pub fn next_sample(&mut self) -> Option<f32> {
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

        Some(sample)
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
        Duration::default()
    }

    pub(super) fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }

    #[inline]
    fn get_buffer(decoded: AudioBufferRef, spec: &SignalSpec) -> SampleBuffer<f32> {
        let duration = units::Duration::from(decoded.capacity() as u64);
        let mut buffer = SampleBuffer::new(duration, *spec);
        buffer.copy_interleaved_ref(decoded);
        buffer
    }

    fn create_decoder(
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

        AudioDecoder {
            decoder,
            current_packet_offset: 0,
            format: probe_result.format,
            total_duration,
            buffer,
            spec,
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
