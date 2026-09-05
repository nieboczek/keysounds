use crate::app::gui::view::Element;
use iced::{Shadow, overlay::menu, widget::space};
use serde::Deserialize;

mod border;
mod color;

pub use self::{border::Border, color::Color};

pub const MISSING_COLOR: Color = Color::from_rgb(1.0, 0.0, 1.0);

#[derive(Deserialize, Clone)]
pub struct Theme {
    pub bg: Color,
    pub text: Color,
    pub separator: Color,
    pub sounds: Sounds,
    pub search: Search,
    pub tabs: Tabs,
    pub filter_presets: FilterPresets,
    pub player_overlay: PlayerOverlay,
    pub settings: Settings,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Sounds {
    pub bg: Color,
    pub bg_hovered: Color,
    pub border: Border,
    pub border_hovered: Border,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Search {
    pub bg: Color,
    pub text: Color,
    pub placeholder: Color,
    pub selection: Color,
    pub border: Border,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Tabs {
    pub underline: Color,
    pub text: Color,
    pub text_hovered: Color,
    pub text_active: Color,
}

#[derive(Deserialize, Clone, Copy)]
pub struct FilterPresets {
    pub bg: Color,
    pub bg_hovered: Color,
    pub bg_active: Color,
    pub effects: Color,
    pub keybind: Color,
    pub border: Border,
    pub border_hovered: Border,
    pub border_active: Border,
    pub icons: Color,
    pub icons_hovered: Color,
    pub name: Color,
    pub name_disabled: Color,
    pub add_new_text: Color,
    pub toggle_fg_on: Color,
    pub toggle_fg_off: Color,
    pub toggle_bg_on: Color,
    pub toggle_bg_off: Color,
    pub property_names: Color,
    pub property_values: Color,
    pub property_slider_bg: Color,
    pub property_slider_head: Color,
}

#[derive(Deserialize, Clone, Copy)]
pub struct PlayerOverlay {
    pub bg: Color,
    pub border: Border,
    pub time_bg: Color,
    pub time_border: Border,
    pub stop_icon: Color,
    pub stop_bg: Color,
    pub stop_bg_hovered: Color,
    pub stop_border: Border,
    pub progress_bar: Color,
    pub progress_bar_bg: Color,
    pub progress_bar_border: Border,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Settings {
    pub names: Color,
    pub values: Color,
    pub value_bg: Color,
    pub selected_value_bg: Color,
    pub icons: Color,
    pub icons_hovered: Color,
    pub value_borders: Border,
}

impl Default for Theme {
    fn default() -> Self {
        let bg = Color::hex(0x0e1110);
        let bg_light = Color::hex(0x151a18);
        let bg_lighter = Color::hex(0x1c2320);
        let bg_active = Color::hex(0x362d1d);
        let text = Color::hex(0xedf1ec);
        let text_dark = Color::hex(0x8c9689);
        let text_darker = Color::hex(0x565f58);
        let border_color = Color::hex(0x202824);
        let border = Border::new(border_color, 8.0);
        let active_color = Color::hex(0xd99a4e);
        let border_active = Border::new(active_color, 8.0);
        let border_hovered_color = Color::hex(0x2a332d);
        let border_hovered = Border::new(border_hovered_color, 8.0);

        Self {
            bg,
            text,
            separator: border_color,
            sounds: Sounds {
                bg: bg_light,
                bg_hovered: bg_lighter,
                border,
                border_hovered,
            },
            search: Search {
                bg: bg_lighter,
                text,
                placeholder: text_darker,
                selection: bg_active,
                border: border_hovered,
            },
            tabs: Tabs {
                underline: active_color,
                text: text_dark,
                text_hovered: text,
                text_active: text,
            },
            filter_presets: FilterPresets {
                bg: bg_light,
                bg_hovered: bg_lighter,
                bg_active,
                effects: text_dark,
                keybind: text_darker,
                border,
                border_hovered,
                border_active,
                icons: text_dark,
                icons_hovered: text,
                name: text,
                name_disabled: text_dark,
                add_new_text: text_dark,
                toggle_fg_on: bg,
                toggle_fg_off: text_dark,
                toggle_bg_on: active_color,
                toggle_bg_off: bg,
                property_names: text_dark,
                property_values: text_dark,
                property_slider_bg: bg_lighter,
                property_slider_head: text,
            },
            player_overlay: PlayerOverlay {
                bg: bg_light,
                border: border_hovered,
                time_bg: bg_light,
                time_border: border_hovered,
                stop_icon: text,
                stop_bg: bg_light,
                stop_bg_hovered: bg_lighter,
                stop_border: border_hovered,
                progress_bar: active_color,
                progress_bar_bg: bg,
                progress_bar_border: border_hovered,
            },
            settings: Settings {
                names: text,
                values: text,
                value_bg: bg_light,
                selected_value_bg: bg_lighter,
                icons: text_darker,
                icons_hovered: text_dark,
                value_borders: border,
            },
        }
    }
}

impl iced::theme::Base for Theme {
    fn base(&self) -> iced::theme::Style {
        iced::theme::Style {
            background_color: self.bg.into(),
            text_color: self.text.into(),
        }
    }

    fn default(_preference: iced::theme::Mode) -> Self {
        Default::default()
    }

    fn mode(&self) -> iced::theme::Mode {
        iced::theme::Mode::Dark
    }

    fn name(&self) -> &str {
        "Theme"
    }

    fn palette(&self) -> Option<iced::theme::Palette> {
        None
    }
}

pub fn v_separator<'a>() -> Element<'a> {
    container(space::vertical())
        .width(1)
        .style(container_separator)
        .into()
}

#[expect(unused)] // remove if this is used at some point
pub fn h_separator<'a>() -> Element<'a> {
    container(space::horizontal())
        .height(1)
        .style(container_separator)
        .into()
}

macro_rules! impl_catalog {
    ($($mod_name:ident => $fn:ident,)*) => {
        $(
        use iced::widget::$mod_name;
        impl $mod_name::Catalog for Theme {
            type Class<'a> = $mod_name::StyleFn<'a, Self>;

            fn default<'a>() -> Self::Class<'a> {
                Box::new($fn)
            }

            fn style(&self, item: &Self::Class<'_>) -> $mod_name::Style {
                item(self)
            }
        }
        )*
    };
}

macro_rules! impl_catalog_with_status {
    ($($mod_name:ident => $fn:ident,)*) => {
        $(
        use iced::widget::$mod_name;
        impl $mod_name::Catalog for Theme {
            type Class<'a> = $mod_name::StyleFn<'a, Self>;

            fn default<'a>() -> <Self as $mod_name::Catalog>::Class<'a> {
                Box::new($fn)
            }

            fn style(&self, class: &<Self as $mod_name::Catalog>::Class<'_>, status: $mod_name::Status) -> $mod_name::Style {
                class(self, status)
            }
        }
        )*
    };
}

impl_catalog! {
    text => text_default,
    container => container_default,
    progress_bar => progress_bar_default,
}

impl_catalog_with_status! {
    button => button_default,
    scrollable => scrollable_default,
    svg => svg_default,
    text_input => text_input_default,
    slider => slider_default,
    toggler => toggler_default,
    pick_list => pick_list_default,
}

impl menu::Catalog for Theme {
    type Class<'a> = menu::StyleFn<'a, Self>;

    fn default<'a>() -> <Self as menu::Catalog>::Class<'a> {
        Box::new(menu_default)
    }

    fn style(&self, class: &<Self as menu::Catalog>::Class<'_>) -> menu::Style {
        class(self)
    }
}

pub fn slider_default(theme: &Theme, _status: slider::Status) -> slider::Style {
    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                theme.filter_presets.property_slider_bg.into(),
                theme.filter_presets.property_slider_bg.into(),
            ),
            width: 4.0,
            border: Border::none().into(),
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 6.0 },
            background: theme.filter_presets.property_slider_head.into(),
            border_width: 0.0,
            border_color: MISSING_COLOR.into(),
        },
    }
}

