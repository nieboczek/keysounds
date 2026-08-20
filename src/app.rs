use crate::app::{
    audio::{AudioDecoder, FilterChain},
    config::Config,
    gui::{KeybindTarget, Theme},
    keybind_listener::KeybindListener,
};
use cpal::{
    Device,
    traits::{DeviceTrait, HostTrait},
};
use iced::widget::svg;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex, atomic::AtomicU64},
    time::{Duration, Instant},
};

pub mod audio;
pub mod config;
pub mod gui;
pub mod keybind_listener;

pub struct App {
    _keep_alive: audio::KeepAlive,
    keybind_listener: KeybindListener,
    playing_sound: Option<PlayingSound>,
    target_sample_rate: u32,
    decoder: Arc<Mutex<Option<AudioDecoder>>>,
    decoder_pos: Arc<AtomicU64>,
    config: Config,
    filter_chain: Arc<Mutex<FilterChain>>,

    // GUI - Audio Settings
    input_devices: Vec<DeviceOption>,
    output_devices: Vec<DeviceOption>,
    mic_device: DeviceOption,
    out_device: DeviceOption,
    virtual_out_device: DeviceOption,

    // GUI - Main
    theme: Theme,
    svgs: Svgs,
    page: Page,
    search: String,
    selected_preset: usize,
    recording_keybind: Option<KeybindTarget>,
}

#[derive(Clone, Debug)]
pub struct DeviceOption {
    pub device: Device,
    label: String,
}

impl DeviceOption {
    fn new(device: Device, label: impl ToString) -> Self {
        Self {
            device,
            label: label.to_string(),
        }
    }
}

impl PartialEq for DeviceOption {
    fn eq(&self, other: &Self) -> bool {
        self.device == other.device
    }
}

impl std::fmt::Display for DeviceOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

pub struct Svgs {
    stop: svg::Handle,
    drag_handle: svg::Handle,
    expand_arrow: svg::Handle,
    x: svg::Handle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Sounds,
    FilterChain,
    Settings,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Sound {
    name: String,
    path: String,
    #[serde(
        default = "Sound::default_volume",
        skip_serializing_if = "Sound::is_default_volume"
    )]
    volume: f32,
}

impl Sound {
    #[inline]
    const fn default_volume() -> f32 {
        1.0
    }

    #[inline]
    const fn is_default_volume(volume: &f32) -> bool {
        *volume == Self::default_volume()
    }
}

struct PlayingSound {
    randomly_triggered: bool,
    duration: Duration,
    sound: Sound,
}

impl App {
    fn is_monitor_source(device: &Device) -> bool {
        device.id().is_ok_and(|id| id.id().ends_with(".monitor"))
    }

    fn device_label(device: &Device) -> String {
        device
            .description()
            .map(|desc| desc.name().to_string())
            .unwrap_or_else(|_| device.to_string())
    }

    fn resolve_device(devices: &[DeviceOption], config_name: &str, kind: &str) -> DeviceOption {
        let fallback = || devices.first().cloned().expect("No devices present");
        if config_name == "default" {
            return fallback();
        }

        devices
            .iter()
            .find(|option| {
                option.device.id().is_ok_and(|id| id.id() == config_name)
                    || option
                        .device
                        .description()
                        .is_ok_and(|desc| desc.name() == config_name)
            })
            .cloned()
            .unwrap_or_else(|| {
                tracing::warn!(
                    "{kind} device \"{config_name}\" not found, using the system default"
                );
                fallback()
            })
    }

    #[expect(clippy::new_without_default)]
    pub fn new() -> App {
        let start_instant = Instant::now();
        let config = Self::load_config_result();
        let decoder = Arc::new(Mutex::new(None));
        let decoder_pos = Arc::new(AtomicU64::new(u64::MAX));
        let host = cpal::default_host();

        let default_input = host.default_input_device();
        let default_output = host.default_output_device();

        let mut input_devices = Vec::new();
        if let Some(device) = &default_input {
            input_devices.push(DeviceOption::new(
                device.clone(),
                format!("System Default: {}", Self::device_label(device)),
            ));
        }

        input_devices.extend(
            host.input_devices()
                .unwrap()
                .filter(|device| !Self::is_monitor_source(device))
                .map(|device| DeviceOption::new(device.clone(), Self::device_label(&device))),
        );

        let mut output_devices = Vec::new();
        if let Some(device) = &default_output {
            output_devices.push(DeviceOption::new(
                device.clone(),
                format!("System Default: {}", Self::device_label(device)),
            ));
        }

        output_devices.extend(
            host.output_devices()
                .unwrap()
                .map(|device| DeviceOption::new(device.clone(), Self::device_label(&device))),
        );

        let mic_device = Self::resolve_device(&input_devices, &config.input_device, "Input");
        let out_device = Self::resolve_device(&output_devices, &config.output_device, "Output");

        let virtual_out_device = output_devices
            .iter()
            .find(|option| {
                option
                    .device
                    .id()
                    .is_ok_and(|id| id.id() == config.virtual_output_device)
                    || option
                        .device
                        .description()
                        .is_ok_and(|desc| desc.name() == config.virtual_output_device)
            })
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "Could not find output device '{}' in list:\n{:?}",
                    config.virtual_output_device,
                    output_devices
                        .iter()
                        .map(|option| format!("{} ({})", option.label, option.device.id().unwrap()))
                        .collect::<Vec<_>>()
                );
            });

        let (filter_chain, sample_rate, keep_alive) = Self::create_streams(
            &mic_device.device,
            &out_device.device,
            &virtual_out_device.device,
            Arc::clone(&decoder),
            Arc::clone(&decoder_pos),
        );

        macro_rules! include_svg {
            ($path:literal) => {
                svg::Handle::from_memory(include_bytes!($path))
            };
        }

        let app = App {
            _keep_alive: keep_alive,
            keybind_listener: KeybindListener::new(),
            playing_sound: None,
            target_sample_rate: sample_rate,
            decoder,
            decoder_pos,
            config,
            filter_chain,

            input_devices,
            output_devices,
            mic_device,
            out_device,
            virtual_out_device,

            theme: Theme::default(),
            svgs: Svgs {
                stop: include_svg!("../assets/stop.svg"),
                drag_handle: include_svg!("../assets/drag-handle.svg"),
                expand_arrow: include_svg!("../assets/expand-arrow.svg"),
                x: include_svg!("../assets/x.svg"),
            },
            page: Page::Sounds,
            search: String::new(),
            selected_preset: 0,
            recording_keybind: None,
        };
        tracing::info!("App startup time: {:?}", start_instant.elapsed());
        app
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn gui_scale(&self) -> f32 {
        self.config.gui_scale
    }
}
