use crate::app::config::Keybind;
use rdev::{EventType, Key};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

pub(super) struct KeybindListener {
    rx: Receiver<Keybind>,
}

impl KeybindListener {
    pub(super) fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let mut listener = InternalListener::new(tx);
        thread::spawn(|| {
            if let Err(err) = rdev::listen(move |event| listener.new_event(event)) {
                tracing::error!(?err, "Global keybind listener error");
            }
        });

        Self { rx }
    }

    pub fn try_recv(&self) -> Option<Keybind> {
        self.rx.try_recv().ok()
    }
}

struct InternalListener {
    tx: Sender<Keybind>,
    ctrl: bool,
    alt: bool,
    shift: bool,
}

impl InternalListener {
    fn new(tx: Sender<Keybind>) -> Self {
        Self {
            tx,
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    fn new_event(&mut self, event: rdev::Event) {
        let pressed = matches!(event.event_type, EventType::KeyPress(_));

        if let EventType::KeyPress(key) | EventType::KeyRelease(key) = event.event_type {
            match key {
                Key::ControlLeft | Key::ControlRight => self.ctrl = pressed,
                Key::Alt => self.alt = pressed,
                Key::ShiftLeft | Key::ShiftRight => self.shift = pressed,
                key if pressed => {
                    let result = self.tx.send(Keybind {
                        ctrl: self.ctrl,
                        alt: self.alt,
                        shift: self.shift,
                        key,
                    });

                    if result.is_err() {
                        panic!("Expected panic: Shutting down InternalListener");
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn write_key_str(key: Key, f: &mut impl std::fmt::Write) -> std::fmt::Result {
    match key {
        Key::KeyA => write!(f, "A"),
        Key::KeyB => write!(f, "B"),
        Key::KeyC => write!(f, "C"),
        Key::KeyD => write!(f, "D"),
        Key::KeyE => write!(f, "E"),
        Key::KeyF => write!(f, "F"),
        Key::KeyG => write!(f, "G"),
        Key::KeyH => write!(f, "H"),
        Key::KeyI => write!(f, "I"),
        Key::KeyJ => write!(f, "J"),
        Key::KeyK => write!(f, "K"),
        Key::KeyL => write!(f, "L"),
        Key::KeyM => write!(f, "M"),
        Key::KeyN => write!(f, "N"),
        Key::KeyO => write!(f, "O"),
        Key::KeyP => write!(f, "P"),
        Key::KeyQ => write!(f, "Q"),
        Key::KeyR => write!(f, "R"),
        Key::KeyS => write!(f, "S"),
        Key::KeyT => write!(f, "T"),
        Key::KeyU => write!(f, "U"),
        Key::KeyV => write!(f, "V"),
        Key::KeyW => write!(f, "W"),
        Key::KeyX => write!(f, "X"),
        Key::KeyY => write!(f, "Y"),
        Key::KeyZ => write!(f, "Z"),
        Key::Num0 => write!(f, "0"),
        Key::Num1 => write!(f, "1"),
        Key::Num2 => write!(f, "2"),
        Key::Num3 => write!(f, "3"),
        Key::Num4 => write!(f, "4"),
        Key::Num5 => write!(f, "5"),
        Key::Num6 => write!(f, "6"),
        Key::Num7 => write!(f, "7"),
        Key::Num8 => write!(f, "8"),
        Key::Num9 => write!(f, "9"),
        Key::F1 => write!(f, "F1"),
        Key::F2 => write!(f, "F2"),
        Key::F3 => write!(f, "F3"),
        Key::F4 => write!(f, "F4"),
        Key::F5 => write!(f, "F5"),
        Key::F6 => write!(f, "F6"),
        Key::F7 => write!(f, "F7"),
        Key::F8 => write!(f, "F8"),
        Key::F9 => write!(f, "F9"),
        Key::F10 => write!(f, "F10"),
        Key::F11 => write!(f, "F11"),
        Key::F12 => write!(f, "F12"),
        Key::Pause => write!(f, "Pause"),
        Key::Space => write!(f, "Space"),
        Key::Backspace => write!(f, "Backspace"),
        Key::Return => write!(f, "Enter"),
        Key::Tab => write!(f, "Tab"),
        Key::Escape => write!(f, "Esc"),
        Key::Delete => write!(f, "Del"),
        Key::Insert => write!(f, "Ins"),
        Key::Home => write!(f, "Home"),
        Key::End => write!(f, "End"),
        Key::PageUp => write!(f, "PgUp"),
        Key::PageDown => write!(f, "PgDown"),
        Key::LeftArrow => write!(f, "Left"),
        Key::RightArrow => write!(f, "Right"),
        Key::UpArrow => write!(f, "Up"),
        Key::DownArrow => write!(f, "Down"),
        Key::Minus => write!(f, "-"),
        Key::Equal => write!(f, "="),
        Key::LeftBracket => write!(f, "["),
        Key::RightBracket => write!(f, "]"),
        Key::BackSlash => write!(f, "\\"),
        Key::SemiColon => write!(f, ";"),
        Key::Quote => write!(f, "'"),
        Key::Comma => write!(f, ","),
        Key::Dot => write!(f, "."),
        Key::Slash => write!(f, "/"),
        Key::BackQuote => write!(f, "`"),
        Key::Kp0 => write!(f, "Kp0"),
        Key::Kp1 => write!(f, "Kp1"),
        Key::Kp2 => write!(f, "Kp2"),
        Key::Kp3 => write!(f, "Kp3"),
        Key::Kp4 => write!(f, "Kp4"),
        Key::Kp5 => write!(f, "Kp5"),
        Key::Kp6 => write!(f, "Kp6"),
        Key::Kp7 => write!(f, "Kp7"),
        Key::Kp8 => write!(f, "Kp8"),
        Key::Kp9 => write!(f, "Kp9"),
        Key::KpMultiply => write!(f, "KpMultiply"),
        Key::KpPlus => write!(f, "KpPlus"),
        Key::KpDelete => write!(f, "KpDelete"),
        Key::KpDivide => write!(f, "KpDivide"),
        Key::KpReturn => write!(f, "KpEnter"),
        Key::KpMinus => write!(f, "KpMinus"),
        Key::CapsLock => write!(f, "CapsLock"),
        Key::ScrollLock => write!(f, "ScrollLock"),
        Key::NumLock => write!(f, "NumLock"),
        Key::PrintScreen => write!(f, "PrintScreen"),
        Key::Unknown(code) => write!(f, "Code:{code}"),
        key => panic!("Shouldn't have to display {key:?}"),
    }
}

/// Accepts a lowercase key str.
pub fn parse_key(str: &str) -> Result<Key, String> {
    match str {
        "a" => Ok(Key::KeyA),
        "b" => Ok(Key::KeyB),
        "c" => Ok(Key::KeyC),
        "d" => Ok(Key::KeyD),
        "e" => Ok(Key::KeyE),
        "f" => Ok(Key::KeyF),
        "g" => Ok(Key::KeyG),
        "h" => Ok(Key::KeyH),
        "i" => Ok(Key::KeyI),
        "j" => Ok(Key::KeyJ),
        "k" => Ok(Key::KeyK),
        "l" => Ok(Key::KeyL),
        "m" => Ok(Key::KeyM),
        "n" => Ok(Key::KeyN),
        "o" => Ok(Key::KeyO),
        "p" => Ok(Key::KeyP),
        "q" => Ok(Key::KeyQ),
        "r" => Ok(Key::KeyR),
        "s" => Ok(Key::KeyS),
        "t" => Ok(Key::KeyT),
        "u" => Ok(Key::KeyU),
        "v" => Ok(Key::KeyV),
        "w" => Ok(Key::KeyW),
        "x" => Ok(Key::KeyX),
        "y" => Ok(Key::KeyY),
        "z" => Ok(Key::KeyZ),
        "0" => Ok(Key::Num0),
        "1" => Ok(Key::Num1),
        "2" => Ok(Key::Num2),
        "3" => Ok(Key::Num3),
        "4" => Ok(Key::Num4),
        "5" => Ok(Key::Num5),
        "6" => Ok(Key::Num6),
        "7" => Ok(Key::Num7),
        "8" => Ok(Key::Num8),
        "9" => Ok(Key::Num9),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        "pause" => Ok(Key::Pause),
        "space" => Ok(Key::Space),
        "return" | "enter" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "escape" | "esc" => Ok(Key::Escape),
        "delete" | "del" => Ok(Key::Delete),
        "insert" | "ins" => Ok(Key::Insert),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" | "pgup" => Ok(Key::PageUp),
        "pagedown" | "pgdown" | "pgdn" => Ok(Key::PageDown),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "-" => Ok(Key::Minus),
        "=" => Ok(Key::Equal),
        "[" => Ok(Key::LeftBracket),
        "]" => Ok(Key::RightBracket),
        "\\" => Ok(Key::BackSlash),
        ";" => Ok(Key::SemiColon),
        "'" => Ok(Key::Quote),
        "," => Ok(Key::Comma),
        "." => Ok(Key::Dot),
        "/" => Ok(Key::Slash),
        "`" => Ok(Key::BackQuote),
        "kp0" | "keypad0" => Ok(Key::Kp0),
        "kp1" | "keypad1" => Ok(Key::Kp1),
        "kp2" | "keypad2" => Ok(Key::Kp2),
        "kp3" | "keypad3" => Ok(Key::Kp3),
        "kp4" | "keypad4" => Ok(Key::Kp4),
        "kp5" | "keypad5" => Ok(Key::Kp5),
        "kp6" | "keypad6" => Ok(Key::Kp6),
        "kp7" | "keypad7" => Ok(Key::Kp7),
        "kp8" | "keypad8" => Ok(Key::Kp8),
        "kp9" | "keypad9" => Ok(Key::Kp9),
        "kpmultiply" => Ok(Key::KpMultiply),
        "kpplus" => Ok(Key::KpPlus),
        "kpdelete" => Ok(Key::KpDelete),
        "kpdivide" => Ok(Key::KpDivide),
        "kpenter" => Ok(Key::KpReturn),
        "kpminus" => Ok(Key::KpMinus),
        "capslock" => Ok(Key::CapsLock),
        "scrolllock" => Ok(Key::ScrollLock),
        "numlock" => Ok(Key::NumLock),
        "printscreen" => Ok(Key::PrintScreen),
        str => {
            if let Some(code) = str.strip_prefix("code:") {
                let code = code.parse::<u32>().map_err(|e| e.to_string())?;
                Ok(Key::Unknown(code))
            } else {
                Err(format!("Unknown key: {str}"))
            }
        }
    }
}
