use crate::app::{App, Mode};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{List, Paragraph, StatefulWidget, Widget};
use std::time::Duration;

macro_rules! subtext {
    ($string:expr) => {
        Line::from_iter([Span::raw(" -> ").dark_gray(), Span::raw($string)])
    };
}

macro_rules! bool {
    ($bool:expr, $desc:expr) => {
        Line::from_iter([
            if $bool {
                Span::raw("ON  ").light_green()
            } else {
                Span::raw("OFF ").light_red()
            },
            Span::raw($desc),
        ])
    };
}

macro_rules! sep {
    () => {
        Line::default()
    };
}

macro_rules! text {
    ($string:expr) => {
        Line::from($string)
    };
}

#[inline]
fn format_time_left(dur: Duration) -> String {
    let string = dur.as_secs().to_string();
    let len = string.len();

    string
        + "s"
        + match len {
            0 => unreachable!(),
            1 => "  ",
            2 => " ",
            3.. => "",
        }
}

impl Widget for &mut App {
    #[inline]
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.mode {
            Mode::Normal | Mode::SearchSfx => {
                let [player_area, selectables_area] = Layout::vertical([
                    Constraint::Length(4), // player; 2 lines + 2 empty
                    Constraint::Fill(1),   // selectables;
                ])
                .areas(area);

                self.render_player(player_area, buf);
                if self.mode == Mode::Normal {
                    self.render_selectables(selectables_area, buf);
                } else {
                    self.render_input(selectables_area, buf);
                }
            }
            Mode::EditConfig | Mode::EditInputDevice | Mode::EditOutputDevice => {
                self.render_config_editor(area, buf);
            }
            Mode::EditSfxs => {
                self.render_sfxs(area, buf);
            }
            Mode::EditSfxName
            | Mode::EditSfxPath
            | Mode::EditSfxVolume
            | Mode::EditSfxSkipTo
            | Mode::SelectedSfxName
            | Mode::SelectedSfxPath
            | Mode::SelectedSfxVolume
            | Mode::SelectedSfxSkipTo => {
                self.render_sfx(area, buf);
            }
            Mode::Null => {
                Paragraph::new(Text::from_iter([
                    "You have f'd something up.",
                    "Consider reporting this as an issue",
                    "(if this is an build from master),",
                    "or fix it yourself.",
                    "(Tried to render Mode::Null)",
                ]))
                .render(area, buf);
            }
        }
    }
}

impl App {
    #[inline]
    fn render_sfx(&mut self, area: Rect, buf: &mut Buffer) {
        // Offset accounted for "Create new sfx" item
        let sfx = &self.config.sfx[self.list_state.selected().unwrap() - 1];

        macro_rules! sfx_prop {
            ($app:expr, $value:expr, $name:expr, $selected_mode:ident, $edit_mode:ident, $validate_fn:expr) => {
                if $app.mode == Mode::$edit_mode {
                    let span = Span::raw(&$app.input);
                    let span = if $validate_fn(&$app.input) {
                        span.green()
                    } else {
                        span.red()
                    };

                    Line::from_iter([Span::raw(concat!("* ", $name, ": ")), span])
                } else if $app.mode == Mode::$selected_mode {
                    text!(format!(concat!("* ", $name, ": {}"), $value))
                } else {
                    text!(format!(concat!("  ", $name, ": {}"), $value))
                }
            };
        }

        let name = sfx_prop!(
            self,
            sfx.name,
            "Name",
            SelectedSfxName,
            EditSfxName,
            Self::validate_sfx_name
        );
        let path = sfx_prop!(
            self,
            sfx.path,
            "Path",
            SelectedSfxPath,
            EditSfxPath,
            Self::validate_sfx_path
        );
        let volume = sfx_prop!(
            self,
            sfx.volume,
            "Volume",
            SelectedSfxVolume,
            EditSfxVolume,
            Self::validate_sfx_volume
        );
        let skip_to = sfx_prop!(
            self,
            sfx.skip_to,
            "Skip to",
            SelectedSfxSkipTo,
            EditSfxSkipTo,
            Self::validate_sfx_skip_to
        );

        Paragraph::new(Text::from_iter([name, path, volume, skip_to])).render(area, buf);
    }

