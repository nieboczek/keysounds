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
            self.gui_scale_pick_list(),
        ])
        .into()
    }

    fn gui_scale_pick_list(&self) -> Element<'_> {
        #[derive(Clone, PartialEq)]
        struct ScaleWrapper(f32);

        impl std::fmt::Display for ScaleWrapper {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}%", (self.0 * 100.0) as usize)
            }
        }

        let scales = crate::app::gui::SCALES.map(|x| ScaleWrapper(x));

        row([
            Self::setting_name("GUI Scale"),
            container(pick_list(
                scales,
                Some(ScaleWrapper(self.config.gui_scale)),
                |wrapper| Message::SetGuiScale(wrapper.0),
            ))
            .align_right(Length::Fill)
            .into(),
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
            Self::setting_name(name),
            container(pick_list(options, Some(selected), on_select))
                .align_right(Length::Fill)
                .into(),
        ])
        .into()
    }

    fn setting_name(name: &str) -> Element<'_> {
        text(name).style(theme::text_setting_name).into()
    }
}
