use crossbeam_channel::Receiver;
use ratatui::widgets::ListState;
use rodio::{cpal::traits::HostTrait, DeviceTrait, OutputStream, OutputStreamHandle, Sink};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

pub(crate) mod audio;
pub(crate) mod config;
pub(crate) mod implementation;
pub(crate) mod widget;

pub(crate) struct AudioStateInner {
    sink1: Sink,
    sink2: Sink,
    stop: bool,
    skip_to: f64,
}

impl AudioStateInner {
    pub(crate) fn should_skip(&self) -> bool {
        self.skip_to > -0.5
    }

    pub(crate) fn complete_skip(&mut self) {
        self.skip_to = -1.0;
    }
}

type AudioState = Arc<Mutex<AudioStateInner>>;

pub(crate) struct App {
    _keep_alive: (
        OutputStream,
        OutputStreamHandle,
        OutputStream,
        OutputStreamHandle,
    ),
    play_handle: Option<JoinHandle<()>>,
    receiver: Receiver<Action>,
    state: ListState,
    shit_mic: bool,
    random_audio_triggering: bool,
    config: Config,
    audio_state: AudioState,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Audio {
    name: String,
    path: String,
    skip_to: f64,
    volume: f64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
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

        let (stream1, stream_handle1) =
            OutputStream::try_default().expect("Default output stream not found");
        let sink1 = Sink::try_new(&stream_handle1).expect("Failed to create sink");

        let device2 = rodio::cpal::default_host()
            .output_devices()
            .unwrap()
            .find(|d| d.name().unwrap() == config.output_device)
            .expect("Virtual cable output device not found");

        let (stream2, stream_handle2) =
            OutputStream::try_from_device(&device2).expect("Failed to open cable output stream");
        let sink2 = Sink::try_new(&stream_handle2).expect("Failed to create cable sink");

        App {
            _keep_alive: (stream1, stream_handle1, stream2, stream_handle2),
            play_handle: None,
            receiver,
            state: ListState::default().with_selected(Some(0)),
            shit_mic: false,
            random_audio_triggering: false,
            config,
            audio_state: Arc::new(Mutex::new(AudioStateInner {
                sink1,
                sink2,
                stop: false,
                skip_to: -1.0,
            })),
        }
    }
}
