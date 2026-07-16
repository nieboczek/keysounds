use crate::app::{Action, App, Sfx};
use iced::{
    Element, Fill, Subscription, Task, time,
    widget::{
        Column, Row, button, column, container, progress_bar, row, scrollable, svg, text,
        text_input,
    },
};
use rand::RngExt;
use std::time::{Duration, Instant};
use std::{path::Path, sync::atomic::Ordering};

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    PlaySfx(usize),
    StopSfx,
    SearchInput(String),
    SearchSubmit,
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.trigger_sfx_randomly();
                self.handle_actions();
            }
            Message::PlaySfx(index) => {
                if let Some(sfx) = self.config.sfx.get(index) {
                    self.play_sfx(sfx.clone(), false);
                }
            }
            Message::StopSfx => {
                *self.decoder.lock().unwrap() = None;
                self.sfx_data = None;
            }
            Message::SearchInput(input) => {
                self.search = input;
            }
            Message::SearchSubmit => {
                if Self::is_possible_path(&self.search) {
                    // Copy Path on Windows for some reason inserts quotation marks
                    let path = self.search.trim_matches('"').to_string();
                    if Path::new(&path).exists() {
                        self.search.clear();
                        self.play_sfx_from_path(path);
                    }
                } else {
                    let sfx = self.get_search_results().next().map(|(_, sfx)| sfx.clone());
                    if let Some(sfx) = sfx {
                        self.play_sfx(sfx, false);
                    }
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let heading = self
            .sfx_data
            .as_ref()
            .map(|d| d.sfx.name.as_str())
            .unwrap_or("");

        let pos = Duration::from_nanos(self.decoder_pos.load(Ordering::Relaxed));
        let duration = self
            .sfx_data
            .as_ref()
            .map(|d| d.duration)
            .unwrap_or_default();

        let progress = if duration.as_secs_f32() > 0.0 {
            (pos.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let header = column![
            text(heading).size(20),
            row([
                container(text(Self::format_time_left(duration.saturating_sub(pos))).size(14))
                    .style(container::rounded_box)
                    .center_y(32)
                    .padding([4, 8])
                    .into(),
                button(svg(self.svgs.stop.clone()).style(|_, _| svg::Style {
                    color: Some(iced::Color::WHITE)
                }))
                .padding(0)
                .height(32)
                .width(32)
                .on_press(Message::StopSfx)
                .into(),
                progress_bar(0.0..=1.0, progress)
                    .length(Fill)
                    .girth(32)
                    .into(),
            ])
            .spacing(4),
        ]
        .spacing(4);

        let search = {
            text_input("Search SFX...", &self.search)
                .on_input(Message::SearchInput)
                .on_submit(Message::SearchSubmit)
        };

        let sfx_list: Element<'_, Message> = if self.config.sfx.is_empty() {
            text("No sounds configured").into()
        } else {
            let mut content = Column::new().spacing(8);
            let mut current_row = Row::new().spacing(8);
            let mut count = 0;

            for (i, sfx) in self.get_search_results() {
                let btn: Element<'_, Message> = button(text(sfx.name.as_str()).size(14))
                    .width(128)
                    .height(128)
                    .on_press(Message::PlaySfx(i))
                    .into();

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

        container(column![header, search, sfx_list].spacing(8).padding(16))
            .width(Fill)
            .height(Fill)
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

    fn get_search_results(&self) -> impl Iterator<Item = (usize, &Sfx)> {
        let search = self.search.to_lowercase();
        self.config
            .sfx
            .iter()
            .enumerate()
            .filter(move |(_, sfx)| Self::search_matches(&search, &sfx.name))
    }

    fn search_matches(search: &str, sfx_name: &str) -> bool {
        sfx_name.to_lowercase().contains(search) // TODO: advanced search algorithm, upgrade to fzf at some point
    }

    fn handle_actions(&mut self) {
        let mut guard = self.action_channel.lock().unwrap();

        let old = std::mem::replace(&mut *guard, Action::None);
        match old {
            Action::None => {}
            Action::SetKeybinds(_) => *guard = old,
            Action::SearchAndPlay => self.search.clear(),
            Action::StopSfx => {
                *self.decoder.lock().unwrap() = None;
                self.sfx_data = None;
            }
            Action::FilterPreset(filters) => {
                self.filter_chain.lock().unwrap().sync_with_vector(filters);
            }
        }
    }

    fn trigger_sfx_randomly(&mut self) {
        if self.random_sfx_triggering && self.rst_deadline <= Instant::now() {
            let idx: usize = self.rng.random_range(0..self.config.rst_sfx_list.len());
            let name = &self.config.rst_sfx_list[idx];
            let sfx = self.config.sfx.iter().find(|sfx| &sfx.name == name);

            if let Some(sfx) = sfx {
                self.play_sfx(sfx.clone(), true);
            }

            let min = self.config.rst_range.0;
            let max = self.config.rst_range.1;
            self.rst_deadline += Duration::from_secs_f32(self.rng.random_range(min..=max));
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
