use crate::app::{
    App,
    config::filter::AudioFilter,
    gui::{
        Message, Theme,
        view::{Element, max_content_column::max_content_column, theme},
    },
};
use iced::{
    Alignment, Length,
    widget::{button, column, container, row, scrollable, svg, text, toggler},
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
                    .enumerate()
                    .map(|(i, filter)| self.filter_preset(i, filter)),
            )
            .spacing(4),
        );

        row([presets.into(), theme::v_separator(), filters.into()])
            .spacing(8)
            .into()
    }

    fn filter_preset<'a>(&'a self, i: usize, filter: &'a AudioFilter) -> Element<'a> {
        container(
            column(
                iter::once(
                    row([
                        button(svg(self.svgs.drag_handle.clone()).style(theme::svg_filter))
                            .width(Length::Shrink)
                            .padding(0)
                            .into(),
                        button(text(filter.name()))
                            .on_press(Message::ExpandFilter(i))
                            .width(Length::Fill)
                            .padding(0)
                            .into(),
                        container(
                            row([
                                toggler(filter.enabled)
                                    .on_toggle(move |v| Message::ToggleFilter(i, v))
                                    .style(|theme: &Theme, _status: toggler::Status| {
                                        toggler::Style {
                                            background: theme.filter_presets.toggle_bg.into(),
                                            background_border_width: 0.0,
                                            background_border_color: theme::MISSING_COLOR.into(),
                                            foreground: match filter.enabled {
                                                true => theme.filter_presets.toggle_on.into(),
                                                false => theme.filter_presets.toggle_off.into(),
                                            },
                                            foreground_border_width: 0.0,
                                            foreground_border_color: theme::MISSING_COLOR.into(),
                                            text_color: None,
                                            border_radius: None,
                                            padding_ratio: 0.1,
                                        }
                                    })
                                    .into(),
                                button(
                                    svg(self.svgs.expand_arrow.clone())
                                        .style(theme::svg_filter)
                                        .rotation(match filter.expanded {
                                            true => 0.0,
                                            false => -std::f32::consts::FRAC_PI_2,
                                        }),
                                )
                                .on_press(Message::ExpandFilter(i))
                                .width(Length::Shrink)
                                .padding(0)
                                .into(),
                            ])
                            .align_y(Alignment::Center)
                            .spacing(4),
                        )
                        .into(),
                    ])
                    .align_y(Alignment::Center)
                    .spacing(6)
                    .into(),
                )
                .chain(match filter.expanded {
                    true => Some(self.filter_properties(i, &filter.filter_type)),
                    false => None,
                }),
            )
            .spacing(4),
        )
        .padding(8)
        .style(theme::container_filter_preset)
        .width(Length::Fill)
        .into()
    }

    fn create_filter_summary(filters: &Vec<AudioFilter>) -> String {
        use std::fmt::Write;

        if filters.is_empty() {
            return "(none)".to_string();
        }

        let mut s = String::new();
        for filter in filters {
            let _ = writeln!(&mut s, "→ {}", filter.name());
        }
        s.pop(); // pop the last newline
        s
    }
}
