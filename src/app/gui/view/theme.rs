use iced::Shadow;
use serde::Deserialize;

mod border;
mod color_proxy;

pub use border::Border;
pub use color_proxy::Color;

const MISSING_COLOR: Color = Color::from_rgb(1.0, 0.0, 1.0);

#[derive(Deserialize, Clone)]
pub struct Theme {
    pub bg: Color,
    pub text: Color,
    pub icon: Color,
    pub search: Search,
    pub tab: Color,
    pub tab_hovered: Color,
    pub tab_active: Color,
    pub sound_bg: Color,
    pub sound_bg_hovered: Color,
    pub overlay_bg: Color,
    pub overlay_border: Border,
    pub player_bar_background: Color,
    pub randomly_triggered_badge_bg: Color,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Search {
    pub bg: Color,
    pub text: Color,
    pub placeholder: Color,
    pub selection: Color,
    pub border: Border,
}

impl Default for Theme {
    fn default() -> Self {
        let bg = Color::from_rgb(0.2, 0.2, 0.2);
        let text = Color::from_rgb(0.95, 0.95, 0.95);

        Self {
            bg,
            text,
            icon: Color::from_rgb(1.0, 1.0, 1.0),
            search: Search {
                bg,
                text,
                placeholder: Color::from_rgb(0.7, 0.7, 0.7),
                selection: Color::from_rgb(0.3, 0.7, 0.5),
                border: Border::colored(2.0, Color::from_rgb(1.0, 1.0, 1.0), 1.0),
            },
            tab: Color::from_rgb(0.2, 0.2, 0.2),
            tab_hovered: Color::from_rgb(0.25, 0.25, 0.25),
            tab_active: Color::from_rgb(0.35, 0.35, 0.35),
            sound_bg: Color::from_rgb(0.2, 0.6, 0.4),
            sound_bg_hovered: Color::from_rgb(0.3, 0.7, 0.5),
            overlay_bg: Color::from_rgb(0.2, 0.2, 0.2),
            overlay_border: Border::colored(4.0, Color::from_rgb(1.0, 1.0, 1.0), 1.0),
            player_bar_background: Color::from_rgb(0.1, 0.1, 0.1),
            randomly_triggered_badge_bg: Color::from_rgb(0.8, 0.1, 0.1),
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

            fn default<'a>() -> Self::Class<'a> {
                Box::new($fn)
            }

            fn style(&self, class: &Self::Class<'_>, status: $mod_name::Status) -> $mod_name::Style {
                class(self, status)
            }
        }
        )*
    };
}

impl_catalog! {
    text => text_default,
    container => container_transparent,
    progress_bar => progress_bar_default,
}

impl_catalog_with_status! {
    button => button_sound,
    scrollable => scrollable_default,
    svg => svg_default,
    text_input => text_input_default,
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
        background: theme.player_bar_background.into(),
        bar: theme.sound_bg.into(),
        border: Border::new(2.0).into(),
    }
}

pub fn svg_default(theme: &Theme, _status: svg::Status) -> svg::Style {
    svg::Style {
        color: theme.icon.into(),
    }
}

pub fn text_default(theme: &Theme) -> text::Style {
    text::Style {
        color: theme.text.into(),
    }
}

pub fn container_transparent(_theme: &Theme) -> container::Style {
    container::Style::default()
}

pub fn container_opaque(theme: &Theme) -> container::Style {
    container::Style {
        background: theme.sound_bg.into(),
        border: Border::new(2.0).into(),
        ..Default::default()
    }
}

pub fn container_badge(theme: &Theme) -> container::Style {
    container::Style {
        background: theme.randomly_triggered_badge_bg.into(),
        border: Border::new(4.0).into(),
        ..Default::default()
    }
}

pub fn container_overlay(theme: &Theme) -> container::Style {
    container::Style {
        background: theme.overlay_bg.into(),
        text_color: theme.text.into(),
        border: theme.overlay_border.into(),
        ..Default::default()
    }
}

pub fn button_sound(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        text_color: theme.text.into(),
        background: match status {
            button::Status::Hovered => theme.sound_bg_hovered.into(),
            _ => theme.sound_bg.into(),
        },
        border: Border::new(2.0).into(),
        ..Default::default()
    }
}

pub fn scrollable_default(theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    let rail = scrollable::Rail {
        background: None,
        border: Border::none().into(),
        scroller: scrollable::Scroller {
            background: theme.text.into(),
            border: Border::new(2.0).into(),
        },
    };

    scrollable::Style {
        container: container_transparent(theme),
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
