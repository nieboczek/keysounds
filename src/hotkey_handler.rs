use crate::app::Action;
use rdev::{Event, EventType, Key};
use std::sync::{Arc, Mutex};
use std::thread;

struct HotkeyHandler {
    ctrl_pressed: bool,
    alt_pressed: bool,
    shift_pressed: bool,
    channel: Arc<Mutex<Action>>,
}

impl HotkeyHandler {
    #[inline]
    const fn new(channel: Arc<Mutex<Action>>) -> HotkeyHandler {
        HotkeyHandler {
            ctrl_pressed: false,
            alt_pressed: false,
            shift_pressed: false,
            channel,
        }
    }

    #[inline]
    fn emit_event(&mut self, event: Event) {
        let pressed = matches!(event.event_type, EventType::KeyPress(_));

        if let EventType::KeyPress(key) | EventType::KeyRelease(key) = event.event_type {
            match key {
                Key::Alt => self.alt_pressed = pressed,
                Key::ControlLeft | Key::ControlRight => self.ctrl_pressed = pressed,
                Key::ShiftLeft | Key::ShiftRight => self.shift_pressed = pressed,
                _ if pressed => self.pressed(key),
                _ => {}
            }
        }
    }

    #[inline]
    fn pressed(&self, key: Key) {
        if !self.ctrl_pressed || !self.alt_pressed {
            return;
        }

        let action = match key {
            Key::KeyT => Action::SearchAndPlay,
            Key::KeyY => Action::SkipToPart,
            Key::KeyS => Action::StopSfx,
            Key::KeyG => Action::ToggleShitMic,
            _ => return,
        };

        let mut guard = self.channel.lock().unwrap();
        *guard = action;
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
