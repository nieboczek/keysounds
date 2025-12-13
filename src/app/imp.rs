use crate::app::{Action, App, Mode, StateStatus};
use rand::Rng;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use std::io;
use std::io::Stdout;
use std::path::Path;
use std::time::{Duration, Instant};

impl App {
    #[inline]
    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        let mut idle_render_deadline = Instant::now();
        self.render(terminal)?;

        loop {
            let mut state_status = self.handle_actions();

            if event::poll(Duration::from_millis(5))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_input(key, &mut state_status);
                }
            }

            state_status |= self.trigger_sfx_randomly();
            state_status |= self.check_playing_sfx();

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

    #[inline]
    fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        #[cfg(feature = "render_call_counter")]
        {
            self.render_call_counter += 1;
        }

        terminal.draw(|frame| {
            frame.render_widget(&mut *self, frame.area());
        })?;
        Ok(())
    }

    #[inline]
    fn check_playing_sfx(&mut self) -> StateStatus {
        if self.sfx_data.is_some() {
            if self.decoder.lock().unwrap().is_none() {
                self.sfx_data = None;
                return StateStatus::Updated;
            }
            return StateStatus::IdleRender;
        }
        StateStatus::Unaffected
    }

    #[inline]
    fn trigger_sfx_randomly(&mut self) -> StateStatus {
        if self.random_sfx_triggering && self.rat_deadline <= Instant::now() {
            let idx: usize = self.rng.random_range(0..self.config.rat_sfx_list.len());
            let name = &self.config.rat_sfx_list[idx];
            let sfx = self.config.sfx.iter().find(|sfx| &sfx.name == name);

            if let Some(sfx) = sfx {
                self.play_sfx(sfx.clone(), true);
            }
            // TODO: warn here if no sfx is found

            let min = self.config.rat_range.0;
            let max = self.config.rat_range.1;
            self.rat_deadline += Duration::from_secs_f32(self.rng.random_range(min..=max));
            return StateStatus::Updated;
        }
        StateStatus::Unaffected
    }

    #[inline]
    fn handle_actions(&mut self) -> StateStatus {
        let mut guard = self.action_channel.lock().unwrap();

        let old = std::mem::replace(&mut *guard, Action::None);
        match old {
            Action::None => return StateStatus::Unaffected,
            Action::SearchAndPlay => {
                Self::focus_console();
                self.input.clear();
                self.mode = Mode::SearchSfx;

                *guard = Action::None;
                return StateStatus::IgnoreNextKeyPress;
            }
            Action::SkipToPart => {
                if let Some(data) = &mut self.sfx_data {
                    let dur = Duration::from_secs_f32(data.sfx.skip_to);
                    self.decoder.lock().unwrap().as_mut().unwrap().seek(dur);
                }
            }
            Action::StopSfx => {
                *self.decoder.lock().unwrap() = None;
                self.sfx_data = None;
            }
            Action::FilterPreset(filters) => {
                self.filter_chain.lock().unwrap().sync_with_vector(filters);
            }
            Action::SetKeybinds(_) => {
                *guard = old;
                return StateStatus::Unaffected;
            }
        }
        StateStatus::Updated
    }

    pub fn validate_sfx_name(str: &str) -> bool {
        !str.is_empty()
    }

    pub fn validate_sfx_path(str: &str) -> bool {
        Path::new(str).is_file()
    }

    pub fn validate_sfx_volume(str: &str) -> bool {
        str.parse::<f32>().is_ok_and(|x| x >= 0.0)
    }

    pub fn validate_sfx_skip_to(str: &str) -> bool {
        str.parse::<f32>().is_ok_and(|x| x >= 0.0)
    }

    pub fn validate_input_device(str: &str) -> bool {
        !str.is_empty() // TODO: replace with better check that scans actual audio devices
    }

    pub fn validate_output_device(str: &str) -> bool {
        !str.is_empty() // TODO: replace with better check that scans actual audio devices
    }

    #[inline]
    fn focus_console() {
        // Windows is the reason why we can't have nice things in life.
        #[cfg(windows)]
        unsafe {
            use winapi::um::wincon::GetConsoleWindow;
            use winapi::um::winuser::{
                HWND_NOTOPMOST, HWND_TOPMOST, INPUT, INPUT_KEYBOARD, INPUT_u, KEYEVENTF_KEYUP,
                SW_RESTORE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SendInput, SetForegroundWindow,
                SetWindowPos, ShowWindow, VK_MENU,
            };

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
