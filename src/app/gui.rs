use crate::app::{
    Action, App, Page, Sound,
    theme::{self, Theme},
};
use iced::{
    Fill, Subscription, Task, time,
    widget::{
        Column, Row, button, column, container, progress_bar, row, scrollable, svg, text,
        text_input,
    },
};
use rand::RngExt;
use std::time::{Duration, Instant};
use std::{path::Path, sync::atomic::Ordering};

mod overlay;

pub type Element<'a, Message = self::Message> = iced::Element<'a, Message, Theme>;

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
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let heading = self
            .playing_sound
            .as_ref()
            .map(|d| d.sound.name.as_str())
            .unwrap_or("");

        let pos = Duration::from_nanos(self.decoder_pos.load(Ordering::Relaxed));
        let duration = self
            .playing_sound
            .as_ref()
            .map(|d| d.duration)
            .unwrap_or_default();

        let progress = if duration.as_secs_f32() > 0.0 {
            (pos.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };

        fn tab(page_name: &str, page: Page, current_page: Page) -> Element<'_> {
            button(text(page_name))
                .style(move |theme: &Theme, status| button::Style {
                    background: match status {
                        _ if current_page == page => theme.tab_active.into(),
                        button::Status::Active | button::Status::Disabled => theme.tab.into(),
                        _ => theme.tab_hovered.into(),
                    },
                    text_color: theme.text.into(),
                    ..Default::default()
                })
                .on_press(Message::ChangePage(page))
                .into()
        }

        let tabs = row([
            tab("Sounds", Page::Sounds, self.page),
            tab("Filter Chain", Page::FilterChain, self.page),
            tab("Sound Triggering", Page::SoundTriggering, self.page),
        ]);

        let search = {
            text_input("Search Sounds...", &self.search)
                .on_input(Message::SearchInput)
                .on_submit(Message::SearchSubmit)
        };

        let sound_list: Element<'_> = if self.config.sounds.is_empty() {
            text("No sounds configured").into()
        } else {
            let mut content = Column::new().spacing(8);
            let mut current_row = Row::new().spacing(8);
            let mut count = 0;

            for (i, sound) in self.get_search_results() {
                let btn = button(text(sound.name.as_str()).size(14))
                    .width(128)
                    .height(128)
                    .on_press(Message::PlaySound(i));

                current_row = current_row.push(btn);
                count += 1;

                if count % 3 == 0 {
                    content = content.push(current_row);
                    current_row = Row::new().spacing(8);
                }
            }

            if count % 3 != 0 {
                content = content.push(current_row);
            }

            scrollable(content).into()
        };

        let base = container(
            column([tabs.into(), search.into(), sound_list])
                .spacing(8)
                .padding(8),
        )
        .width(Fill)
        .height(Fill);

        overlay::Overlay::new(
            base,
            move || {
                container(
                    container(
                        column([
                            text(heading).size(20).into(),
                            row([
                                container(
                                    text(Self::format_time_left(duration.saturating_sub(pos)))
                                        .size(14),
                                )
                                .style(theme::container_opaque)
                                .center_y(32)
                                .padding([4, 8])
                                .into(),
                                button(svg(self.svgs.stop.clone()).style(|_, _| svg::Style {
                                    color: Some(iced::Color::WHITE),
                                }))
                                .padding(0)
                                .height(32)
                                .width(32)
                                .on_press(Message::StopSound)
                                .into(),
                                progress_bar(0.0..=1.0, progress)
                                    .length(Fill)
                                    .girth(32)
                                    .into(),
                            ])
                            .spacing(4)
                            .into(),
                        ])
                        .spacing(4),
                    )
                    .padding(12)
                    .style(theme::container_overlay),
                )
                .padding(8)
                .into()
            },
            self.playing_sound.is_some(),
        )
        .into()
    }

    pub fn subscription(_state: &App) -> Subscription<Message> {
        time::every(Duration::from_millis(16)).map(|_| Message::Tick)
    }

    fn format_time_left(dur: Duration) -> String {
        let total_secs = dur.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
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
            let idx: usize = self
                .rng
                .random_range(0..self.config.sound_triggering_sound_list.len());
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

    pub fn is_possible_path(str: &str) -> bool {
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
