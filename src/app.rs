use crate::app::{
    audio::{AudioDecoder, FilterChain},
    config::{AudioFilter, Config, Keybind},
};
use cpal::traits::{DeviceTrait, HostTrait};
use iced::widget::svg;
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex, atomic::AtomicU64},
    time::{Duration, Instant},
};

pub mod audio;
pub mod config;
pub mod gui;

pub struct App {
    _keep_alive: audio::KeepAlive,
    action_channel: Arc<Mutex<Action>>,
    random_sfx_triggering: bool,
    rst_deadline: Instant,
    sfx_data: Option<SfxData>,
    target_sample_rate: u32,
    decoder: Arc<Mutex<Option<AudioDecoder>>>,
    decoder_pos: Arc<AtomicU64>,
    config: Config,
    rng: ThreadRng,
    filter_chain: Arc<Mutex<FilterChain>>,

    // GUI
    svgs: Svgs,
    page: Page,
    search: String,
    selected_sfx: Option<usize>,
    editing_sfx: Option<usize>,
    settings_open: bool,
}

pub struct Svgs {
    stop: svg::Handle,
}

#[derive(PartialEq, Eq)]
pub enum Page {
    Sounds,
    Microphone,
    RandomSfx,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Sfx {
    name: String,
    path: String,
    #[serde(default = "default_volume", skip_serializing_if = "is_default_volume")]
    volume: f32,
}

#[inline]
const fn default_volume() -> f32 {
    1.0
}

#[inline]
const fn is_default_volume(volume: &f32) -> bool {
    *volume == 1.0
}

struct SfxData {
    randomly_triggered: bool,
    duration: Duration,
    sfx: Sfx,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    SearchAndPlay,
    StopSfx,
    FilterPreset(Vec<AudioFilter>),
    SetKeybinds(Vec<Keybind>),
    None,
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
        let decoder_pos = Arc::new(AtomicU64::new(u64::MAX));
        let host = cpal::default_host();

        let Some(mic_device) = host.input_devices().unwrap().find(|device| {
            device
                .description()
                .is_ok_and(|desc| desc.name() == config.input_device)
        }) else {
            panic!(
                "Could not find input device in list:\n{:?}",
                host.input_devices()
                    .unwrap()
                    .map(|d| d.description().unwrap().name().to_string())
                    .collect::<Vec<_>>()
            );
        };

        let out_device = host
            .default_output_device()
            .expect("No output devices present");

        let Some(virtual_out_device) = host.output_devices().unwrap().find(|device| {
            device
                .description()
                .is_ok_and(|desc| desc.name() == config.virtual_output_device)
                || device
                    .id()
                    .is_ok_and(|id| id.id() == config.virtual_output_device)
        }) else {
            panic!(
                "Could not find output device '{}' in list:\n{:?}",
                config.virtual_output_device,
                host.output_devices()
                    .unwrap()
                    .map(|d| format!("{} ({})", d.description().unwrap().name(), d.id().unwrap()))
                    .collect::<Vec<_>>()
            );
        };

        let (filter_chain, sample_rate, keep_alive) = Self::create_streams(
            &mic_device,
            &out_device,
            &virtual_out_device,
            Arc::clone(&decoder),
            Arc::clone(&decoder_pos),
        );

        App {
            _keep_alive: keep_alive,
            action_channel,
            random_sfx_triggering: false,
            rst_deadline: Instant::now(),
            sfx_data: None,
            target_sample_rate: sample_rate,
            decoder,
            decoder_pos,
            config,
            rng: rand::rng(),
            filter_chain,

            svgs: Svgs {
                stop: svg::Handle::from_memory(include_bytes!("../assets/stop.svg")),
            },
            page: Page::Sounds,
            search: String::new(),
            selected_sfx: None,
            editing_sfx: None,
            settings_open: false,
        }
    }
}
