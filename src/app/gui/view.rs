use crate::app::{
    App, Page,
    gui::{Message, view::theme::Theme},
};
use iced::{
    Fill,
    widget::{
        Column, Row, button, column, container, progress_bar, row, scrollable, svg, text,
        text_input,
    },
};
use std::{iter, sync::atomic::Ordering, time::Duration};

mod overlay;
pub mod theme;

pub type Element<'a, Message = super::Message> = iced::Element<'a, Message, Theme>;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let tabs = row([
            self.tab("Sounds", Page::Sounds),
            self.tab("Filter Chain", Page::FilterChain),
            self.tab("Sound Triggering", Page::SoundTriggering),
        ]);

        let page_element = match self.page {
            Page::Sounds => self.sounds_page(),
            Page::FilterChain => self.sounds_page(),
            Page::SoundTriggering => self.sounds_page(),
        };

        let base = container(column([tabs.into(), page_element]).spacing(8).padding(8))
            .width(Fill)
            .height(Fill);

        overlay::Overlay::new(base, || self.player_overlay(), self.playing_sound.is_some()).into()
    }

    fn sounds_page(&self) -> Element<'_> {
        let search = text_input("Search Sounds...", &self.search)
            .on_input(Message::SearchInput)
            .on_submit(Message::SearchSubmit);

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

        column([search.into(), sound_list]).into()
    }

    fn tab<'a>(&'a self, page_name: &'a str, page: Page) -> Element<'a> {
        button(text(page_name))
            .style(move |theme: &Theme, status| button::Style {
                background: match status {
                    _ if self.page == page => theme.tab_active.into(),
                    button::Status::Active | button::Status::Disabled => theme.tab.into(),
                    _ => theme.tab_hovered.into(),
                },
                text_color: theme.text.into(),
                ..Default::default()
            })
            .on_press(Message::ChangePage(page))
            .into()
    }

    fn player_overlay(&self) -> Element<'_> {
        let Some(playing_sound) = self.playing_sound.as_ref() else {
            panic!("Overlay shouldn't be created when a sound isn't playing");
        };

        let sound_name = &playing_sound.sound.name;
        let pos = Duration::from_nanos(self.decoder_pos.load(Ordering::Relaxed));
        let duration = playing_sound.duration;
        let time_left_str = Self::format_time_left(duration.saturating_sub(pos));
        let progress = if duration.as_secs_f32() > 0.0 {
            (pos.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let player = container(
            column([
                row(iter::once(text(sound_name).size(20).into()).chain(
                    match playing_sound.randomly_triggered {
                        true => Some(Self::randomly_triggered_badge()),
                        false => None,
                    },
                ))
                .align_y(iced::Center)
                .spacing(8)
                .into(),
                row([
                    container(text(time_left_str).size(14))
                        .style(theme::container_opaque)
                        .center_y(32)
                        .padding([4, 8])
                        .into(),
                    button(svg(self.svgs.stop.clone()))
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
        .style(theme::container_overlay);

        container(player).padding(8).into()
    }

    fn randomly_triggered_badge() -> Element<'static> {
        container(text("Randomly Triggered"))
            .padding(2)
            .style(theme::container_badge)
            .into()
    }

    fn format_time_left(dur: Duration) -> String {
        let total_secs = dur.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}
