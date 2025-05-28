use crate::App;
use rdev::{listen, Event, EventType, Key};
use std::{
    sync::{LazyLock, Mutex},
    thread,
};

struct HotkeyHandler {
    ctrl_held: bool,
    alt_held: bool,
    app: &'static mut App,
}

static HANDLER: LazyLock<Mutex<Option<HotkeyHandler>>> = LazyLock::new(|| Mutex::new(None));

impl HotkeyHandler {
    fn new(app: &'static mut App) -> HotkeyHandler {
        HotkeyHandler {
            ctrl_held: false,
            alt_held: false,
            app,
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

        match key {
            Key::KeyT => self.app.search_and_play(),
            _ => {}
        }
    }
}

pub fn start(app: &'static mut App) {
    *HANDLER.lock().unwrap() = Some(HotkeyHandler::new(app));

    thread::spawn(|| {
        if let Err(err) = listen(HotkeyHandler::callback) {
            eprintln!("Global hotkey error: {:?}", err);
        }
    });
}
