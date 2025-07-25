use rand::rngs::ThreadRng;
use ratatui::widgets::ListState;
use rodio::{
    cpal::{self, traits::HostTrait, Stream},
    DeviceTrait, OutputStream, OutputStreamBuilder, Sink,
};
use serde::{Deserialize, Serialize};
use std::{
    ops::BitOrAssign,
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::{Duration, Instant},
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
    #[inline]
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
    rat_deadline: Instant,
    inputting: bool,
    audio_meta: AudioMeta,
    audio: Option<Audio>,
    input: String,
    config: Config,
    rng: ThreadRng,
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

#[inline]
const fn default_volume() -> f32 {
    1.0
}

#[inline]
fn is_default_volume(volume: &f32) -> bool {
    *volume == 1.0
}

#[inline]
fn is_skip_to_default(skip_to: &f32) -> bool {
    *skip_to == 0.0
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) input_device: String,
    pub(crate) output_device: String,
    pub(crate) rat_range: (f32, f32),
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
    #[inline]
    pub(crate) fn new(channel: Arc<Mutex<Action>>) -> App {
        let config = config::load_config();
        let host = cpal::default_host();

        // Microphone Device
        let microphone_device = host
            .input_devices()
            .unwrap()
            .find(|device| device.name().unwrap_or_default() == config.input_device)
            .expect("Could not find input device");

        // Virtual Device
        let virtual_device = cpal::default_host()
            .output_devices()
            .unwrap()
            .find(|device| device.name().unwrap_or_default() == config.output_device)
            .expect("Virtual cable output device not found");

        // Shit Mic Mode initialized to false
        let shit_mic = Arc::new(AtomicBool::default());

        // Start microphone forwarding to virtual output
        let streams =
            audio::forward_input(&microphone_device, &virtual_device, Arc::clone(&shit_mic));

        // Default Output Sink
        let mut default_out = OutputStreamBuilder::open_default_stream().unwrap();
        let default_sink = Sink::connect_new(default_out.mixer());

        // Virtual Output Sink
        let mut virtual_out = OutputStreamBuilder::from_device(virtual_device)
            .unwrap()
            .open_stream()
            .unwrap();
        let virtual_sink = Sink::connect_new(virtual_out.mixer());

        default_out.log_on_drop(false);
        virtual_out.log_on_drop(false);

        App {
            _keep_alive: (default_out, virtual_out, streams),
            channel,
            state: ListState::default().with_selected(Some(0)),
            shit_mic,
            random_audio_triggering: false,
            rat_deadline: Instant::now(),
            inputting: false,
            audio_meta: AudioMeta::reset(),
            audio: None,
            input: String::new(),
            config,
            rng: rand::rng(),
            #[cfg(feature = "render_call_counter")]
            render_call_counter: 0,
            sinks: (default_sink, virtual_sink),
        }
    }
}
