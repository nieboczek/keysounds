use super::{Action, App};
use crate::{app::AudioMeta, config};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    prelude::CrosstermBackend,
    Terminal,
};
use std::{
    io::{self, Stdout},
    time::Duration,
};

impl App {
    #[inline]
    pub(crate) fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> io::Result<()> {
        loop {
            let ignore_next_key = self.recieve();

            terminal.draw(|frame| {
                frame.render_widget(&mut *self, frame.area());
            })?;

            if event::poll(Duration::from_millis(5))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    if ignore_next_key {
                        continue;
                    }

                    if self.inputting {
                        match key.code {
                            KeyCode::Char(ch) => self.input.push(ch),
                            KeyCode::Backspace => {
                                let _ = self.input.pop();
                            }
                            KeyCode::Esc => {
                                self.input = String::new();
                                self.inputting = false;
                            }
                            KeyCode::Enter => self.submit_input(),
                            _ => {}
                        }
                        continue;
                    }

                    if self.handle_key(key.code) {
                        break;
                    }
                }
            }

            if self.sinks.0.empty() {
                self.audio_meta = AudioMeta::reset();
                self.audio = None;
            }
        }
        Ok(())
    }

    #[inline]
    fn submit_input(&mut self) {
        let option = self.config.audios.iter().find(|audio| {
            audio
                .name
                .to_ascii_lowercase()
                .contains(&self.input.to_ascii_lowercase())
        });

        if let Some(audio) = option.cloned() {
            self.play_file(&audio.path, audio.volume);
            self.audio = Some(audio);
            self.input = String::new();
            self.inputting = false;
        }
    }

    #[inline]
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

    #[inline]
    fn is_separator(&self, idx: usize) -> bool {
        idx == 4
    }

    #[inline]
    fn recieve(&mut self) -> bool {
        if let Ok(action) = self.receiver.recv_timeout(Duration::from_millis(5)) {
            match action {
                Action::SearchAndPlay => {
                    self.inputting = true;
                    return true;
                }
                Action::SkipToPart => {
                    if let Some(audio) = &self.audio {
                        let dur = Duration::from_secs_f32(audio.skip_to);

                        let _ = self.sinks.0.try_seek(dur);
                        let _ = self.sinks.1.try_seek(dur);
                    }
                }
                Action::StopAudio => {
                    self.audio_meta = AudioMeta::reset();
                    self.audio = None;

                    self.sinks.0.stop();
                    self.sinks.1.stop();
                }
                Action::ToggleShitMic => {
                    self.shit_mic = !self.shit_mic;
                }
            }
        }
        return false;
    }
}
