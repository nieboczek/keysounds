use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::{
    fs::{read_to_string, write},
    io::ErrorKind,
    path::PathBuf,
    sync::mpsc::Receiver,
};

mod implementation;
mod widget;

pub struct App {
    receiver: Receiver<Action>,
    state: ListState,
    shit_mic: bool,
    random_audio_triggering: bool,
    config: Config,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    rat_range: (usize, usize),
    rat_audio_list: Vec<String>,
}

pub enum Action {
    SearchAndPlay,
    SkipToPart,
    StopAudio,
    ToggleShitMic,
}

fn get_config_file() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.toml")
}

pub(super) fn read_config() -> Config {
    let contents = match read_to_string(get_config_file()) {
        Ok(contents) => contents,
        Err(err) => {
            if err.kind() != ErrorKind::NotFound {
                panic!("Couldn't read the config file! {err}");
            }

            let config = Config {
                rat_range: (600, 900),
                rat_audio_list: Vec::new(),
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
