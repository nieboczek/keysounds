use crate::app::{
    App,
    config::AudioFilter,
    gui::{
        Message, Theme,
        view::{Element, max_content_column::max_content_column, theme},
    },
};
use iced::{
    Alignment, Length,
    widget::{button, column, container, row, scrollable, slider, svg, text},
};
use std::iter;

impl App {
    pub(super) fn filter_chain_page(&self) -> Element<'_> {
        let presets =
            scrollable(
                max_content_column(self.config.filter_presets.iter().enumerate().map(
                    |(i, preset)| {
                        button(
                            column([
                                text(&preset.name).into(),
                                text(Self::create_filter_summary(&preset.filters))
                                    .style(theme::text_filter_preset_effects)
                                    .size(14)
                                    .into(),
                                text(preset.keybind.to_string())
                                    .style(theme::text_filter_preset_keybind)
                                    .size(14)
                                    .into(),
                            ])
                            .spacing(4),
                        )
                        .width(Length::Fill)
                        .on_press(Message::SelectPreset(i))
                        .style(move |theme: &Theme, status: button::Status| {
                            let active = self.selected_preset == i;
                            button::Style {
                                text_color: theme.text.into(),
                                background: match status {
                                    _ if active => theme.filter_presets.bg_active.into(),
                                    button::Status::Hovered | button::Status::Pressed => {
                                        theme.filter_presets.bg_hovered.into()
                                    }
                                    _ => theme.filter_presets.bg.into(),
                                },
                                border: match status {
                                    _ if active => theme.filter_presets.border_active.into(),
                                    button::Status::Hovered | button::Status::Pressed => {
                                        theme.filter_presets.border_hovered.into()
                                    }
                                    _ => theme.filter_presets.border.into(),
                                },
                                ..Default::default()
                            }
                        })
                        .into()
                    },
                ))
                .spacing(4),
            );

        let filters = scrollable(
            column(
                self.config.filter_presets[self.selected_preset]
                    .filters
                    .iter()
                    .map(|filter| {
                        container(
                            column(
                                iter::once(
                                    row([
                                        button(
                                            svg(self.svgs.drag_handle.clone())
                                                .style(theme::svg_filter),
                                        )
                                        .width(Length::Shrink)
                                        .padding(0)
                                        .into(),
                                        text(filter.human_name()).into(),
                                        container(
                                            button(
                                                svg(self.svgs.expand_arrow.clone())
                                                    .style(theme::svg_filter),
                                            )
                                            .width(Length::Shrink)
                                            .padding(0),
                                        )
                                        .align_right(Length::Fill)
                                        .into(),
                                    ])
                                    .align_y(Alignment::Center)
                                    .spacing(6)
                                    .into(),
                                )
                                .chain(match true {
                                    true => Some(Self::filter_properties(filter)),
                                    false => None,
                                }),
                            )
                            .spacing(4),
                        )
                        .padding(8)
                        .style(theme::container_filter_preset)
                        .width(Length::Fill)
                        .into()
                    }),
            )
            .spacing(4),
        );

        row([presets.into(), theme::v_separator(), filters.into()])
            .spacing(8)
            .into()
    }

    fn filter_properties(filter: &AudioFilter) -> Element<'_> {
        column([row([
            text("Sample property").into(),
            slider(0.0..=1000.0, 42.0, |_| Message::Tick).into(),
        ])
        .align_y(Alignment::Center)
        .spacing(4)
        .into()])
        .into()
    }

    fn create_filter_summary(filters: &Vec<AudioFilter>) -> String {
        use std::fmt::Write;

        if filters.is_empty() {
            return "(empty)".to_string();
        }

        let mut s = String::new();
        for filter in filters {
            let _ = writeln!(&mut s, "→ {}", filter.human_name());
        }
        s.pop(); // pop the last newline
        s
    }
}
