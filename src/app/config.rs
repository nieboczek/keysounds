use crate::app::Config;
use std::{
    fs::{read_to_string, write},
    io::ErrorKind,
    path::PathBuf,
};

fn get_config_file() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.toml")
}

pub(super) fn load_config() -> Config {
    let contents = match read_to_string(get_config_file()) {
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

            write_config(&config);
            return config;
        }
    };
    toml::from_str::<Config>(&contents).unwrap()
}

pub(super) fn write_config(config: &Config) {
    let contents = toml::to_string(&config).unwrap();
    write(get_config_file(), contents).unwrap();
}
