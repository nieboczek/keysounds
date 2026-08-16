use crate::app::{
    App,
    gui::{
        Message,
        view::{Element, theme},
    },
};
use iced::{
    Length,
    widget::{button, column, container, pick_list, row, scrollable, text, text_input},
};

impl App {
    pub(super) fn settings_page(&self) -> Element<'_> {
        column([row([
            text("Input Device").style(theme::text_setting_name).into(),
            container(text("Value")).align_right(Length::Fill).into(),
        ])
        .into()])
        .into()
    }
}
