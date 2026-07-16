use crate::app::Action;
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
            key,
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
