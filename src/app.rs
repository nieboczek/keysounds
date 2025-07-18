use ratatui::widgets::ListState;
use rodio::{
    cpal::{self, traits::HostTrait, Stream},
    DeviceTrait, OutputStream, OutputStreamBuilder, Sink,
};
use serde::{Deserialize, Serialize};
use std::{
    ops::BitOrAssign,
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::Duration,
};

mod audio;
pub(crate) mod config;
mod imp;
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
    _keep_alive: (OutputStream, OutputStream, (Stream, Stream)),
    channel: Arc<Mutex<Action>>,
    state: ListState,
    shit_mic: Arc<AtomicBool>,
    random_audio_triggering: bool,
    inputting: bool,
    audio_meta: AudioMeta,
    audio: Option<Audio>,
    input: String,
    config: Config,
    #[cfg(feature = "render_call_counter")]
    render_call_counter: u32,
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
    None,
}

#[derive(PartialEq, Debug)]
pub(crate) enum StateStatus {
    Unaffected,
    IdleRender,
    Updated,
    IgnoreNextKeyPress,
    Quit,
}

impl BitOrAssign for StateStatus {
    /// Updates the state without overwriting more important statuses like `StateStatus::Quit`.
    fn bitor_assign(&mut self, rhs: Self) {
        match rhs {
            StateStatus::Unaffected => {}
            StateStatus::IdleRender => {
                if *self == StateStatus::Unaffected {
                    *self = StateStatus::IdleRender;
                }
            }
            StateStatus::Updated => {
                if matches!(self, StateStatus::Unaffected | StateStatus::IdleRender) {
                    *self = StateStatus::Updated;
                }
            }
            StateStatus::IgnoreNextKeyPress => {
                if *self != StateStatus::Quit {
                    *self = StateStatus::IgnoreNextKeyPress;
                }
            }
            StateStatus::Quit => *self = StateStatus::Quit,
        }
    }
}

impl App {
    pub(crate) fn new(channel: Arc<Mutex<Action>>) -> App {
        let config = config::load_config();
        let host = cpal::default_host();

        // Default Output Sink
        let default_out = OutputStreamBuilder::open_default_stream().unwrap();
        let default_sink = Sink::connect_new(default_out.mixer());

        // Virtual Output Sink
        let virtual_device = cpal::default_host()
            .output_devices()
            .unwrap()
            .find(|device| device.name().unwrap_or_default() == config.output_device)
            .expect("Virtual cable output device not found");

        let virtual_out = OutputStreamBuilder::from_device(virtual_device.clone())
            .unwrap()
            .open_stream()
            .unwrap();
        let virtual_sink = Sink::connect_new(virtual_out.mixer());

        // Microphone Device
        let microphone_device = host
            .input_devices()
            .unwrap()
            .find(|device| device.name().unwrap_or_default() == config.input_device)
            .expect("Could not find input device");

        // Shit Mic Mode initialized to false
        let shit_mic = Arc::new(AtomicBool::default());

        // Start microphone forwarding to virtual output
        let _ss = audio::forward_input(microphone_device, virtual_device, Arc::clone(&shit_mic));

        App {
            _keep_alive: (default_out, virtual_out, _ss),
            channel,
            state: ListState::default().with_selected(Some(0)),
            shit_mic,
            random_audio_triggering: false,
            inputting: false,
            audio_meta: AudioMeta::reset(),
            audio: None,
            #[cfg(feature = "render_call_counter")]
            render_call_counter: 0,
            input: String::new(),
            config,
            sinks: (default_sink, virtual_sink),
        }
    }
}
