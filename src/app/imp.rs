use crate::app::{Action, App, AudioMeta, Mode, StateStatus};
use rand::Rng;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::event::{self, Event},
};
use std::{
    io::{self, Stdout},
    path::Path,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

#[cfg(windows)]
use winapi::um::{
    wincon::GetConsoleWindow,
    winuser::{
        HWND_NOTOPMOST, HWND_TOPMOST, INPUT, INPUT_KEYBOARD, INPUT_u, KEYEVENTF_KEYUP, SW_RESTORE,
        SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SendInput, SetForegroundWindow, SetWindowPos,
        ShowWindow, VK_MENU,
    },
};

impl App {
    #[inline]
    pub(crate) fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> io::Result<()> {
        let mut idle_render_deadline = Instant::now();
        self.render(terminal)?;

        loop {
            let mut state_status = self.handle_actions();

            if event::poll(Duration::from_millis(1))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_input(key, &mut state_status);
                }
            }

            state_status |= self.trigger_audio_randomly();
            state_status |= self.check_playing_audio();

            match state_status {
                StateStatus::Unaffected => {}
                StateStatus::IdleRender => {
                    let now = Instant::now();
                    if idle_render_deadline <= now {
                        idle_render_deadline = now + Duration::from_secs(1);
                        self.render(terminal)?;
                    }
                }
                StateStatus::Updated | StateStatus::IgnoreNextKeyPress => self.render(terminal)?,
                StateStatus::Quit => break,
            }
        }
        Ok(())
    }

    #[cfg(feature = "render_call_counter")]
    #[inline]
    fn increment_render_call_counter(&mut self) {
        // Rust doesn't allow applying cfg(feature) directly onto an expression
        self.render_call_counter += 1;
    }

    #[inline]
    fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        #[cfg(feature = "render_call_counter")]
        self.increment_render_call_counter();

        terminal.draw(|frame| {
            frame.render_widget(&mut *self, frame.area());
        })?;
        Ok(())
    }

    #[inline]
    fn check_playing_audio(&mut self) -> StateStatus {
        if self.audio.is_some() {
            if self.sinks.0.empty() {
                self.audio_meta = AudioMeta::reset();
                self.audio = None;
                return StateStatus::Updated;
            }
            return StateStatus::IdleRender;
        }
        StateStatus::Unaffected
    }

    #[inline]
    fn trigger_audio_randomly(&mut self) -> StateStatus {
        if self.random_audio_triggering && self.rat_deadline <= Instant::now() {
            let idx: usize = self.rng.random_range(0..self.config.rat_audio_list.len());
            let name = &self.config.rat_audio_list[idx];
            let audio = self.config.audios.iter().find(|audio| &audio.name == name);

            if let Some(audio) = audio.cloned() {
                self.play_audio(audio, true);
            }
            // TODO: warn here if no audio is found

            let min = self.config.rat_range.0;
            let max = self.config.rat_range.1;
            self.rat_deadline += Duration::from_secs_f32(self.rng.random_range(min..=max));
            return StateStatus::Updated;
        }
        StateStatus::Unaffected
    }

    #[inline]
    fn handle_actions(&mut self) -> StateStatus {
        let mut guard = self.channel.lock().unwrap();

        match *guard {
            Action::SearchAndPlay => {
                Self::focus_console();
                self.input.clear();
                self.mode = Mode::SearchAudio;

                *guard = Action::None;
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
            Action::None => return StateStatus::Unaffected,
        }

        *guard = Action::None;
        StateStatus::Updated
    }

    #[inline]
    pub(crate) const fn is_separator(&self, idx: usize) -> bool {
        match self.mode {
            Mode::Normal => idx == 4,
            Mode::EditConfig => idx == 2 || idx == 6,
            Mode::EditAudios => false,
            _ => unreachable!(),
        }
    }

    pub(crate) fn validate_audio_name(str: &String) -> bool {
        !str.is_empty()
    }

    pub(crate) fn validate_audio_path(str: &String) -> bool {
        Path::new(str).is_file()
    }

    pub(crate) fn validate_audio_volume(str: &String) -> bool {
        str.parse::<f32>().is_ok_and(|x| x >= 0.0)
    }

    pub(crate) fn validate_audio_skip_to(str: &String) -> bool {
        str.parse::<f32>().is_ok_and(|x| x >= 0.0)
    }

    pub(crate) fn validate_input_device(str: &String) -> bool {
        !str.is_empty() // TODO: replace with better check that scans actual audio devices
    }

    pub(crate) fn validate_output_device(str: &String) -> bool {
        !str.is_empty() // TODO: replace with better check that scans actual audio devices
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
