use crate::app::gui::view::theme::{Color, MISSING_COLOR};
use serde::Deserialize;

#[derive(Deserialize, Clone, Copy)]
pub struct Border {
    color: Color,
    width: f32,
    radius: f32,
}

impl Border {
    pub(super) fn uncolored(radius: f32) -> Self {
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

    pub(super) fn new(color: Color, radius: f32) -> Self {
        Self {
            color,
            width: 1.0,
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
