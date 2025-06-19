use rodio::buffer::SamplesBuffer;
use std::{fs::File, thread, time::Duration};
use symphonia::{
    core::{
        audio::{AudioBufferRef, Signal},
        codecs::DecoderOptions,
        formats::{SeekMode, SeekTo},
        io::MediaSourceStream,
        units::Time,
    },
    default::{get_codecs, get_probe},
};

use crate::app::AudioState;

// TODO: replace that std::thread::sleep with a smarter thing.
// TODO: make this more reliable for something else than mp3 files.
pub(crate) fn load_and_play(path: &str, state: AudioState) {
    let file = File::open(path).expect("Failed to open file");
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let probed = get_probe()
        .format(
            &Default::default(),
            mss,
            &Default::default(),
            &Default::default(),
        )
        .expect("Failed to probe format");

    let mut format = probed.format;
    let track = format.default_track().expect("No default track");
    let track_id = track.id;

    let mut decoder = get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .expect("Failed to make decoder");

    while let Ok(packet) = format.next_packet() {
        if let Ok(decoded) = decoder.decode(&packet) {
            let spec = decoded.spec();
            let channels = spec.channels.count() as u16;
            let sample_rate = spec.rate;

            let mut interleaved = Vec::new();

            match decoded {
                AudioBufferRef::S16(buf) => {
                    for i in 0..buf.frames() {
                        for ch in 0..channels {
                            interleaved.push(buf.chan(ch as usize)[i]);
                        }
                    }
                }
                AudioBufferRef::F32(buf) => {
                    for i in 0..buf.frames() {
                        for ch in 0..channels {
                            let sample = buf.chan(ch as usize)[i];
                            let clamped = (sample * i16::MAX as f32)
                                .clamp(i16::MIN as f32, i16::MAX as f32)
                                as i16;
                            interleaved.push(clamped);
                        }
                    }
                }
                _ => panic!("Unsupported audio format"),
            }

            let source = SamplesBuffer::new(channels, sample_rate, interleaved);

            if let Ok(state) = state.lock().as_mut() {
                // check for end suffering flag
                if state.should_stop() {
                    state.reset();
                    break;
                } else if state.should_skip() {
                    let seconds = state.skip_to as u64;
                    let time = Time {
                        seconds: seconds,
                        frac: state.skip_to - seconds as f64,
                    };

                    format
                        .seek(
                            SeekMode::Coarse,
                            SeekTo::Time {
                                time: time,
                                track_id: Some(track_id),
                            },
                        )
                        .expect("Seeking failed");

                    decoder.reset();
                    state.reset();
                    state.sink1.clear();
                    state.sink2.clear();

                    state.sink1.append(source.clone());
                    state.sink2.append(source);

                    state.sink1.play();
                    state.sink2.play();
                    continue;
                }

                state.sink1.append(source.clone());
                state.sink2.append(source);
            }

            // 1152.0 being samples per frame, 1 packet should have 1 frame
            let secs_per_frame = 1152.0 / sample_rate as f64;
            // -0.002 (-2ms), because we can't trust technology
            thread::sleep(Duration::from_secs_f64(secs_per_frame - 0.002));
        }
    }
}
