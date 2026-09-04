use crate::app::{App, Sound, config::filter::AudioFilter};
use serde::{Deserialize, Serialize};
use std::{
    fs::{read_to_string, write},
    io::ErrorKind,
    path::PathBuf,
};

pub mod filter;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Keybind {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: rdev::Key,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FilterPreset {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keybind: Option<Keybind>,
    pub filters: Vec<AudioFilter>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub input_device: String,
    pub output_device: String,
    pub virtual_output_device: String,
    pub gui_scale: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_and_play_keybind: Option<Keybind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_sound_keybind: Option<Keybind>,
    pub filter_presets: Vec<FilterPreset>,
    pub sounds: Vec<Sound>,
}

impl App {
    pub fn load_config(&mut self) {
        self.config = Self::load_config_result();
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
                    gui_scale: 1.0,
                    search_and_play_keybind: Some(Keybind {
                        ctrl: true,
                        alt: true,
                        shift: false,
                        key: rdev::Key::KeyT,
                    }),
                    stop_sound_keybind: Some(Keybind {
                        ctrl: true,
                        alt: true,
                        shift: false,
                        key: rdev::Key::KeyS,
                    }),
                    filter_presets: Vec::new(),
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
        let mut dir = dirs::config_dir().unwrap();
        dir.push("keysounds/config.toml");
        dir
    }
}

mod keybind_serde {
    use crate::app::{config::Keybind, keybind_listener};
    use serde::{Deserialize, Serialize};
    use std::{fmt, str::FromStr};

    impl fmt::Display for Keybind {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.ctrl {
                write!(f, "Ctrl+")?;
            }
            if self.alt {
                write!(f, "Alt+")?;
            }
            if self.shift {
                write!(f, "Shift+")?;
            }
            keybind_listener::write_key_str(self.key, f)
        }
    }

    impl FromStr for Keybind {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let parts = s.split('+').map(|p| p.trim());
            let mut ctrl = false;
            let mut alt = false;
            let mut shift = false;
            let mut key = None;

            for part in parts {
                if part.is_empty() {
                    continue;
                }

                let part = part.to_ascii_lowercase();
                if part == "ctrl" {
                    if ctrl {
                        return Err("Ctrl was already specified".to_string());
                    }
                    ctrl = true;
                } else if part == "alt" {
                    if alt {
                        return Err("Alt was already specified".to_string());
                    }
                    alt = true;
                } else if part == "shift" {
                    if shift {
                        return Err("Shift was already specified".to_string());
                    }
                    shift = true;
                } else {
                    if key.is_some() {
                        return Err(format!("Multiple keys specified: {s}"));
                    }
                    key = Some(keybind_listener::parse_key(&part)?);
                }
            }

            let key = key.ok_or_else(|| format!("No key specified: {s}"))?;
            Ok(Keybind {
                ctrl,
                alt,
                shift,
                key,
            })
        }
    }

    impl Serialize for Keybind {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for Keybind {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            s.parse().map_err(serde::de::Error::custom)
        }
    }
}
