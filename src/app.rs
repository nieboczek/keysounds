use crate::app::audio::{AudioDecoder, FilterChain};
use cpal::traits::{DeviceTrait, HostTrait};
use rand::rngs::ThreadRng;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

mod audio;
mod config;
mod imp;
mod input;
mod widget;

pub struct App {
    _keep_alive: audio::KeepAlive,
    action_channel: Arc<Mutex<Action>>,
    list_state: ListState,
    shit_mic: Arc<AtomicBool>,
    random_sfx_triggering: bool,
    rat_deadline: Instant,
    mode: Mode,
    sfx_data: Option<SfxData>,
    decoder: Arc<Mutex<Option<AudioDecoder>>>,
    input: String,
    config: Config,
    rng: ThreadRng,
    filter_chain: Arc<Mutex<FilterChain>>,
    #[cfg(feature = "render_call_counter")]
    render_call_counter: u32,
}

#[derive(Serialize, Deserialize, Clone)]
struct Sfx {
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

#[derive(Serialize, Deserialize)]
enum AudioFilter {
    #[serde(rename = "boost_bass")]
    BoostBass { gain: f32, cutoff: f32 },
    #[serde(rename = "shittify")]
    Shittify,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    input_device: String,
    output_device: String,
    rat_range: (f32, f32),
    rat_sfx_list: Vec<String>,
    mic_filters: Vec<AudioFilter>,
    sfx: Vec<Sfx>,
}

pub enum Action {
    SearchAndPlay,
    SkipToPart,
    StopSfx,
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
    SearchSfx,
    EditConfig,

    EditInputDevice,
    EditOutputDevice,
    EditSfxs,

    SelectedSfxName,
    SelectedSfxPath,
    SelectedSfxVolume,
    SelectedSfxSkipTo,
    EditSfxName,
    EditSfxPath,
    EditSfxVolume,
    EditSfxSkipTo,
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
    #[inline]
    pub fn new(action_channel: Arc<Mutex<Action>>) -> App {
        let config = Self::load_config_result();
        let host = cpal::default_host();

        let mic_device = host
            .input_devices()
            .unwrap()
            .find(|device| device.name().unwrap_or_default() == config.input_device)
            .expect("Could not find input device");

        let default_out_device = host
            .default_output_device()
            .expect("No output devices present");

        let out_device = host
            .output_devices()
            .unwrap()
            .find(|device| device.name().unwrap_or_default() == config.output_device)
            .expect("Output device not found");

        let decoder = Arc::new(Mutex::new(None));

        let (filter_chain, keep_alive) = Self::create_streams(
            &mic_device,
            &default_out_device,
            &out_device,
            Arc::clone(&decoder),
        );

        filter_chain
            .lock()
            .unwrap()
            .sync_with_config(&config.mic_filters);

        let shit_mic = Arc::new(AtomicBool::new(false));

        App {
            _keep_alive: keep_alive,
            action_channel,
            list_state: ListState::default().with_selected(Some(0)),
            shit_mic,
            random_sfx_triggering: false,
            rat_deadline: Instant::now(),
            mode: Mode::Normal,
            sfx_data: None,
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
