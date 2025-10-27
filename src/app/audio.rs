use crate::app::{App, Audio};
use rodio::{
    Decoder, Source,
    cpal::{
        Device, Stream, StreamConfig,
        traits::{DeviceTrait, StreamTrait},
    },
};
use std::{
    fs::File,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

type AudioBuf = Arc<Mutex<Vec<f32>>>;

#[inline]
fn create_input_stream(device: &Device, config: &StreamConfig, buf: AudioBuf) -> Stream {
    let channels = config.channels as usize;

    device
        .build_input_stream(
            config,
            move |data: &[f32], _| {
                let mut buf = buf.lock().unwrap();

                for &sample in data {
                    buf.push(sample);
                }

                let max_size = 48000 * channels;
                let len = buf.len();

                if len > max_size {
                    buf.drain(0..len - max_size);
                }
            },
            |err| eprintln!("Input stream error: {err}"),
            None,
        )
        .unwrap()
}

#[inline]
fn create_output_stream(
    device: &Device,
    config: &StreamConfig,
    buf: AudioBuf,
    shit_mic: Arc<AtomicBool>,
) -> Stream {
    let channels = config.channels as usize;

    device
        .build_output_stream(
            config,
            move |data: &mut [f32], _| {
                let mut buffer = buf.lock().unwrap();

                for frame in data.chunks_mut(channels) {
                    let len = buffer.len();
                    if len >= channels {
                        // Take samples from buffer
                        for (i, sample_out) in frame.iter_mut().enumerate() {
                            if i < len {
                                if shit_mic.load(Ordering::Relaxed) {
                                    let sample_i16 = (buffer[i] * i16::MAX as f32) as i16;

                                    // BOOST THE AUDIO 15 TIMES and then CLIP IT A LOT
                                    let distorted = (sample_i16 as i32 * 15).clamp(-10000, 10000) as i16;

                                    // QUIETER AUDIO 2 TIMES and cast to f32
                                    let sample = (distorted / 2) as f32 / i16::MAX as f32;

                                    *sample_out = sample;
                                } else {
                                    *sample_out = buffer[i];
                                }
                            }
                        }

                        // Remove used samples
                        buffer.drain(0..channels.min(len));
                    } else {
                        // No audio available, output silence
                        for sample_out in frame.iter_mut() {
                            *sample_out = 0.0;
                        }
                    }
                }
            },
            |err| eprintln!("Output stream error: {err}"),
            None,
        )
        .unwrap()
}

#[inline]
pub(crate) fn forward_input(
    input_device: &Device,
    output_device: &Device,
    shit_mic: Arc<AtomicBool>,
) -> (Stream, Stream) {
    let input_config = input_device.default_input_config().unwrap();
    let output_config = output_device.default_output_config().unwrap();

    let buf: AudioBuf = Arc::new(Mutex::new(Vec::new()));
    let buf_clone = Arc::clone(&buf);

    let input_stream = create_input_stream(input_device, &input_config.into(), buf_clone);
    let output_stream = create_output_stream(output_device, &output_config.into(), buf, shit_mic);

    input_stream.play().unwrap();
    output_stream.play().unwrap();

    (input_stream, output_stream)
}

impl App {
    pub(super) fn play_audio(&mut self, audio: Audio, randomly_triggered: bool) {
        let path = Path::new(&audio.path);
        let file0 = File::open(path).unwrap();
        let file1 = File::open(path).unwrap();

        let source0 = Decoder::try_from(file0).unwrap();
        let source1 = Decoder::try_from(file1).unwrap();
        self.audio_meta.duration = source0.total_duration().unwrap_or_default();

        self.sinks.0.clear();
        self.sinks.1.clear();

        self.sinks.0.append(source0);
        self.sinks.1.append(source1);

        self.sinks.0.set_volume(audio.volume);
        self.sinks.1.set_volume(audio.volume);

        self.sinks.0.play();
        self.sinks.1.play();

        self.audio = Some(audio);
        self.audio_meta.randomly_triggered = randomly_triggered;
    }
}
