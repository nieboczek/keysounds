use crate::app::{App, Sfx, StateStatus};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

pub enum Mode {
    Normal,
    SearchSfx,
    EditConfig,
    EditInputDevice,
    EditOutputDevice,
    EditSfxs,
    SfxProp(SfxProp),
}

pub enum SfxProp {
    Selected(SfxPropType),
    Editing(SfxPropType),
}

#[derive(Clone, Copy)]
pub enum SfxPropType {
    Name,
    Path,
    Volume,
    SkipTo,
}

impl SfxPropType {
    #[inline]
    fn previous(&self) -> Option<SfxPropType> {
        match self {
            Self::Name => None,
            Self::Path => Some(Self::Name),
            Self::Volume => Some(Self::Path),
            Self::SkipTo => Some(Self::Volume),
        }
    }

    #[inline]
    fn next(&self) -> Option<SfxPropType> {
        match self {
            Self::Name => Some(Self::Path),
            Self::Path => Some(Self::Volume),
            Self::Volume => Some(Self::SkipTo),
            Self::SkipTo => None,
        }
    }

    #[inline]
    fn get_input_string(&self, sfx: &Sfx) -> String {
        match self {
            Self::Name => sfx.name.to_string(),
            Self::Path => sfx.path.to_string(),
            Self::Volume => sfx.volume.to_string(),
            Self::SkipTo => sfx.skip_to.to_string(),
        }
    }

    #[inline]
    fn set_property(&self, sfx: &mut Sfx, input: &mut String) {
        match self {
            Self::Name => sfx.name = input.split_off(0),
            Self::Path => sfx.path = input.split_off(0),
            Self::Volume => {
                if let Ok(volume) = input.parse::<f32>() {
                    sfx.volume = volume;
                }
            }
            Self::SkipTo => {
                if let Ok(skip_to) = input.parse::<f32>() {
                    sfx.skip_to = skip_to;
                }
            }
        }
    }
}

impl App {
    fn m_add_to_input(
        &mut self,
        k: KeyCode,
        handle_special: impl Fn(&mut App) -> StateStatus,
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
            _ => return handle_special(self),
        }
        StateStatus::Updated
    }

    fn m_list_input(
        &mut self,
        k: KeyCode,
        handle_special: impl Fn(&mut App) -> StateStatus,
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
            _ => return handle_special(self),
        }
        StateStatus::Updated
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
                |a| match k {
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
                        0 => a.random_sfx_triggering = !a.random_sfx_triggering,
                        1 => return StateStatus::Unaffected, // TODO (sfx list)
                        2 => return StateStatus::Unaffected, // TODO (range)
                        3 => return StateStatus::Unaffected, // separator
                        4 => {
                            a.list_state.select_first();
                            a.mode = Mode::EditConfig;
                        }
                        5 => a.load_config(),          // Reload config
                        6 => return StateStatus::Quit, // Exit
                        _ => unreachable!(),
                    }
                    StateStatus::Updated
                },
                |_| {},
            ),
            Mode::SearchSfx => match k {
                KeyCode::Char(ch) if matches!(self.mode, Mode::SearchSfx) => {
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
                |_| {
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
                            a.input = a.config.virtual_output_device.to_string();
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
                |_| StateStatus::Unaffected,
                |a| {
                    a.config.input_device = a.input.split_off(0);
                    a.mode = Mode::EditConfig;
                    StateStatus::Updated
                },
            ),
            Mode::EditOutputDevice => self.m_add_to_input(
                k,
                |_| StateStatus::Unaffected,
                |a| {
                    a.config.virtual_output_device = a.input.split_off(0);
                    a.mode = Mode::EditConfig;
                    StateStatus::Updated
                },
            ),
            Mode::EditSfxs => self.m_list_input(
                k,
                |_| StateStatus::Unaffected,
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
                            a.mode = Mode::SfxProp(SfxProp::Selected(SfxPropType::Name));
                        }
                        1.. => a.mode = Mode::SfxProp(SfxProp::Selected(SfxPropType::Name)),
                    }
                    StateStatus::Updated
                },
                |a| {
                    a.list_state.select(Some(7));
                    a.mode = Mode::EditConfig;
                },
            ),
            Mode::SfxProp(SfxProp::Selected(prop_type)) => {
                match k {
                    KeyCode::Up => {
                        if let Some(previous) = prop_type.previous() {
                            self.mode = Mode::SfxProp(SfxProp::Selected(previous));
                            StateStatus::Updated
                        } else {
                            StateStatus::Unaffected
                        }
                    }
                    KeyCode::Down => {
                        if let Some(next) = prop_type.next() {
                            self.mode = Mode::SfxProp(SfxProp::Selected(next));
                            StateStatus::Updated
                        } else {
                            StateStatus::Unaffected
                        }
                    }
                    KeyCode::Enter => {
                        self.input = prop_type.get_input_string(&self.config.sfx[self.list_state.selected().unwrap() - 1]);
                        self.mode = Mode::SfxProp(SfxProp::Editing(prop_type));
                        StateStatus::Updated
                    }
                    KeyCode::Esc => {
                        self.mode = Mode::EditSfxs;
                        StateStatus::Updated
                    }
                    _ => StateStatus::Unaffected
                }
            }
            Mode::SfxProp(SfxProp::Editing(prop_type)) => 
                self.m_add_to_input(
                    k,
                    |_| StateStatus::Unaffected,
                    |a| {
                        prop_type.set_property(&mut a.config.sfx[a.list_state.selected().unwrap() - 1], &mut a.input);
                        a.mode = Mode::SfxProp(SfxProp::Selected(prop_type));
                        StateStatus::Updated
                    },
                ),
        };
    }

    #[inline]
    fn is_separator(&self, idx: usize) -> bool {
        match self.mode {
            Mode::Normal => idx == 3,
            Mode::EditConfig => idx == 1 || idx == 5,
            Mode::EditSfxs => false,
            _ => unreachable!(),
        }
    }
}
