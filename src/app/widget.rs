use crate::app::{App, Mode};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span, Text},
    widgets::{List, Paragraph, StatefulWidget, Widget},
};
use std::{sync::atomic::Ordering, time::Duration};

macro_rules! hotkey {
    ($key:expr, $desc:expr) => {
        Line::from_iter([
            Span::raw("-> ").dark_gray(),
            Span::raw(concat!("Ctrl+Alt+", $key)).white(),
            Span::raw(concat!(": ", $desc)),
        ])
    };
}

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
            Mode::Normal | Mode::SearchAudio => {
                let [hotkey_area, player_area, selectables_area] = Layout::vertical([
                    Constraint::Length(5), // hotkeys; 4 lines + 1 empty
                    Constraint::Length(3), // player; 2 lines + 1 empty
                    Constraint::Fill(1),   // selectables;
                ])
                .areas(area);

                Paragraph::new(Text::from_iter([
                    hotkey!("T", "Search and play audio."),
                    hotkey!("Y", "Skip audio to target time."),
                    hotkey!("S", "Stop audio."),
                    hotkey!("G", "Toggle shit mic mode."),
                ]))
                .render(hotkey_area, buf);

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
            Mode::EditAudios => {
                self.render_audios(area, buf);
            }
            Mode::EditAudioName
            | Mode::EditAudioPath
            | Mode::EditAudioVolume
            | Mode::EditAudioSkipTo
            | Mode::SelectedAudioName
            | Mode::SelectedAudioPath
            | Mode::SelectedAudioVolume
            | Mode::SelectedAudioSkipTo => {
                self.render_audio(area, buf);
            }
            Mode::Null => {
                Paragraph::new(Text::from_iter([
                    "you have fucked something up.",
                    "consider reporting this as an issue",
                    "or fix it yourself",
                    "(Tried to render Mode::Null)",
                ]))
                .render(area, buf);
            }
        }
    }
}

impl App {
    #[inline]
    fn render_audio(&mut self, area: Rect, buf: &mut Buffer) {
        // Offset accounted for "Create new audio" item
        let audio = &self.config.audios[self.state.selected().unwrap() - 1];

        macro_rules! audio_prop {
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

        let name = audio_prop!(
            self,
            audio.name,
            "Name",
            SelectedAudioName,
            EditAudioName,
            Self::validate_audio_name
        );
        let path = audio_prop!(
            self,
            audio.path,
            "Path",
            SelectedAudioPath,
            EditAudioPath,
            Self::validate_audio_path
        );
        let volume = audio_prop!(
            self,
            audio.volume,
            "Volume",
            SelectedAudioVolume,
            EditAudioVolume,
            Self::validate_audio_volume
        );
        let skip_to = audio_prop!(
            self,
            audio.skip_to,
            "Skip to",
            SelectedAudioSkipTo,
            EditAudioSkipTo,
            Self::validate_audio_skip_to
        );

        Paragraph::new(Text::from_iter([name, path, volume, skip_to])).render(area, buf);
    }

    #[inline]
    fn render_audios(&mut self, area: Rect, buf: &mut Buffer) {
        let mut items = vec![text!("Create new audio")];
        items.extend(
            self.config
                .audios
                .iter()
                .map(|audio| text!(audio.name.as_str())),
        );

        StatefulWidget::render(
            List::new(items).highlight_symbol("* "),
            area,
            buf,
            &mut self.state,
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
                self.config.output_device,
                EditOutputDevice,
                Self::validate_output_device
            ),
            sep!(),
            text!("Random audio triggering"),
            subtext!(format!(
                "audio list: [ {} elements ]",
                self.config.rat_audio_list.len()
            )),
            subtext!(format!(
                "range: {} - {} s",
                self.config.rat_range.0, self.config.rat_range.1
            )),
            sep!(),
            text!(format!("Audios: [ {} elements ]", self.config.audios.len())),
            text!("Save and go back"),
        ];

        StatefulWidget::render(
            List::new(items).highlight_symbol("* "),
            area,
            buf,
            &mut self.state,
        );
    }

    #[inline]
    fn render_player(&self, area: Rect, buf: &mut Buffer) {
        //  37s  Audio Name
        //  >>-- RANDOM TRIGGER

        let time = format_time_left(
            self.audio_meta
                .duration
                .saturating_sub(self.sinks.0.get_pos()),
        );

        let name = self.audio.as_ref().map_or("", |audio| &audio.name);
        let animation = "----";

        #[cfg(not(feature = "render_call_counter"))]
        let note = if self.audio_meta.randomly_triggered {
            "RANDOM TRIGGER"
        } else {
            ""
        };

        #[cfg(feature = "render_call_counter")]
        let note = self.render_call_counter.to_string();

        Paragraph::new(format!("  {time} {name}\n  {animation} {note}")).render(area, buf);
    }

    #[inline]
    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        // Search: what
        // Matches: what, what is love, what the hell

        let matches = self
            .config
            .audios
            .iter()
            .filter_map(|audio| {
                audio
                    .name
                    .to_ascii_lowercase()
                    .contains(&self.input.to_ascii_lowercase())
                    .then_some(&audio.name)
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
            bool!(self.shit_mic.load(Ordering::Relaxed), "Shit mic mode"),
            bool!(self.random_audio_triggering, "Random audio triggering"),
            subtext!(format!(
                "audio list: [ {} elements ]",
                self.config.rat_audio_list.len()
            )),
            subtext!(format!(
                "range: {} - {} s",
                self.config.rat_range.0, self.config.rat_range.1
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
            &mut self.state,
        );
    }
}
