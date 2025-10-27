use crate::app::{Action, App, Audio, AudioMeta, Mode, StateStatus};
use rand::Rng;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
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

    fn m_add_to_input(
        &mut self,
        k: KeyCode,
        handle_special: impl Fn(&mut App, KeyCode) -> StateStatus,
        handle_submit: impl Fn(&mut App) -> StateStatus,
    ) -> StateStatus {
        match k {
            KeyCode::Char(ch) => {
                self.input.push(ch);
            }
            KeyCode::Backspace => {
                if self.input.is_empty() {
                    return StateStatus::Unaffected;
                }
                self.input.pop();
            }
            KeyCode::Enter => return handle_submit(self),
            k => return handle_special(self, k),
        }
        StateStatus::Updated
    }

    fn m_list_input(
        &mut self,
        k: KeyCode,
        handle_special: impl Fn(&mut App, KeyCode) -> StateStatus,
        handle_submit: impl Fn(&mut App, usize) -> StateStatus,
        handle_esc: impl Fn(&mut App),
    ) -> StateStatus {
        match k {
            KeyCode::Down => {
                self.state.select_next();

                if self.is_separator(self.state.selected().unwrap()) {
                    self.state.select_next();
                }
            }
            KeyCode::Up => {
                self.state.select_previous();

                if self.is_separator(self.state.selected().unwrap()) {
                    self.state.select_previous();
                }
            }
            KeyCode::Esc => handle_esc(self),
            KeyCode::Enter => return handle_submit(self, self.state.selected().unwrap()),
            k => return handle_special(self, k),
        }
        StateStatus::Updated
    }

    fn m_audio_prop_selected_input(
        &mut self,
        k: KeyCode,
        previous_mode: Mode,
        next_mode: Mode,
        submit_mode: Mode,
        input_getter: impl Fn(&Audio) -> String,
    ) -> StateStatus {
        match k {
            KeyCode::Up => {
                if previous_mode != Mode::Null {
                    self.mode = previous_mode;
                    return StateStatus::Updated;
                }
            }
            KeyCode::Down => {
                if next_mode != Mode::Null {
                    self.mode = next_mode;
                    return StateStatus::Updated;
                }
            }
            KeyCode::Enter => {
                self.input = input_getter(&self.config.audios[self.state.selected().unwrap() - 1]);
                self.mode = submit_mode;
                return StateStatus::Updated;
            }
            KeyCode::Esc => {
                self.mode = Mode::EditAudios;
                return StateStatus::Updated;
            }
            _ => {}
        }
        StateStatus::Unaffected
    }

    fn m_audio_prop_edit_input(
        &mut self,
        k: KeyCode,
        validate: impl Fn(&String) -> bool,
        modify: impl Fn(&mut Audio, &mut String) -> Mode,
    ) -> StateStatus {
        self.m_add_to_input(
            k,
            |_, _| StateStatus::Unaffected,
            |a| {
                if validate(&a.input) {
                    a.mode = modify(
                        &mut a.config.audios[a.state.selected().unwrap() - 1],
                        &mut a.input,
                    );
                    StateStatus::Updated
                } else {
                    StateStatus::Unaffected
                }
            },
        )
    }

    #[inline]
    fn handle_input(&mut self, key: KeyEvent, state_status: &mut StateStatus) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        if *state_status == StateStatus::IgnoreNextKeyPress {
            return;
        }

        let k = key.code;

        *state_status |= match self.mode {
            Mode::Normal => self.m_list_input(
                k,
                |a, k| match k {
                    KeyCode::Char('q') => StateStatus::Quit,
                    KeyCode::Char('r') => {
                        a.load_config();
                        StateStatus::Updated
                    }

                    #[cfg(feature = "vhs_keybinds")]
                    KeyCode::Char('t') => {
                        a.mode = Mode::SearchAudio;
                        StateStatus::IgnoreNextKeyPress
                    }
                    #[cfg(feature = "vhs_keybinds")]
                    KeyCode::Char('s') => {
                        a.audio_meta = AudioMeta::reset();
                        a.audio = None;

                        a.sinks.0.stop();
                        a.sinks.1.stop();
                        StateStatus::Updated
                    }

                    _ => StateStatus::Unaffected,
                },
                |a, idx| {
                    match idx {
                        0 => {
                            a.shit_mic.fetch_not(Ordering::Relaxed);
                        }
                        1 => a.random_audio_triggering = !a.random_audio_triggering,
                        2 => return StateStatus::Unaffected, // TODO (audio list)
                        3 => return StateStatus::Unaffected, // TODO (range)
                        4 => return StateStatus::Unaffected, // separator
                        5 => {
                            a.state.select_first();
                            a.mode = Mode::EditConfig;
                        }
                        6 => a.load_config(),          // Reload config
                        7 => return StateStatus::Quit, // Exit
                        _ => unreachable!(),
                    }
                    StateStatus::Updated
                },
                |_| {},
            ),
            Mode::SearchAudio => match k {
                KeyCode::Char(ch) if self.mode == Mode::SearchAudio => {
                    self.input.push(ch);
                    let input = &self.input.to_ascii_lowercase();

                    for audio in &self.config.audios {
                        if audio.name.to_ascii_lowercase().contains(input) {
                            *state_status = StateStatus::Updated;
                            return;
                        }
                    }

                    self.input.pop();
                    StateStatus::Unaffected
                }
                KeyCode::Esc => {
                    self.input.clear();
                    self.mode = Mode::Normal;
                    StateStatus::Updated
                }
                KeyCode::Char(ch) => {
                    self.input.push(ch);
                    StateStatus::Updated
                }
                KeyCode::Backspace => {
                    if self.input.is_empty() {
                        StateStatus::Unaffected
                    } else {
                        self.input.pop();
                        StateStatus::Updated
                    }
                }
                KeyCode::Enter => {
                    let option = self.config.audios.iter().find(|audio| {
                        audio
                            .name
                            .to_ascii_lowercase()
                            .contains(&self.input.to_ascii_lowercase())
                    });

                    if let Some(audio) = option.cloned() {
                        self.play_audio(audio, false);
                        self.input = String::new();
                        self.mode = Mode::Normal;
                    }
                    StateStatus::Updated
                }
                _ => StateStatus::Unaffected,
            },
            Mode::EditConfig => self.m_list_input(
                k,
                |_, k| {
                    if k == KeyCode::Char('q') {
                        StateStatus::Quit
                    } else {
                        StateStatus::Unaffected
                    }
                },
                |a, idx| {
                    match idx {
                        0 => {
                            a.input = a.config.input_device.to_string();
                            a.mode = Mode::EditInputDevice;
                        }
                        1 => {
                            a.input = a.config.output_device.to_string();
                            a.mode = Mode::EditOutputDevice;
                        },
                        2 => return StateStatus::Unaffected, // separator
                        3 => return StateStatus::Unaffected, // Random audio triggering
                        4 => return StateStatus::Unaffected, // TODO (audio list)
                        5 => return StateStatus::Unaffected, // TODO (range)
                        6 => return StateStatus::Unaffected, // separator
                        7 => {
                            a.state.select(Some(0));
                            a.mode = Mode::EditAudios;
                        }
                        8 => {
                            a.save_config();
                            a.state.select(Some(5));
                            a.mode = Mode::Normal;
                        }
                        _ => unreachable!(),
                    }
                    StateStatus::Updated
                },
                |a| {
                    a.save_config();
                    a.state.select(Some(5));
                    a.mode = Mode::Normal;
                },
            ),
            Mode::EditInputDevice => self.m_add_to_input(
                k,
                |_, _| StateStatus::Unaffected,
                |a| {
                    a.config.input_device = a.input.split_off(0);
                    a.mode = Mode::EditConfig;
                    StateStatus::Updated
                },
            ),
            Mode::EditOutputDevice => self.m_add_to_input(
                k,
                |_, _| StateStatus::Unaffected,
                |a| {
                    a.config.output_device = a.input.split_off(0);
                    a.mode = Mode::EditConfig;
                    StateStatus::Updated
                },
            ),
            Mode::EditAudios => self.m_list_input(
                k,
                |_, _| StateStatus::Unaffected,
                |a, idx| {
                    match idx {
                        0 => {
                            a.config.audios.insert(
                                0,
                                Audio {
                                    name: String::new(),
                                    path: String::new(),
                                    volume: 1.0,
                                    skip_to: 0.0,
                                },
                            );
                            a.state.select(Some(1));
                            a.mode = Mode::SelectedAudioName;
                        }
                        1.. => a.mode = Mode::SelectedAudioName,
                    }
                    StateStatus::Updated
                },
                |a| {
                    a.state.select(Some(7));
                    a.mode = Mode::EditConfig;
                },
            ),
            Mode::SelectedAudioName => self.m_audio_prop_selected_input(
                k,
                Mode::Null,
                Mode::SelectedAudioPath,
                Mode::EditAudioName,
                |a| a.name.to_string(),
            ),
            Mode::EditAudioName => {
                self.m_audio_prop_edit_input(k, Self::validate_audio_name, |a, s| {
                    a.name = s.split_off(0);
                    Mode::SelectedAudioName
                })
            }
            Mode::SelectedAudioPath => self.m_audio_prop_selected_input(
                k,
                Mode::SelectedAudioName,
                Mode::SelectedAudioVolume,
                Mode::EditAudioPath,
                |a| a.path.to_string(),
            ),
            Mode::EditAudioPath => {
                self.m_audio_prop_edit_input(k, Self::validate_audio_path, |a, s| {
                    a.path = s.split_off(0);
                    Mode::SelectedAudioPath
                })
            }
            Mode::SelectedAudioVolume => self.m_audio_prop_selected_input(
                k,
                Mode::SelectedAudioPath,
                Mode::SelectedAudioSkipTo,
                Mode::EditAudioVolume,
                |a| a.volume.to_string(),
            ),
            Mode::EditAudioVolume => {
                self.m_audio_prop_edit_input(k, Self::validate_audio_volume, |a, s| {
                    a.volume = s.parse().unwrap();
                    Mode::SelectedAudioVolume
                })
            }
            Mode::SelectedAudioSkipTo => self.m_audio_prop_selected_input(
                k,
                Mode::SelectedAudioVolume,
                Mode::Null,
                Mode::EditAudioSkipTo,
                |a| a.skip_to.to_string(),
            ),
            Mode::EditAudioSkipTo => {
                self.m_audio_prop_edit_input(k, Self::validate_audio_skip_to, |a, s| {
                    a.skip_to = s.parse().unwrap();
                    Mode::SelectedAudioSkipTo
                })
            }
            Mode::Null => {
                if k == KeyCode::Char('q') {
                    StateStatus::Quit
                } else {
                    StateStatus::Unaffected
                }
            }
        };
    }

    #[inline]
    fn handle_actions(&mut self) -> StateStatus {
        let mut guard = self.channel.lock().unwrap();

        match *guard {
            Action::SearchAndPlay => {
                Self::focus_console();
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
    const fn is_separator(&self, idx: usize) -> bool {
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
