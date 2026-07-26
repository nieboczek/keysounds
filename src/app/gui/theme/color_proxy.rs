use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "ColorString", into = "ColorString")]
pub struct Color(iced::Color);

impl Color {
    pub(super) const fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self(iced::Color::from_rgb(r, g, b))
    }
}

impl Default for Color {
    fn default() -> Self {
        Self(iced::Color::BLACK)
    }
}

impl From<iced::Color> for Color {
    fn from(value: iced::Color) -> Self {
        Self(value)
    }
}

impl From<Color> for iced::Color {
    fn from(value: Color) -> Self {
        value.0
    }
}

impl From<Color> for iced::Background {
    fn from(value: Color) -> Self {
        Self::Color(value.0)
    }
}

impl From<Color> for Option<iced::Color> {
    fn from(value: Color) -> Self {
        Some(value.0)
    }
}

impl From<Color> for Option<iced::Background> {
    fn from(value: Color) -> Self {
        Some(iced::Background::Color(value.0))
    }
}

impl From<Color> for ColorString {
    fn from(value: Color) -> Self {
        fn encode(v: f32) -> String {
            format!("{:02x}", (v * 255.0) as u8)
        }

        let color = value.0;
        let mut str = encode(color.r);
        str.push_str(&encode(color.g));
        str.push_str(&encode(color.b));
        if color.a != 1.0 {
            str.push_str(&encode(color.a));
        }

        Self(str)
    }
}

#[derive(Serialize, Deserialize)]
struct ColorString(String);

impl TryFrom<ColorString> for Color {
    type Error = ColorParseError;

    fn try_from(value: ColorString) -> Result<Self, Self::Error> {
        value
            .0
            .parse::<iced::Color>()
            .map_err(|e| ColorParseError(e.to_string()))
            .map(|color| color.into())
    }
}

struct ColorParseError(String);

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