pub fn text_input_default(theme: &Theme, _status: text_input::Status) -> text_input::Style {
    let search = theme.search;
    text_input::Style {
        background: search.bg.into(),
        border: search.border.into(),
        icon: MISSING_COLOR.into(),
        placeholder: search.placeholder.into(),
        value: search.text.into(),
        selection: search.selection.into(),
    }
}

pub fn progress_bar_default(theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: theme.player_overlay.progress_bar_bg.into(),
        bar: theme.player_overlay.progress_bar.into(),
        border: theme.player_overlay.progress_bar_border.into(),
    }
}

pub fn svg_default(theme: &Theme, _status: svg::Status) -> svg::Style {
    svg::Style {
        color: theme.text.into(),
    }
}

pub fn svg_filter(theme: &Theme, status: svg::Status) -> svg::Style {
    svg::Style {
        color: match status {
            svg::Status::Idle => theme.filter_presets.icons.into(),
            svg::Status::Hovered => theme.filter_presets.icons_hovered.into(),
        },
    }
}

pub fn svg_keybind_x(theme: &Theme, status: svg::Status) -> svg::Style {
    svg::Style {
        color: match status {
            svg::Status::Idle => theme.settings.icons.into(),
            svg::Status::Hovered => theme.settings.icons_hovered.into(),
        },
    }
}

pub fn svg_stop(theme: &Theme, _status: svg::Status) -> svg::Style {
    svg::Style {
        color: theme.player_overlay.stop_icon.into(),
    }
}

