use std::{
    io::{self, Stdout},
    sync::mpsc::Receiver,
    time::Duration,
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    prelude::CrosstermBackend,
    widgets::ListState,
    Terminal,
};

use super::{read_config, Action, App};

impl App {
    pub fn new(receiver: Receiver<Action>) -> App {
        let config = read_config();
        App {
            receiver,
            state: ListState::default().with_selected(Some(0)),
            shit_mic: false,
            random_audio_triggering: false,
            config,
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        loop {
            self.recieve();

            terminal.draw(|frame| {
                frame.render_widget(&mut *self, frame.area());
            })?;

            if event::poll(std::time::Duration::from_millis(1))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    if self.handle_key(key.code) {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Char('q') => return true,
            KeyCode::Down => {
                // 6 being the size of the list
                if self.state.selected().unwrap() + 1 < 6 {
                    self.state.select_next();
                }
            }
            KeyCode::Up => {
                if self.state.selected().unwrap() > 0 {
                    self.state.select_previous();
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                match self.state.selected().unwrap() {
                    0 => self.shit_mic = !self.shit_mic,
                    1 => self.random_audio_triggering = !self.random_audio_triggering,
                    2 => {} // TODO (range)
                    3 => {} // TODO (audio list)
                    4 => {} // TODO (reload configs)
                    5 => return true,
                    _ => unreachable!(),
                }
            }
            _ => {}
        }
        false
    }

    fn recieve(&mut self) {
        if let Ok(action) = self.receiver.recv_timeout(Duration::from_millis(1)) {
            match action {
                Action::SearchAndPlay => {}
                Action::SkipToPart => {}
                Action::StopAudio => {}
                Action::ToggleShitMic => {}
            }
        }
    }
}
