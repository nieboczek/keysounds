use rand::rngs::ThreadRng;
use ratatui::widgets::ListState;
use rodio::{
    DeviceTrait, OutputStream, OutputStreamBuilder, Sink,
    cpal::{self, Stream, traits::HostTrait},
};
use serde::{Deserialize, Serialize};
use std::{
    ops::BitOrAssign,
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::{Duration, Instant},
};

mod audio;
mod config;
mod imp;
mod widget;

pub(crate) struct App {
    _keep_alive: (OutputStream, OutputStream, (Stream, Stream)),
    channel: Arc<Mutex<Action>>,
    state: ListState,
    shit_mic: Arc<AtomicBool>,
    random_audio_triggering: bool,
    rat_deadline: Instant,
    mode: Mode,
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
struct Audio {
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
const fn is_default_volume(volume: &f32) -> bool {
    *volume == 1.0
}

#[inline]
const fn is_skip_to_default(skip_to: &f32) -> bool {
    *skip_to == 0.0
}

struct AudioMeta {
    randomly_triggered: bool,
    duration: Duration,
}

impl AudioMeta {
    #[inline]
    fn reset() -> AudioMeta {
        AudioMeta {
            randomly_triggered: false,
            duration: Duration::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    input_device: String,
    output_device: String,
    rat_range: (f32, f32),
    rat_audio_list: Vec<String>,
    audios: Vec<Audio>,
}

pub(crate) enum Action {
    SearchAndPlay,
    SkipToPart,
    StopAudio,
    ToggleShitMic,
    None,
}

#[derive(PartialEq)]
enum StateStatus {
    Unaffected,
    IdleRender,
    Updated,
    IgnoreNextKeyPress,
    Quit,
}

#[derive(PartialEq)]
enum Mode {
    Null,

    Normal,
    SearchAudio,
    EditConfig,

    EditInputDevice,
    EditOutputDevice,
    EditAudios,

    SelectedAudioName,
    SelectedAudioPath,
    SelectedAudioVolume,
    SelectedAudioSkipTo,
    EditAudioName,
    EditAudioPath,
    EditAudioVolume,
    EditAudioSkipTo,
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
        let config = Self::load_config_result();
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
            mode: Mode::Normal,
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
