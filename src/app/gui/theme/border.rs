use crate::app::gui::theme::{Color, MISSING_COLOR};
use serde::Deserialize;

#[derive(Deserialize, Clone, Copy)]
pub struct Border {
    color: Color,
    width: f32,
    radius: f32,
}

impl Border {
    pub(super) fn new(radius: f32) -> Self {
        Self {
            color: MISSING_COLOR,
            width: 0.0,
            radius,
        }
    }

    pub(super) fn none() -> Self {
        Self {
            color: MISSING_COLOR,
            width: 0.0,
            radius: 0.0,
        }
    }

    pub(super) fn colored(radius: f32, color: Color, width: f32) -> Self {
        Self {
            color,
            width,
            radius,
        }
    }
}

impl From<Border> for iced::Border {
    fn from(value: Border) -> Self {
        Self {
            color: value.color.into(),
            width: value.width,
            radius: value.radius.into(),
        }
    }
}
