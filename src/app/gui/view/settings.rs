use crate::app::{
    App, DeviceOption,
    config::Keybind,
    gui::{
        KeybindTarget, Message,
        view::{Element, theme},
    },
};
use iced::{
    Alignment, Length,
    widget::{button, column, container, pick_list, row, svg, text},
};
use std::iter;

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
            self.keybind_row(
                "Search and Play Keybind",
                self.config.search_and_play_keybind,
                KeybindTarget::SearchAndPlay,
            ),
            self.keybind_row(
                "Stop Sound Keybind",
                self.config.stop_sound_keybind,
                KeybindTarget::StopSound,
            ),
        ])
        .spacing(4)
        .into()
    }

    fn keybind_row<'a>(
        &self,
        name: &'a str,
        keybind: Option<Keybind>,
        target: KeybindTarget,
    ) -> Element<'a> {
        let recording = self.recording_keybind == Some(target);

        let label = match recording {
            true => "Press keybind...".to_string(),
            false => keybind
                .map(|keybind| keybind.to_string())
                .unwrap_or_else(|| "None".to_string()),
        };

        let record_button = button(
            row(
                iter::once(text(label).into()).chain(if keybind.is_some() && !recording {
                    Some(
                        button(
                            svg(self.svgs.x.clone())
                                .width(14)
                                .style(theme::svg_keybind_x),
                        )
                        .padding(0)
                        .on_press(Message::ClearKeybind(target))
                        .into(),
                    )
                } else {
                    None
                }),
            )
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .style(if recording {
            theme::button_setting_recording
        } else {
            theme::button_setting_value
        })
        .on_press(if recording {
            Message::CancelRecordingKeybind
        } else {
            Message::StartRecordingKeybind(target)
        });

        row([
            Self::setting_name(name),
            container(record_button).align_right(Length::Fill).into(),
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
