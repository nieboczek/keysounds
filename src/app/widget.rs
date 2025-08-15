use crate::app::{App, MENU_ITEMS, Mode};
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

macro_rules! subfield {
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

macro_rules! action {
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

#[inline]
fn get_menu_items<'a>(
    shit_mic: bool,
    random_audio_triggering: bool,
    audio_list_len: usize,
    range: (f32, f32),
) -> [Line<'a>; MENU_ITEMS] {
    [
        bool!(shit_mic, "Shit mic mode"),
        bool!(random_audio_triggering, "Random audio triggering"),
        subfield!(format!("audio list: [ {audio_list_len} elements ]")),
        subfield!(format!("range: {}-{}s", range.0, range.1)),
        sep!(),
        action!("Edit config"),
        action!("Reload config"),
        action!("Exit"),
    ]
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [hotkey_area, player_area, selectables_area] = Layout::vertical([
            Constraint::Length(5), // hotkeys; 4 lines + 1 empty
            Constraint::Length(3), // player; 2 lines + 1 empty
            Constraint::Fill(1),   // selectables;
        ])
        .areas(area);

        App::render_hotkeys(hotkey_area, buf);
        self.render_player(player_area, buf);
        match self.mode {
            Mode::Normal => self.render_selectables(selectables_area, buf),
            Mode::SearchAudio => self.render_input(selectables_area, buf),
            Mode::EditConfig => {}
        }
    }
}

impl App {
    #[inline]
    fn render_hotkeys(area: Rect, buf: &mut Buffer) {
        Paragraph::new(Text::from_iter([
            hotkey!("T", "Search and play audio."),
            hotkey!("Y", "Skip audio to target time."),
            hotkey!("S", "Stop audio."),
            hotkey!("G", "Toggle shit mic mode."),
        ]))
        .render(area, buf);
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
        let items = get_menu_items(
            self.shit_mic.load(Ordering::Relaxed),
            self.random_audio_triggering,
            self.config.rat_audio_list.len(),
            self.config.rat_range,
        );

        StatefulWidget::render(
            List::new(items).highlight_symbol("* "),
            area,
            buf,
            &mut self.state,
        );
    }
}
