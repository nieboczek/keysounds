use crate::app::{Action, App, Sound};
use rdev::Key;
use serde::{Deserialize, Serialize};
use std::{
    fs::{read_to_string, write},
    io::ErrorKind,
    path::PathBuf,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFilter {
    BoostBass {
        gain: f32,
        cutoff: f32,
    },
    Shittify {
        strength: i32,
        cutoff: i32,
    },
    Reverb {
        room_size: f32,
        damping: f32,
        wet: f32,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Keybind {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub key: Key,
    pub action: Action,
}

impl Keybind {
    pub fn default_keybind(key: Key, action: Action) -> Self {
        Keybind {
            shift: false,
            ctrl: true,
            alt: true,
            key,
            action,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub input_device: String,
    pub output_device: String,
    pub virtual_output_device: String,
    pub sound_triggering_interval_range: (f32, f32),
    pub sound_triggering_sound_list: Vec<String>,
    pub keybinds: Vec<Keybind>,
    pub sounds: Vec<Sound>,
}

impl App {
    pub fn load_config(&mut self) {
        self.config = Self::load_config_result();
        *self.action_channel.lock().unwrap() = Action::SetKeybinds(self.config.keybinds.clone());
    }

    pub fn save_config(&self) {
        Self::save_config_result(&self.config);
    }

    pub fn load_config_result() -> Config {
        let contents = match read_to_string(Self::config_file()) {
            Ok(contents) => contents,
            Err(err) => {
                if err.kind() != ErrorKind::NotFound {
                    panic!("Couldn't read the config file: {err}");
                }

                let config = Config {
                    input_device: String::new(),
                    output_device: String::new(),
                    virtual_output_device: String::from("CABLE Input (VB-Audio Virtual Cable)"),
                    sound_triggering_interval_range: (600.0, 900.0),
                    sound_triggering_sound_list: Vec::new(),
                    keybinds: vec![
                        Keybind::default_keybind(Key::KeyT, Action::SearchAndPlay),
                        Keybind::default_keybind(Key::KeyS, Action::StopSound),
                    ],
                    sounds: Vec::new(),
                };

                Self::save_config_result(&config);
                return config;
            }
        };
        toml::from_str::<Config>(&contents).unwrap()
    }

    pub fn save_config_result(config: &Config) {
        let contents = toml::to_string(config).unwrap();
        write(Self::config_file(), contents).unwrap();
    }

    fn config_file() -> PathBuf {
        let mut dir = dirs_next::config_dir().unwrap();
        dir.push("keysounds/config.toml");
        dir
    }
}
