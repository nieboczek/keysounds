use rdev::{listen, Event, EventType, Key};
use std::{
    sync::{mpsc::Sender, LazyLock, Mutex},
    thread,
};

use crate::app::Action;

struct HotkeyHandler {
    ctrl_held: bool,
    alt_held: bool,
    sender: Sender<Action>,
}

static HANDLER: LazyLock<Mutex<Option<HotkeyHandler>>> = LazyLock::new(|| Mutex::new(None));

impl HotkeyHandler {
    fn new(sender: Sender<Action>) -> HotkeyHandler {
        HotkeyHandler {
            ctrl_held: false,
            alt_held: false,
            sender,
        }
    }

    fn callback(event: Event) {
        let pressed = matches!(event.event_type, EventType::KeyPress(_));

        if let EventType::KeyPress(key) | EventType::KeyRelease(key) = event.event_type {
            if let Some(ref mut handler) = HANDLER.lock().unwrap().as_mut() {
                match key {
                    Key::Alt => handler.alt_held = pressed,
                    Key::ControlLeft => handler.ctrl_held = pressed,
                    _ if pressed => handler.pressed(key),
                    _ => {}
                }
            }
        }
    }

    fn pressed(&mut self, key: Key) {
        if !self.ctrl_held || !self.alt_held {
            return;
        }

        let _ = match key {
            Key::KeyT => self.sender.send(Action::SearchAndPlay),
            Key::KeyY => self.sender.send(Action::SkipToPart),
            Key::KeyS => self.sender.send(Action::StopAudio),
            Key::KeyG => self.sender.send(Action::ToggleShitMic),
            _ => Ok(()),
        };
    }
}

pub fn start(sender: Sender<Action>) {
    *HANDLER.lock().unwrap() = Some(HotkeyHandler::new(sender));

    thread::spawn(|| {
        if let Err(err) = listen(HotkeyHandler::callback) {
            eprintln!("Global hotkey error: {:?}", err);
        }
    });
}
