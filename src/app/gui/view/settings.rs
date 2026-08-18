use crate::app::{
    App, DeviceOption,
    gui::{
        Message,
        view::{Element, theme},
    },
};
use iced::{
    Length,
    widget::{column, container, pick_list, row, text},
};

impl App {
    pub(super) fn settings_page(&self) -> Element<'_> {
        column([
            self.device_pick_list(
                "Microphone Device",
                &self.input_devices,
                &self.mic_device,
                Message::SetMicDevice,
            ),
            self.device_pick_list(
                "Output Device",
                &self.output_devices,
                &self.out_device,
                Message::SetOutDevice,
            ),
            self.device_pick_list(
                "Virtual Output Device",
                &self.output_devices,
                &self.virtual_out_device,
                Message::SetVirtualOutDevice,
            ),
        ])
        .into()
    }

    fn device_pick_list<'a>(
        &'a self,
        name: &'a str,
        options: &'a [DeviceOption],
        selected: &'a DeviceOption,
        on_select: impl Fn(DeviceOption) -> Message + 'a,
    ) -> Element<'a> {
        row([
            text(name).style(theme::text_setting_name).into(),
            container(pick_list(options, Some(selected), on_select))
                .align_right(Length::Fill)
                .into(),
        ])
        .into()
    }
}
