use crate::app::{App, Config};
use std::{
    fs::{read_to_string, write},
    io::ErrorKind,
    path::PathBuf,
};

impl App {
    pub(crate) fn load_config(&mut self) {
        self.config = Self::load_config_result();
    }

    pub(crate) fn save_config(&self) {
        Self::save_config_result(&self.config);
    }

    pub(crate) fn load_config_result() -> Config {
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
                    rat_audio_list: Vec::new(),
                    audios: Vec::new(),
                };

                Self::save_config_result(&config);
                return config;
            }
        };
        toml::from_str::<Config>(&contents).unwrap()
    }

    pub(crate) fn save_config_result(config: &Config) {
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
