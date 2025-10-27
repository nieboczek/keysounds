use crate::app::Action;
use rdev::{Event, EventType, Key};
use std::{
    sync::{Arc, Mutex},
    thread,
};

struct HotkeyHandler {
    ctrl_held: bool,
    alt_held: bool,
    channel: Arc<Mutex<Action>>,
}

static HANDLER: Mutex<Option<HotkeyHandler>> = Mutex::new(None);

impl HotkeyHandler {
    #[inline]
    const fn new(channel: Arc<Mutex<Action>>) -> HotkeyHandler {
        HotkeyHandler {
            ctrl_held: false,
            alt_held: false,
            channel,
        }
    }

    #[inline]
    fn callback(event: Event) {
        let pressed = matches!(event.event_type, EventType::KeyPress(_));

        if let EventType::KeyPress(key) | EventType::KeyRelease(key) = event.event_type {
            if let Some(handler) = HANDLER.lock().unwrap().as_mut() {
                match key {
                    Key::Alt => handler.alt_held = pressed,
                    Key::ControlLeft => handler.ctrl_held = pressed,
                    _ if pressed => handler.pressed(key),
                    _ => {}
                }
            }
        }
    }

    #[inline]
    fn pressed(&self, key: Key) {
        if !self.ctrl_held || !self.alt_held {
            return;
        }

        let action = match key {
            Key::KeyT => Action::SearchAndPlay,
            Key::KeyY => Action::SkipToPart,
            Key::KeyS => Action::StopAudio,
            Key::KeyG => Action::ToggleShitMic,
            _ => return,
        };

        let mut guard = self.channel.lock().unwrap();
        *guard = action;
    }
}

#[inline]
pub fn start() -> Arc<Mutex<Action>> {
    let channel = Arc::new(Mutex::new(Action::None));
    *HANDLER.lock().unwrap() = Some(HotkeyHandler::new(Arc::clone(&channel)));

    thread::spawn(|| {
        if let Err(err) = rdev::listen(HotkeyHandler::callback) {
            eprintln!("Global hotkey error: {err:?}");
        }
    });

    channel
}
