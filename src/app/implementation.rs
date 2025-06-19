use super::{Action, App};
use crate::{audio, config};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    prelude::CrosstermBackend,
    Terminal,
};
use std::{
    io::{self, Stdout},
    sync::Arc,
    thread,
    time::Duration,
};

impl App {
    pub(crate) fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> io::Result<()> {
        loop {
            self.recieve();

            terminal.draw(|frame| {
                frame.render_widget(&mut *self, frame.area());
            })?;

            if event::poll(Duration::from_millis(1))? {
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
                // 7 being the size of the list
                let next_selection = self.state.selected().unwrap() + 1;
                if next_selection < 7 {
                    self.state.select_next();

                    if self.is_separator(next_selection) {
                        self.state.select_next();
                    }
                }
            }
            KeyCode::Up => {
                let selection = self.state.selected().unwrap();
                if selection > 0 {
                    self.state.select_previous();

                    if self.is_separator(selection - 1) {
                        self.state.select_previous();
                    }
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                match self.state.selected().unwrap() {
                    0 => self.shit_mic = !self.shit_mic,
                    1 => self.random_audio_triggering = !self.random_audio_triggering,
                    2 => {} // TODO (range)
                    3 => {} // TODO (audio list)
                    4 => {} // separator
                    5 => self.config = config::read_config(),
                    6 => return true,
                    _ => unreachable!(),
                }
            }
            _ => {}
        }
        false
    }

    fn is_separator(&self, idx: usize) -> bool {
        idx == 4
    }

    fn recieve(&mut self) {
        if let Ok(action) = self.receiver.recv_timeout(Duration::from_millis(1)) {
            match action {
                Action::SearchAndPlay => {
                    let state = Arc::clone(&self.audio_state);

                    self.play_handle = Some(thread::spawn(move || {
                        audio::load_and_play("D:/Useful Folder/mp3/sound_effects/dream.mp3", state);
                    }));
                }
                Action::SkipToPart => {
                    if let Ok(state) = self.audio_state.lock().as_mut() {
                        if let Some(_) = &mut self.play_handle {
                            state.skip_to(69.420);
                        }
                    }
                }
                Action::StopAudio => {
                    if let Ok(state) = self.audio_state.lock().as_mut() {
                        if let Some(handle) = &mut self.play_handle {
                            if !handle.is_finished() {
                                // set end suffering flag
                                state.stop_audio();
                            }

                            state.sink1.stop();
                            state.sink2.stop();
                        }
                    }
                }
                Action::ToggleShitMic => {
                    self.shit_mic = !self.shit_mic;
                }
            }
        }
    }
}
