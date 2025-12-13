use crate::app::{Action, App, Sfx};
use rdev::Key;
use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, write};
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub enum AudioFilter {
    #[serde(rename = "boost_bass")]
    BoostBass { gain: f32, cutoff: f32 },
    #[serde(rename = "shittify")]
    Shittify,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Keybind {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub key: rdev::Key,
    pub action: Action,
}

impl Keybind {
    pub fn default_keybind(key: rdev::Key, action: Action) -> Self {
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
    pub rat_range: (f32, f32),
    pub rat_sfx_list: Vec<String>,
    pub keybinds: Vec<Keybind>,
    pub sfx: Vec<Sfx>,
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
        let contents = match read_to_string(Self::get_config_file()) {
            Ok(contents) => contents,
            Err(err) => {
                if err.kind() != ErrorKind::NotFound {
                    panic!("Couldn't read the config file: {err}");
                }

                let config = Config {
                    input_device: String::new(),
                    output_device: String::from("CABLE Input (VB-Audio Virtual Cable)"),
                    rat_range: (600.0, 900.0),
                    rat_sfx_list: Vec::new(),
                    keybinds: vec![
                        Keybind::default_keybind(Key::KeyT, Action::SearchAndPlay),
                        Keybind::default_keybind(Key::KeyY, Action::SkipToPart),
                        Keybind::default_keybind(Key::KeyS, Action::StopSfx),
                        Keybind::default_keybind(
                            Key::KeyG,
                            Action::FilterPreset(vec![AudioFilter::Shittify]),
                        ),
                    ],
                    sfx: Vec::new(),
                };

                Self::save_config_result(&config);
                return config;
            }
        };
        toml::from_str::<Config>(&contents).unwrap()
    }

    pub fn save_config_result(config: &Config) {
        let contents = toml::to_string(config).unwrap();
        write(Self::get_config_file(), contents).unwrap();
    }

    fn get_config_file() -> PathBuf {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("config.toml")
    }
}
