use rodio::{
    cpal::{
        traits::{DeviceTrait, StreamTrait},
        Device, FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig,
    },
    Decoder, Source,
};
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};

use crate::App;

type AudioBuf = Arc<Mutex<Vec<f32>>>;

fn create_input_stream<T>(device: &Device, config: &StreamConfig, buf: AudioBuf) -> Stream
where
    T: Sample + SizedSample,
    f32: FromSample<T>,
{
    let channels = config.channels as usize;

    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let mut buf = buf.lock().unwrap();

                for &sample in data.iter() {
                    let sample: f32 = Sample::from_sample(sample);
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

fn create_output_stream<T>(device: &Device, config: &StreamConfig, buf: AudioBuf) -> Stream
where
    T: Sample + SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;

    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                let mut buffer = buf.lock().unwrap();

                for frame in data.chunks_mut(channels) {
                    let len = buffer.len();
                    if len >= channels {
                        // Take samples from buffer
                        for (i, sample_out) in frame.iter_mut().enumerate() {
                            if i < len {
                                let sample: T = Sample::from_sample(buffer[i]);
                                *sample_out = sample;
                            }
                        }

                        // Remove used samples
                        buffer.drain(0..channels.min(len));
                    } else {
                        // No audio available, output silence
                        for sample_out in frame.iter_mut() {
                            *sample_out = Sample::from_sample(0.0f32);
                        }
                    }
                }
            },
            |err| eprintln!("Output stream error: {err}"),
            None,
        )
        .unwrap()
}

pub(crate) fn forward_input(input_device: Device, output_device: Device) -> (Stream, Stream) {
    let input_config = input_device.default_input_config().unwrap();
    let output_config = output_device.default_output_config().unwrap();

    let buf: AudioBuf = Arc::new(Mutex::new(Vec::new()));
    let buf_clone = Arc::clone(&buf);

    let input_stream = match input_config.sample_format() {
        SampleFormat::F32 => {
            create_input_stream::<f32>(&input_device, &input_config.into(), buf_clone)
        }
        _ => panic!("Formats other than F32 are not supported on input streams"),
    };

    let output_stream = match output_config.sample_format() {
        SampleFormat::F32 => {
            create_output_stream::<f32>(&output_device, &output_config.into(), buf)
        }
        _ => panic!("Formats other than F32 are not supported on output streams"),
    };

    input_stream.play().unwrap();
    output_stream.play().unwrap();

    (input_stream, output_stream)
}

impl App {
    #[inline]
    pub(crate) fn play_file(&mut self, path: &str, volume: f32) {
        let file0 = File::open(path).unwrap();
        let file1 = File::open(path).unwrap();

        let source0 = Decoder::new(BufReader::new(file0)).unwrap();
        let source1 = Decoder::new(BufReader::new(file1)).unwrap();

        self.audio_meta.duration = source0.total_duration().unwrap_or_default();

        self.sinks.0.append(source0);
        self.sinks.1.append(source1);

        self.sinks.0.set_volume(volume);
        self.sinks.1.set_volume(volume);
    }
}
