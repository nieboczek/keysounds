use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span, Text},
    widgets::{List, Paragraph, StatefulWidget, Widget},
};

use super::App;

macro_rules! hotkey {
    ($key:expr, $desc:expr) => {
        Line::from(vec![
            Span::raw("-> ").dark_gray(),
            Span::raw(concat!("Ctrl+Alt+", $key)).white(),
            Span::raw(concat!(": ", $desc)),
        ])
    };
}

macro_rules! subfield {
    () => {
        Span::raw(" -> ").dark_gray()
    };
}

macro_rules! bool {
    ($val:expr, $desc:expr) => {
        Text::from(vec![Line::from(vec![
            if $val {
                Span::raw("ON  ").light_green()
            } else {
                Span::raw("OFF ").light_red()
            },
            Span::raw($desc),
        ])])
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
        let (start, end) = self.config.rat_range;
        let audio_count = self.config.rat_audio_list.len();

        let items = [
            bool!(self.shit_mic, "Shit mic mode"),
            bool!(self.random_audio_triggering, "Random audio triggering"),
            Text::from(Line::from(vec![
                subfield!(),
                Span::raw(format!("range: {}-{}s", start, end)),
            ])),
            Text::from(vec![
                Line::from(vec![
                    subfield!(),
                    Span::raw(format!("audio list: [ {} elements ]", audio_count)),
                ]),
                Line::default(),
            ]),
            Text::from("Reload configs"),
            Text::from("Exit"),
        ];

        StatefulWidget::render(
            List::new(items).highlight_symbol("* "),
            area,
            buf,
            &mut self.state,
        );
    }
}
