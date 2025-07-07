use super::{Action, App};
use crate::{app::AudioMeta, config, StateStatus};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    prelude::CrosstermBackend,
    Terminal,
};
use std::{
    io::{self, Stdout},
    sync::atomic::Ordering,
    time::Duration,
};

#[cfg(windows)]
use winapi::um::{
    wincon::GetConsoleWindow,
    winuser::{
        INPUT_u, SendInput, SetForegroundWindow, SetWindowPos, ShowWindow, HWND_NOTOPMOST,
        HWND_TOPMOST, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, SWP_NOMOVE, SWP_NOSIZE,
        SWP_SHOWWINDOW, SW_RESTORE, VK_MENU,
    },
};

impl App {
    #[inline]
    pub(crate) fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> io::Result<()> {
        self.render(terminal)?;

        loop {
            let mut state_status = self.recieve();

            if event::poll(Duration::from_millis(1))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_input(key, &mut state_status);
                }
            }

            if self.sinks.0.empty() {
                self.audio_meta = AudioMeta::reset();
                self.audio = None;
                state_status |= StateStatus::Updated;
            }

            if self.audio.is_some() {
                state_status |= StateStatus::IdleRender;
            }

            self.idle_render_counter -= 1;
            match state_status {
                StateStatus::Unaffected => {}
                StateStatus::IdleRender => {
                    if self.idle_render_counter <= 0 {
                        self.idle_render_counter %= 1000;
                        self.render(terminal)?;
                    }
                }
                StateStatus::Updated => self.render(terminal)?,
                StateStatus::IgnoreNextKeyPress => unreachable!(),
                StateStatus::Quit => break,
            }
        }
        Ok(())
    }

    #[inline]
    fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        terminal.draw(|frame| {
            frame.render_widget(&mut *self, frame.area());
        })?;
        Ok(())
    }

    #[inline]
    fn handle_input(&mut self, key: KeyEvent, state_status: &mut StateStatus) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        if *state_status == StateStatus::IgnoreNextKeyPress {
            *state_status = StateStatus::Updated;
            return;
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
            return;
        }

        if key.code == KeyCode::Char('q') {
            *state_status = StateStatus::Quit;
            return;
        }

        *state_status |= self.handle_key(key.code);
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
    fn handle_key(&mut self, key_code: KeyCode) -> StateStatus {
        match key_code {
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
                    0 => {
                        let _ = self.shit_mic.fetch_not(Ordering::Relaxed);
                    }
                    1 => self.random_audio_triggering = !self.random_audio_triggering,
                    2 => return StateStatus::Unaffected, // TODO (range)
                    3 => return StateStatus::Unaffected, // TODO (audio list)
                    4 => return StateStatus::Unaffected, // separator
                    5 => self.config = config::load_config(), // Reload config
                    6 => return StateStatus::Quit,       // Exit
                    _ => unreachable!(),
                }
            }
            KeyCode::Char('r') => self.config = config::load_config(),
            _ => {}
        };
        StateStatus::Updated
    }

    #[inline]
    fn is_separator(&self, idx: usize) -> bool {
        idx == 4
    }

    #[inline]
    fn recieve(&mut self) -> StateStatus {
        if let Ok(action) = self.receiver.recv_timeout(Duration::from_millis(1)) {
            match action {
                Action::SearchAndPlay => {
                    App::focus_console();
                    self.inputting = true;
                    return StateStatus::IgnoreNextKeyPress;
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
                    self.shit_mic.fetch_not(Ordering::Relaxed);
                }
            }
            return StateStatus::Updated;
        }
        StateStatus::Unaffected
    }

    #[inline]
    fn focus_console() {
        // Windows is the reason why we can't have nice things in life.
        #[cfg(windows)]
        unsafe {
            // Fuck you I'm not feeling like using some bullshit ass API to do this. (deprecated)
            let hwnd = GetConsoleWindow();

            if hwnd.is_null() {
                return;
            }

            // https://stackoverflow.com/questions/30512267/keeping-the-terminal-in-focus
            ShowWindow(hwnd, SW_RESTORE);
            SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE + SWP_NOSIZE);
            SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE + SWP_NOSIZE);
            SetWindowPos(
                hwnd,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_SHOWWINDOW + SWP_NOMOVE + SWP_NOSIZE,
            );

            // This is the simplest way I found how to send a fucking Alt key
            let mut inputs = [
                INPUT {
                    type_: INPUT_KEYBOARD,
                    u: std::mem::zeroed::<INPUT_u>(),
                },
                INPUT {
                    type_: INPUT_KEYBOARD,
                    u: std::mem::zeroed::<INPUT_u>(),
                },
            ];

            // Alt press
            inputs[0].u.ki_mut().wVk = VK_MENU as u16;
            inputs[0].u.ki_mut().wScan = 0;
            inputs[0].u.ki_mut().dwFlags = 0;
            inputs[0].u.ki_mut().time = 0;
            inputs[0].u.ki_mut().dwExtraInfo = 0;

            // Alt release
            inputs[1].u.ki_mut().wVk = VK_MENU as u16;
            inputs[1].u.ki_mut().wScan = 0;
            inputs[1].u.ki_mut().dwFlags = KEYEVENTF_KEYUP;
            inputs[1].u.ki_mut().time = 0;
            inputs[1].u.ki_mut().dwExtraInfo = 0;

            SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
            SetForegroundWindow(hwnd);
        }
    }
}
