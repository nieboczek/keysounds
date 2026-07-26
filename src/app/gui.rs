use crate::app::{Action, App, Page, Sound};
use iced::{Subscription, Task, time};
use rand::RngExt;
use std::time::{Duration, Instant};
use std::{path::Path, sync::atomic::Ordering};

mod view;
pub use view::theme::Theme;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ChangePage(Page),
    PlaySound(usize),
    StopSound,
    SearchInput(String),
    SearchSubmit,
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.trigger_sound_randomly();
                self.handle_actions();

                if self.decoder_pos.load(Ordering::Relaxed) == u64::MAX {
                    self.playing_sound = None;
                }
            }
            Message::ChangePage(page) => self.page = page,
            Message::PlaySound(index) => {
                if let Some(sound) = self.config.sounds.get(index) {
                    self.play_sound(sound.clone(), true);
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
        }
        Task::none()
    }

    pub fn subscription(_state: &App) -> Subscription<Message> {
        time::every(Duration::from_millis(16)).map(|_| Message::Tick)
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

    fn handle_actions(&mut self) {
        let mut guard = self.action_channel.lock().unwrap();

        let old = std::mem::replace(&mut *guard, Action::None);
        match old {
            Action::None => {}
            Action::SetKeybinds(_) => *guard = old,
            Action::SearchAndPlay => self.search.clear(),
            Action::StopSound => {
                *self.decoder.lock().unwrap() = None;
                self.playing_sound = None;
            }
            Action::FilterPreset(filters) => {
                self.filter_chain.lock().unwrap().sync_with_vector(filters);
            }
        }
    }

    fn trigger_sound_randomly(&mut self) {
        if self.sound_triggering && self.sound_triggering_deadline <= Instant::now() {
            let range = 0..self.config.sound_triggering_sound_list.len();
            let idx = self.rng.random_range(range);
            let name = &self.config.sound_triggering_sound_list[idx];
            let sound = self.config.sounds.iter().find(|sound| &sound.name == name);

            if let Some(sound) = sound {
                self.play_sound(sound.clone(), true);
            }

            let min = self.config.sound_triggering_interval_range.0;
            let max = self.config.sound_triggering_interval_range.1;
            self.sound_triggering_deadline +=
                Duration::from_secs_f32(self.rng.random_range(min..=max));
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
