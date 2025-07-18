use std::{
    fs::{read_to_string, write},
    io::ErrorKind,
    path::PathBuf,
};

use super::Config;

fn get_config_file() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.toml")
}

pub(crate) fn load_config() -> Config {
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

pub(crate) fn write_config(config: &Config) {
    let contents = toml::to_string(&config).unwrap();
    write(get_config_file(), contents).unwrap();
}

#[derive(PartialEq, Debug)]
pub(crate) enum Setting {
    Bool(bool, &'static str),
    Range((f32, f32)),
    AudioList(usize),
    Action(&'static str),
    Separator,
}
