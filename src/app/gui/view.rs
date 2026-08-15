use crate::app::{
    App, Page,
    gui::{Message, view::theme::Theme},
};
use iced::{
    Length,
    widget::{button, column, container, progress_bar, row, space, svg, text},
};
use std::{iter, sync::atomic::Ordering, time::Duration};

mod filter_presets;
mod filter_properties;
mod max_content_column;
mod overlay;
mod settings;
mod sounds;
pub mod theme;

pub type Element<'a, Message = super::Message> = iced::Element<'a, Message, Theme>;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let tabs = row([
            self.tab("Sounds", Page::Sounds),
            self.tab("Filter Chain", Page::FilterChain),
            self.tab("Settings", Page::Settings),
        ])
        .spacing(16);

        let page_element = match self.page {
            Page::Sounds => self.sounds_page(),
            Page::FilterChain => self.filter_chain_page(),
            Page::Settings => self.settings_page(),
        };

        let base = container(column([tabs.into(), page_element]).spacing(8).padding(8))
            .width(Length::Fill)
            .height(Length::Fill);

        overlay::Overlay::new(base, || self.player_overlay(), self.playing_sound.is_some()).into()
    }

    fn tab<'a>(&'a self, page_name: &'a str, page: Page) -> Element<'a> {
        button(
            column([
                text(page_name).into(),
                container(space::horizontal())
                    .height(2)
                    .style(if self.page == page {
                        theme::container_tab_underline
                    } else {
                        theme::container_default
                    })
                    .into(),
            ])
            .width(Length::Shrink)
            .spacing(2),
        )
        .padding(0)
        .on_press(Message::ChangePage(page))
        .style(move |theme: &Theme, status: button::Status| button::Style {
            text_color: match status {
                _ if self.page == page => theme.tabs.text_active.into(),
                button::Status::Pressed => theme.tabs.text_active.into(),
                button::Status::Hovered => theme.tabs.text_hovered.into(),
                _ => theme.tabs.text.into(),
            },
            ..Default::default()
        })
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
                        .style(theme::container_time)
                        .center_y(32)
                        .padding([4, 8])
                        .into(),
                    button(svg(self.svgs.stop.clone()).style(theme::svg_stop))
                        .padding(0)
                        .height(32)
                        .width(32)
                        .on_press(Message::StopSound)
                        .style(theme::button_stop)
                        .into(),
                    progress_bar(0.0..=1.0, progress)
                        .length(Length::Fill)
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
