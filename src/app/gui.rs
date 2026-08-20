use crate::app::{
    App, DeviceOption, Page, Sound,
    config::{Keybind, filter::FilterProperty},
};
use iced::{
    Subscription, Task,
    keyboard::{self, Modifiers},
    time,
};
use std::{path::Path, sync::atomic::Ordering, time::Duration};

mod view;

pub use self::view::theme::Theme;

pub const SCALES: [f32; 23] = [
    0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0, 2.2, 2.4, 2.6,
    2.8, 3.0, 3.5, 4.0,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeybindTarget {
    SearchAndPlay,
    StopSound,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Keyboard(keyboard::Event),
    ChangePage(Page),
    PlaySound(usize),
    StopSound,
    SearchInput(String),
    SearchSubmit,
    // Filter Chain
    SelectPreset(usize),
    ToggleFilter(usize, bool),
    ExpandFilter(usize),
    ChangeFilterProperty(usize, FilterProperty),
    // Settings
    SetMicDevice(DeviceOption),
    SetOutDevice(DeviceOption),
    SetVirtualOutDevice(DeviceOption),
    SetGuiScale(f32),
    // Keybinds
    StartRecordingKeybind(KeybindTarget),
    CancelRecordingKeybind,
    ClearKeybind(KeybindTarget),
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {}
            Message::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match key {
                keyboard::Key::Character(c)
                    if (c == "-" || c == "=") && modifiers == Modifiers::COMMAND =>
                {
                    let idx = SCALES
                        .iter()
                        .enumerate()
                        .min_by(|a, b| {
                            (a.1 - self.config.gui_scale)
                                .abs()
                                .total_cmp(&(b.1 - self.config.gui_scale).abs())
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(0);

                    let new_idx = if c == "-" {
                        idx.saturating_sub(1)
                    } else {
                        (idx + 1).min(SCALES.len() - 1)
                    };
                    self.config.gui_scale = SCALES[new_idx];
                }
                _ => {}
            },
            Message::Keyboard(_) => {}
            Message::ChangePage(page) => {
                self.recording_keybind = None;
                self.page = page;
            }
            Message::PlaySound(index) => {
                if let Some(sound) = self.config.sounds.get(index) {
                    self.play_sound(sound.clone(), false);
                }
            }
            Message::StopSound => {
                *self.decoder.lock().unwrap() = None;
                self.playing_sound = None;
            }
            Message::SearchInput(input) => self.search = input,
            Message::SearchSubmit => {
                if Self::is_possible_path(&self.search) {
                    // Copy Path on Windows for some reason inserts quotation marks
                    let path = self.search.trim_matches('"').to_string();
                    if Path::new(&path).exists() {
                        self.search.clear();
                        self.play_sound_from_path(path);
                    }
                } else {
                    let sound = self
                        .get_search_results()
                        .next()
                        .map(|(_, sound)| sound.clone());
                    if let Some(sound) = sound {
                        self.play_sound(sound, false);
                    }
                }
            }
            Message::SelectPreset(idx) => self.selected_preset = idx,
            Message::ToggleFilter(idx, v) => {
                self.config.filter_presets[self.selected_preset].filters[idx].enabled = v;
            }
            Message::ExpandFilter(idx) => {
                let filter = &mut self.config.filter_presets[self.selected_preset].filters[idx];
                filter.expanded = !filter.expanded;
            }
            Message::ChangeFilterProperty(idx, prop) => {
                let filter = &mut self.config.filter_presets[self.selected_preset].filters[idx];
                prop.set(&mut filter.filter_type);
            }
            Message::SetMicDevice(device) => {
                self.mic_device = device;
                // TODO: actually like reconnect the audio loop and shit
            }
            Message::SetOutDevice(device) => {
                self.out_device = device;
                // TODO: actually like reconnect the audio loop and shit
            }
            Message::SetVirtualOutDevice(device) => {
                self.virtual_out_device = device;
                // TODO: actually like reconnect the audio loop and shit
            }
            Message::SetGuiScale(scale) => {
                self.config.gui_scale = scale;
            }
            Message::StartRecordingKeybind(target) => self.recording_keybind = Some(target),
            Message::CancelRecordingKeybind => self.recording_keybind = None,
            Message::ClearKeybind(target) => {
                let keybind = match target {
                    KeybindTarget::SearchAndPlay => &mut self.config.search_and_play_keybind,
                    KeybindTarget::StopSound => &mut self.config.stop_sound_keybind,
                };
                *keybind = None;
            }
        }

        self.handle_keybinds();

        if self.playing_sound.is_some() && self.decoder_pos.load(Ordering::Relaxed) == u64::MAX {
            self.playing_sound = None;
        }

        Task::none()
    }

    pub fn subscription(_state: &App) -> Subscription<Message> {
        use iced::keyboard::{Key, Modifiers};

        let keyboard = keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                key: Key::Character(ref c),
                modifiers,
                ..
            } if (c == "-" || c == "=") && modifiers == Modifiers::COMMAND => {
                Some(Message::Keyboard(event))
            }
            _ => None,
        });
        let time = time::every(Duration::from_millis(16)).map(|_| Message::Tick);

        Subscription::batch([keyboard, time])
    }

    fn get_search_results(&self) -> impl Iterator<Item = (usize, &Sound)> {
        let search = self.search.to_lowercase();
        self.config
            .sounds
            .iter()
            .enumerate()
            .filter(move |(_, sound)| Self::search_matches(&search, &sound.name))
    }

    fn search_matches(search: &str, sound_name: &str) -> bool {
        sound_name.to_lowercase().contains(search) // TODO: advanced search algorithm, upgrade to fzf at some point
    }

    fn handle_keybinds(&mut self) {
        #[inline]
        fn matches_keybind(keybind: Keybind, target_keybind: impl Into<Option<Keybind>>) -> bool {
            target_keybind.into().is_some_and(|t| t == keybind)
        }

        let Some(keybind) = self.keybind_listener.try_recv() else {
            return;
        };

        if let Some(target) = self.recording_keybind {
            let slot = match target {
                KeybindTarget::SearchAndPlay => &mut self.config.search_and_play_keybind,
                KeybindTarget::StopSound => &mut self.config.stop_sound_keybind,
            };
            *slot = Some(keybind);
            self.recording_keybind = None;
            return;
        }

        if matches_keybind(keybind, self.config.stop_sound_keybind) {
            *self.decoder.lock().unwrap() = None;
            self.playing_sound = None;
        } else if matches_keybind(keybind, self.config.search_and_play_keybind) {
            // noop for now
        } else {
            for preset in &self.config.filter_presets {
                if matches_keybind(keybind, preset.keybind) {
                    let filters = preset.filters.iter().filter_map(|f| match f.enabled {
                        true => Some(f.filter_type.clone()),
                        false => None,
                    });

                    let mut chain = self.filter_chain.lock().unwrap();
                    chain.sync(filters);
                    break;
                }
            }
        }
    }

    fn is_possible_path(str: &str) -> bool {
        #[cfg(windows)]
        {
            // Copy Path on Windows for some reason inserts quotation marks
            let mut chars = str.trim_matches('"').chars();

            macro_rules! ensure_next_char {
                ($expr:expr) => {
                    match chars.next() {
                        Some(ch) => {
                            if !$expr(ch) {
                                return false;
                            }
                            true
                        }
                        None => return true,
                    }
                };
            }

            ensure_next_char!(|c: char| c.is_ascii_uppercase());
            ensure_next_char!(|c: char| c == ':');
            ensure_next_char!(|c: char| c == '/' || c == '\\');
            true
        }
        #[cfg(unix)]
        {
            str.is_empty() || str.starts_with('/')
        }
    }
}