    #[inline]
    fn render_sfxs(&mut self, area: Rect, buf: &mut Buffer) {
        let mut items = vec![text!("Create new sfx")];
        items.extend(self.config.sfx.iter().map(|sfx| text!(sfx.name.as_str())));

        StatefulWidget::render(
            List::new(items).highlight_symbol("* "),
            area,
            buf,
            &mut self.list_state,
        );
    }

    #[inline]
    fn render_config_editor(&mut self, area: Rect, buf: &mut Buffer) {
        macro_rules! editable_string {
            ($app:expr, $label:expr, $value:expr, $edit_mode:ident, $validate_fn:expr) => {
                if $app.mode == Mode::$edit_mode {
                    let span = Span::raw(&$app.input);
                    let span = if $validate_fn(&$app.input) {
                        span.green()
                    } else {
                        span.red()
                    };

                    Line::from_iter([Span::raw(concat!($label, ": ")), span])
                } else {
                    text!(format!(concat!($label, ": {}"), $value))
                }
            };
        }

        let items = [
            editable_string!(
                self,
                "Input device",
                self.config.input_device,
                EditInputDevice,
                Self::validate_input_device
            ),
            editable_string!(
                self,
                "Output device",
                self.config.virtual_output_device,
                EditOutputDevice,
                Self::validate_output_device
            ),
            sep!(),
            text!("Random sfx triggering"),
            subtext!(format!(
                "sfx list: [ {} elements ]",
                self.config.rst_sfx_list.len()
            )),
            subtext!(format!(
                "range: {} - {} s",
                self.config.rst_range.0, self.config.rst_range.1
            )),
            sep!(),
            text!(format!("Sfxs: [ {} elements ]", self.config.sfx.len())),
            text!("Save and go back"),
        ];

        StatefulWidget::render(
            List::new(items).highlight_symbol("* "),
            area,
            buf,
            &mut self.list_state,
        );
    }

    #[inline]
    fn render_player(&self, area: Rect, buf: &mut Buffer) {
        //  37s  Sfx name
        //  >>-- RANDOM TRIGGER

        let text = if let Some(data) = &self.sfx_data {
            let time = format_time_left(
                data.duration
                    .saturating_sub(self.decoder.lock().unwrap().as_ref().unwrap().get_pos()),
            );
            let name = &data.sfx.name;
            let animation = "----";

            #[cfg(not(feature = "render_call_counter"))]
            let note = if data.randomly_triggered {
                "RANDOM TRIGGER"
            } else {
                ""
            };

            #[cfg(feature = "render_call_counter")]
            let note = self.render_call_counter.to_string();

            format!("\n  {time} {name}\n  {animation} {note}")
        } else {
            #[cfg(not(feature = "render_call_counter"))]
            let note = "";

            #[cfg(feature = "render_call_counter")]
            let note = self.render_call_counter.to_string();

            format!("\n  0s\n  ---- {note}")
        };

        Paragraph::new(text).render(area, buf);
    }

    #[inline]
    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        // Search: what
        // Matches: what, what is love, what the hell

        let matches = self
            .config
            .sfx
            .iter()
            .filter_map(|sfx| {
                sfx.name
                    .to_ascii_lowercase()
                    .contains(&self.input.to_ascii_lowercase())
                    .then_some(&sfx.name)
            })
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ");

        Paragraph::new(Text::from_iter([
            Line::from_iter([Span::raw("Search: ").bold(), Span::raw(&self.input)]),
            Line::from_iter([Span::raw("Matches: ").bold(), Span::from(matches)]),
        ]))
        .render(area, buf);
    }

    #[inline]
    fn render_selectables(&mut self, area: Rect, buf: &mut Buffer) {
        let items = [
            bool!(self.random_sfx_triggering, "Random sfx triggering"),
            subtext!(format!(
                "sfx list: [ {} elements ]",
                self.config.rst_sfx_list.len()
            )),
            subtext!(format!(
                "range: {} - {} s",
                self.config.rst_range.0, self.config.rst_range.1
            )),
            sep!(),
            text!("Edit config"),
            text!("Reload config"),
            text!("Exit"),
        ];

        StatefulWidget::render(
            List::new(items).highlight_symbol("* "),
            area,
            buf,
            &mut self.list_state,
        );
    }
}
