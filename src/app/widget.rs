use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span, Text},
    widgets::{List, Paragraph, StatefulWidget, Widget},
};

use crate::config::Setting;

use super::App;

macro_rules! subfield {
    ($string:expr) => {
        Text::from(Line::from_iter([
            Span::raw(" -> ").dark_gray(),
            Span::raw($string),
        ]))
    };
}

macro_rules! hotkey {
    ($key:expr, $desc:expr) => {
        Line::from(vec![
            Span::raw("-> ").dark_gray(),
            Span::raw(concat!("Ctrl+Alt+", $key)).white(),
            Span::raw(concat!(": ", $desc)),
        ])
    };
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
        self.render_selectables(selectables_area, buf);
    }
}

impl App {
    fn render_hotkeys(area: Rect, buf: &mut Buffer) {
        Paragraph::new(Text::from(vec![
            hotkey!("T", "Search and play audio."),
            hotkey!("Y", "Skip audio to target time."),
            hotkey!("S", "Stop audio."),
            hotkey!("G", "Toggle shit mic mode."),
        ]))
        .render(area, buf);
    }

    fn render_player(&self, area: Rect, buf: &mut Buffer) {
        //  37s  Audio Name
        //  >>-- RANDOM TRIGGER

        let time = "69s ";
        let name = "bullshit";
        let animation = "----";
        let note = "RANDOM TRIGGER";

        Paragraph::new(format!("  {time} {name}\n  {animation} {note}\n")).render(area, buf);
    }

    fn render_selectables(&mut self, area: Rect, buf: &mut Buffer) {
        let items = [
            Setting::Bool(self.shit_mic, "Shit mic mode"),
            Setting::Bool(self.random_audio_triggering, "Random audio triggering"),
            Setting::AudioList(self.config.rat_audio_list.len()),
            Setting::Range(self.config.rat_range),
            Setting::Separator,
            Setting::Action("Reload configs"),
            Setting::Action("Exit"),
        ];

        StatefulWidget::render(
            List::new(items).highlight_symbol("* "),
            area,
            buf,
            &mut self.state,
        );
    }
}

impl From<Setting> for Text<'_> {
    fn from(value: Setting) -> Self {
        match value {
            Setting::Bool(bool, str) => Text::from(Line::from_iter([
                if bool {
                    Span::raw("ON  ").light_green()
                } else {
                    Span::raw("OFF ").light_red()
                },
                str.into(),
            ])),
            Setting::Range((min, max)) => subfield!(format!("range: {}-{}s", min, max)),
            Setting::AudioList(len) => subfield!(format!("audio list: [ {} elements ]", len)),
            Setting::Action(str) => str.into(),
            Setting::Separator => Text::from(Line::default()),
        }
    }
}