fn text_color(color: Color) -> text::Style {
    text::Style {
        color: color.into(),
    }
}

pub fn text_default(_theme: &Theme) -> text::Style {
    text::Style::default()
}

pub fn text_setting_name(theme: &Theme) -> text::Style {
    text_color(theme.settings.names)
}

pub fn text_filter_property_name(theme: &Theme) -> text::Style {
    text_color(theme.filter_presets.property_names)
}

pub fn text_filter_property_value(theme: &Theme) -> text::Style {
    text_color(theme.filter_presets.property_values)
}

pub fn text_filter_preset_effects(theme: &Theme) -> text::Style {
    text_color(theme.filter_presets.effects)
}

pub fn text_filter_preset_keybind(theme: &Theme) -> text::Style {
    text_color(theme.filter_presets.keybind)
}

pub fn container_default(_theme: &Theme) -> container::Style {
    container::Style::default()
}

pub fn container_separator(theme: &Theme) -> container::Style {
    container::Style {
        background: theme.separator.into(),
        ..Default::default()
    }
}

pub fn container_tab_underline(theme: &Theme) -> container::Style {
    container::Style {
        background: theme.tabs.underline.into(),
        ..Default::default()
    }
}

pub fn container_time(theme: &Theme) -> container::Style {
    container::Style {
        background: theme.player_overlay.time_bg.into(),
        border: theme.player_overlay.time_border.into(),
        ..Default::default()
    }
}

pub fn container_overlay(theme: &Theme) -> container::Style {
    container::Style {
        background: theme.player_overlay.bg.into(),
        border: theme.player_overlay.border.into(),
        ..Default::default()
    }
}

pub fn container_filter_preset(theme: &Theme) -> container::Style {
    container::Style {
        background: theme.filter_presets.bg.into(),
        border: theme.filter_presets.border.into(),
        ..Default::default()
    }
}

pub fn button_default(theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        text_color: theme.text.into(),
        ..Default::default()
    }
}

pub fn button_stop(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        text_color: theme.text.into(),
        border: theme.player_overlay.stop_border.into(),
        background: match status {
            button::Status::Hovered | button::Status::Pressed => {
                theme.player_overlay.stop_bg_hovered.into()
            }
            _ => theme.player_overlay.stop_bg.into(),
        },
        ..Default::default()
    }
}

pub fn button_setting_value(theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        text_color: theme.settings.values.into(),
        background: theme.settings.value_bg.into(),
        border: theme.settings.value_borders.into(),
        ..Default::default()
    }
}

pub fn button_setting_recording(theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        text_color: theme.settings.values.into(),
        background: theme.settings.selected_value_bg.into(),
        border: theme.settings.value_borders.into(),
        ..Default::default()
    }
}

pub fn button_sound(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        text_color: theme.text.into(),
        background: match status {
            button::Status::Hovered | button::Status::Pressed => theme.sounds.bg_hovered.into(),
            _ => theme.sounds.bg.into(),
        },
        border: match status {
            button::Status::Hovered | button::Status::Pressed => theme.sounds.border_hovered.into(),
            _ => theme.sounds.border.into(),
        },
        ..Default::default()
    }
}

pub fn scrollable_default(theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    let rail = scrollable::Rail {
        background: None,
        border: Border::none().into(),
        scroller: scrollable::Scroller {
            background: theme.text.into(),
            border: Border::uncolored(2.0).into(),
        },
    };

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Color::from_rgb(1.0, 1.0, 1.0).into(),
            border: Border::none().into(),
            shadow: Shadow::default(),
            icon: MISSING_COLOR.into(),
        },
    }
}

pub fn toggler_default(_theme: &Theme, _status: toggler::Status) -> toggler::Style {
    toggler::Style {
        background: MISSING_COLOR.into(),
        background_border_width: 0.0,
        background_border_color: MISSING_COLOR.into(),
        foreground: MISSING_COLOR.into(),
        foreground_border_width: 0.0,
        foreground_border_color: MISSING_COLOR.into(),
        text_color: None,
        border_radius: None,
        padding_ratio: 0.0,
    }
}

pub fn pick_list_default(theme: &Theme, _status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: theme.settings.names.into(),
        placeholder_color: MISSING_COLOR.into(),
        handle_color: theme.settings.values.into(),
        background: theme.settings.value_bg.into(),
        border: theme.settings.value_borders.into(),
    }
}

pub fn menu_default(theme: &Theme) -> menu::Style {
    menu::Style {
        background: theme.settings.value_bg.into(),
        border: theme.settings.value_borders.into(),
        text_color: theme.settings.values.into(),
        selected_text_color: theme.settings.values.into(),
        selected_background: theme.settings.selected_value_bg.into(),
        shadow: Shadow::default(),
    }
}
