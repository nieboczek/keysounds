use crate::app::{
    audio::{AudioDecoder, FilterChain},
    config::{AudioFilter, Config, Keybind},
};
use cpal::traits::{DeviceTrait, HostTrait};
use rand::rngs::ThreadRng;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub mod audio;
pub mod config;
pub mod imp;
pub mod input;
pub mod widget;
pub use input::Mode;

pub struct App {
    _keep_alive: audio::KeepAlive,
    action_channel: Arc<Mutex<Action>>,
    list_state: ListState,
    random_sfx_triggering: bool,
    rst_deadline: Instant,
    mode: Mode,
    sfx_data: Option<SfxData>,
    target_sample_rate: u32,
    decoder: Arc<Mutex<Option<AudioDecoder>>>,
    input: String,
    config: Config,
    rng: ThreadRng,
    filter_chain: Arc<Mutex<FilterChain>>,
    #[cfg(feature = "render_call_counter")]
    render_call_counter: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Sfx {
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

struct SfxData {
    randomly_triggered: bool,
    duration: Duration,
    sfx: Sfx,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "search_and_play")]
    SearchAndPlay,
    #[serde(rename = "skip_to_part")]
    SkipToPart,
    #[serde(rename = "stop_sfx")]
    StopSfx,
    #[serde(rename = "filter_preset")]
    FilterPreset(Vec<AudioFilter>),

    #[serde(rename = "set_keybinds")]
    SetKeybinds(Vec<Keybind>),
    #[serde(rename = "none")]
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

impl std::ops::BitOrAssign for StateStatus {
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
    pub fn device_desc_to_name(desc: cpal::DeviceDescription) -> String {
        format!("{} ({})", desc.name(), desc.driver().unwrap())
    }

    #[inline]
    pub fn new(action_channel: Arc<Mutex<Action>>) -> App {
        let config = Self::load_config_result();
        *action_channel.lock().unwrap() = Action::SetKeybinds(config.keybinds.clone());

        let decoder = Arc::new(Mutex::new(None));
        let host = cpal::default_host();

        let Some(mic_device) = host.input_devices().unwrap().find(|device| {
            device
                .description()
                .is_ok_and(|desc| Self::device_desc_to_name(desc) == config.input_device)
        }) else {
            panic!(
                "Could not find input device in list:\n{:?}",
                host.input_devices()
                    .unwrap()
                    .map(|d| Self::device_desc_to_name(d.description().unwrap()))
                    .collect::<Vec<_>>()
            );
        };

        let out_device = host
            .default_output_device()
            .expect("No output devices present");

        let Some(virtual_out_device) = host.output_devices().unwrap().find(|device| {
            device
                .description()
                .is_ok_and(|desc| Self::device_desc_to_name(desc) == config.virtual_output_device)
        }) else {
            panic!(
                "Could not find output device in list:\n{:?}",
                host.output_devices()
                    .unwrap()
                    .map(|d| Self::device_desc_to_name(d.description().unwrap()))
                    .collect::<Vec<_>>()
            );
        };

        let (filter_chain, sample_rate, keep_alive) = Self::create_streams(
            &mic_device,
            &out_device,
            &virtual_out_device,
            Arc::clone(&decoder),
        );

        App {
            _keep_alive: keep_alive,
            action_channel,
            list_state: ListState::default().with_selected(Some(0)),
            random_sfx_triggering: false,
            rst_deadline: Instant::now(),
            mode: Mode::Normal,
            sfx_data: None,
            target_sample_rate: sample_rate,
            decoder,
            input: String::new(),
            config,
            rng: rand::rng(),
            filter_chain,
            #[cfg(feature = "render_call_counter")]
            render_call_counter: 0,
        }
    }
}
