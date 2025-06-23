use std::time::Duration;

use crossbeam_channel::Receiver;
use ratatui::widgets::ListState;
use rodio::{
    cpal::{self, traits::HostTrait, Stream},
    DeviceTrait, OutputStream, OutputStreamHandle, Sink,
};
use serde::{Deserialize, Serialize};

mod audio;
pub(crate) mod config;
mod implementation;
mod widget;

struct AudioMeta {
    pub(crate) randomly_triggered: bool,
    pub(crate) duration: Duration,
}

impl AudioMeta {
    pub(crate) fn reset() -> AudioMeta {
        AudioMeta {
            randomly_triggered: false,
            duration: Duration::default(),
        }
    }
}

pub(crate) struct App {
    _keep_alive: (
        OutputStream,
        OutputStreamHandle,
        OutputStream,
        OutputStreamHandle,
        (Stream, Stream),
    ),
    receiver: Receiver<Action>,
    state: ListState,
    shit_mic: bool,
    random_audio_triggering: bool,
    inputting: bool,
    audio_meta: AudioMeta,
    audio: Option<Audio>,
    input: String,
    config: Config,
    sinks: (Sink, Sink),
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Audio {
    name: String,
    path: String,
    #[serde(default = "default_volume", skip_serializing_if = "is_default_volume")]
    volume: f32,
    #[serde(default, skip_serializing_if = "is_skip_to_default")]
    skip_to: f32,
}

fn default_volume() -> f32 {
    1.0
}

fn is_default_volume(volume: &f32) -> bool {
    *volume == 1.0
}

fn is_skip_to_default(skip_to: &f32) -> bool {
    *skip_to == 0.0
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) input_device: String,
    pub(crate) output_device: String,
    pub(crate) rat_range: (u32, u32),
    pub(crate) rat_audio_list: Vec<String>,
    pub(crate) audios: Vec<Audio>,
}

pub(crate) enum Action {
    SearchAndPlay,
    SkipToPart,
    StopAudio,
    ToggleShitMic,
}

impl App {
    pub(crate) fn new(receiver: Receiver<Action>) -> App {
        let config = config::read_config();
        let host = cpal::default_host();

        // Default Output Sink
        let (_s1, _sh1) =
            OutputStream::try_default().expect("A default output stream should be created");
        let default_sink = Sink::try_new(&_sh1).expect("Failed to create sink");

        // Virtual Output Sink
        let output_device = cpal::default_host()
            .output_devices()
            .unwrap()
            .find(|device| device.name().unwrap_or_default() == config.output_device)
            .expect("Virtual cable output device not found");

        let (_s2, _sh2) = OutputStream::try_from_device(&output_device)
            .expect("Failed to open cable output stream");
        let output_sink = Sink::try_new(&_sh2).expect("Failed to create cable sink");

        // Microphone Device
        let input_device = host
            .input_devices()
            .unwrap()
            .find(|device| device.name().unwrap_or_default() == config.input_device)
            .expect("Could not find input device");

        // Start microphone forwarding to virtual output
        let _ss = audio::forward_input(input_device, output_device);

        App {
            _keep_alive: (_s1, _sh1, _s2, _sh2, _ss),
            receiver,
            state: ListState::default().with_selected(Some(0)),
            shit_mic: false,
            random_audio_triggering: false,
            inputting: false,
            audio_meta: AudioMeta::reset(),
            audio: None,
            input: String::new(),
            config,
            sinks: (default_sink, output_sink),
        }
    }
}
