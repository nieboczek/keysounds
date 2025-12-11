use crate::app::{App, Mode, Sfx, StateStatus};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use std::sync::atomic::Ordering;

impl App {
    fn m_add_to_input(
        &mut self,
        k: KeyCode,
        handle_special: impl Fn(&mut App, KeyCode) -> StateStatus,
        handle_submit: impl Fn(&mut App) -> StateStatus,
    ) -> StateStatus {
        match k {
            KeyCode::Char(ch) => self.input.push(ch),
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
                self.list_state.select_next();

                if self.is_separator(self.list_state.selected().unwrap()) {
                    self.list_state.select_next();
                }
            }
            KeyCode::Up => {
                self.list_state.select_previous();

                if self.is_separator(self.list_state.selected().unwrap()) {
                    self.list_state.select_previous();
                }
            }
            KeyCode::Esc => handle_esc(self),
            KeyCode::Enter => return handle_submit(self, self.list_state.selected().unwrap()),
            k => return handle_special(self, k),
        }
        StateStatus::Updated
    }

    fn m_sfx_prop_selected_input(
        &mut self,
        k: KeyCode,
        previous_mode: Mode,
        next_mode: Mode,
        submit_mode: Mode,
        input_getter: impl Fn(&Sfx) -> String,
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
                self.input =
                    input_getter(&self.config.sfx[self.list_state.selected().unwrap() - 1]);
                self.mode = submit_mode;
                return StateStatus::Updated;
            }
            KeyCode::Esc => {
                self.mode = Mode::EditSfxs;
                return StateStatus::Updated;
            }
            _ => {}
        }
        StateStatus::Unaffected
    }

    fn m_sfx_prop_edit_input(
        &mut self,
        k: KeyCode,
        modify: impl Fn(&mut Sfx, &mut String) -> Mode,
    ) -> StateStatus {
        self.m_add_to_input(
            k,
            |_, _| StateStatus::Unaffected,
            |a| {
                a.mode = modify(
                    &mut a.config.sfx[a.list_state.selected().unwrap() - 1],
                    &mut a.input,
                );
                StateStatus::Updated
            },
        )
    }

    #[inline]
    pub(super) fn handle_input(&mut self, key: KeyEvent, state_status: &mut StateStatus) {
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
                        a.mode = Mode::SearchSfx;
                        StateStatus::IgnoreNextKeyPress
                    }
                    #[cfg(feature = "vhs_keybinds")]
                    KeyCode::Char('s') => {
                        a.sfx_meta = SfxMeta::reset();
                        a.sfx = None;

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
                        1 => a.random_sfx_triggering = !a.random_sfx_triggering,
                        2 => return StateStatus::Unaffected, // TODO (sfx list)
                        3 => return StateStatus::Unaffected, // TODO (range)
                        4 => return StateStatus::Unaffected, // separator
                        5 => {
                            a.list_state.select_first();
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
            Mode::SearchSfx => match k {
                KeyCode::Char(ch) if self.mode == Mode::SearchSfx => {
                    self.input.push(ch);
                    let input = &self.input.to_ascii_lowercase();

                    for sfx in &self.config.sfx {
                        if sfx.name.to_ascii_lowercase().contains(input) {
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
                    let option = self.config.sfx.iter().find(|sfx| {
                        sfx.name
                            .to_ascii_lowercase()
                            .contains(&self.input.to_ascii_lowercase())
                    });

                    if let Some(sfx) = option {
                        self.play_sfx(sfx.clone(), false);
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
                        }
                        2 => return StateStatus::Unaffected, // separator
                        3 => return StateStatus::Unaffected, // Random sfx triggering
                        4 => return StateStatus::Unaffected, // TODO (sfx list)
                        5 => return StateStatus::Unaffected, // TODO (range)
                        6 => return StateStatus::Unaffected, // separator
                        7 => {
                            a.list_state.select(Some(0));
                            a.mode = Mode::EditSfxs;
                        }
                        8 => {
                            a.save_config();
                            a.list_state.select(Some(5));
                            a.mode = Mode::Normal;
                        }
                        _ => unreachable!(),
                    }
                    StateStatus::Updated
                },
                |a| {
                    a.save_config();
                    a.list_state.select(Some(5));
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
            Mode::EditSfxs => self.m_list_input(
                k,
                |_, _| StateStatus::Unaffected,
                |a, idx| {
                    match idx {
                        0 => {
                            a.config.sfx.insert(
                                0,
                                Sfx {
                                    name: String::new(),
                                    path: String::new(),
                                    volume: 1.0,
                                    skip_to: 0.0,
                                },
                            );
                            a.list_state.select(Some(1));
                            a.mode = Mode::SelectedSfxName;
                        }
                        1.. => a.mode = Mode::SelectedSfxName,
                    }
                    StateStatus::Updated
                },
                |a| {
                    a.list_state.select(Some(7));
                    a.mode = Mode::EditConfig;
                },
            ),
            Mode::SelectedSfxName => self.m_sfx_prop_selected_input(
                k,
                Mode::Null,
                Mode::SelectedSfxPath,
                Mode::EditSfxName,
                |a| a.name.to_string(),
            ),
            Mode::EditSfxName => self.m_sfx_prop_edit_input(k, |a, s| {
                a.name = s.split_off(0);
                Mode::SelectedSfxName
            }),
            Mode::SelectedSfxPath => self.m_sfx_prop_selected_input(
                k,
                Mode::SelectedSfxName,
                Mode::SelectedSfxVolume,
                Mode::EditSfxPath,
                |a| a.path.to_string(),
            ),
            Mode::EditSfxPath => self.m_sfx_prop_edit_input(k, |a, s| {
                a.path = s.split_off(0);
                Mode::SelectedSfxPath
            }),
            Mode::SelectedSfxVolume => self.m_sfx_prop_selected_input(
                k,
                Mode::SelectedSfxPath,
                Mode::SelectedSfxSkipTo,
                Mode::EditSfxVolume,
                |a| a.volume.to_string(),
            ),
            Mode::EditSfxVolume => self.m_sfx_prop_edit_input(k, |a, s| {
                if let Ok(volume) = s.parse::<f32>() {
                    a.volume = volume;
                }
                Mode::SelectedSfxVolume
            }),
            Mode::SelectedSfxSkipTo => self.m_sfx_prop_selected_input(
                k,
                Mode::SelectedSfxVolume,
                Mode::Null,
                Mode::EditSfxSkipTo,
                |a| a.skip_to.to_string(),
            ),
            Mode::EditSfxSkipTo => self.m_sfx_prop_edit_input(k, |a, s| {
                if let Ok(skip_to) = s.parse::<f32>() {
                    a.skip_to = skip_to;
                }
                Mode::SelectedSfxSkipTo
            }),
            Mode::Null => {
                if k == KeyCode::Char('q') {
                    StateStatus::Quit
                } else {
                    StateStatus::Unaffected
                }
            }
        };
    }
}
