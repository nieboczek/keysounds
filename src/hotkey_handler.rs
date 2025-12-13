use crate::app::Action;
use ratatui::crossterm::event::KeyCode;
use rdev::{Event, EventType, Key};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

struct HotkeyHandler {
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    keybinds: HashMap<KeyCombination, Action>,
    channel: Arc<Mutex<Action>>,
}

#[derive(PartialEq, Eq, Hash)]
struct KeyCombination {
    shift: bool,
    ctrl: bool,
    alt: bool,
    key: Key,
}

impl HotkeyHandler {
    #[inline]
    fn new(channel: Arc<Mutex<Action>>) -> HotkeyHandler {
        HotkeyHandler {
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            keybinds: HashMap::new(),
            channel,
        }
    }

    #[inline]
    fn emit_event(&mut self, event: Event) {
        let pressed = matches!(event.event_type, EventType::KeyPress(_));

        if let EventType::KeyPress(key) | EventType::KeyRelease(key) = event.event_type {
            match key {
                Key::ShiftLeft | Key::ShiftRight => self.shift_pressed = pressed,
                Key::ControlLeft | Key::ControlRight => self.ctrl_pressed = pressed,
                Key::Alt => self.alt_pressed = pressed,
                _ if pressed => self.pressed(key),
                _ => {}
            }
        }
    }

    #[inline]
    fn pressed(&mut self, key: Key) {
        let mut guard = self.channel.lock().unwrap();

        let old = std::mem::replace(&mut *guard, Action::None);
        if let Action::SetKeybinds(keybinds) = old {
            for keybind in keybinds {
                let key = KeyCombination {
                    shift: keybind.shift,
                    ctrl: keybind.ctrl,
                    alt: keybind.alt,
                    key: keybind.key,
                };
                self.keybinds.insert(key, keybind.action);
            }
        }

        let combination = KeyCombination {
            shift: self.shift_pressed,
            ctrl: self.ctrl_pressed,
            alt: self.alt_pressed,
            key: key,
        };

        if let Some(action) = self.keybinds.get(&combination) {
            *guard = action.clone();
        }
    }
}

#[inline]
pub fn start() -> Arc<Mutex<Action>> {
    let action_channel = Arc::new(Mutex::new(Action::None));
    let channel_clone = Arc::clone(&action_channel);

    thread::spawn(move || {
        let mut hotkey_handler = HotkeyHandler::new(channel_clone);

        if let Err(err) = rdev::listen(move |event| hotkey_handler.emit_event(event)) {
            eprintln!("Global hotkey error: {err:?}");
        }
    });

    action_channel
}

pub fn map_key(key_code: KeyCode) -> Option<Key> {
    Some(match key_code {
        KeyCode::F(num) => match num {
            1 => Key::F1,
            2 => Key::F2,
            3 => Key::F3,
            4 => Key::F4,
            5 => Key::F5,
            6 => Key::F6,
            7 => Key::F7,
            8 => Key::F8,
            9 => Key::F9,
            10 => Key::F10,
            11 => Key::F11,
            12 => Key::F12,
            _ => unreachable!(),
        },
        KeyCode::Backspace => Key::Backspace,
        KeyCode::CapsLock => Key::CapsLock,
        KeyCode::Delete => Key::Delete,
        KeyCode::Left => Key::LeftArrow,
        KeyCode::Right => Key::RightArrow,
        KeyCode::Down => Key::DownArrow,
        KeyCode::Up => Key::UpArrow,
        KeyCode::End => Key::End,
        KeyCode::Enter => Key::Return,
        KeyCode::Insert => Key::Insert,
        KeyCode::Esc => Key::Escape,
        KeyCode::Home => Key::Home,
        KeyCode::NumLock => Key::NumLock,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PrintScreen => Key::PrintScreen,
        KeyCode::Pause => Key::Pause,
        KeyCode::Tab => Key::Tab,
        KeyCode::Char(ch) => match ch {
            'a' => Key::KeyA,
            'b' => Key::KeyB,
            'c' => Key::KeyC,
            'd' => Key::KeyD,
            'e' => Key::KeyE,
            'f' => Key::KeyF,
            'g' => Key::KeyG,
            'h' => Key::KeyH,
            'i' => Key::KeyI,
            'j' => Key::KeyJ,
            'k' => Key::KeyK,
            'l' => Key::KeyL,
            'm' => Key::KeyM,
            'n' => Key::KeyN,
            'o' => Key::KeyO,
            'p' => Key::KeyP,
            'q' => Key::KeyQ,
            'r' => Key::KeyR,
            's' => Key::KeyS,
            't' => Key::KeyT,
            'u' => Key::KeyU,
            'v' => Key::KeyV,
            'w' => Key::KeyW,
            'x' => Key::KeyX,
            'y' => Key::KeyY,
            'z' => Key::KeyZ,
            '0' => Key::Num0,
            '1' => Key::Num1,
            '2' => Key::Num2,
            '3' => Key::Num3,
            '4' => Key::Num4,
            '5' => Key::Num5,
            '6' => Key::Num6,
            '7' => Key::Num7,
            '8' => Key::Num8,
            '9' => Key::Num9,
            _ => return None,
        },
        _ => return None,
    })
}
