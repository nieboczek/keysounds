use iced::{Background, Border, Color, Shadow};

#[derive(Clone)]
pub struct Theme {
    bg: Color,
    text: Color,
    sound_bg: Color,
    hovered_sound_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::from_rgb(0.2, 0.2, 0.2),
            text: Color::from_rgb(0.95, 0.95, 0.95),
            sound_bg: Color::from_rgb(0.2, 0.6, 0.4),
            hovered_sound_bg: Color::from_rgb(0.3, 0.7, 0.5),
        }
    }
}

impl iced::theme::Base for Theme {
    fn base(&self) -> iced::theme::Style {
        iced::theme::Style {
            background_color: self.bg,
            text_color: self.text,
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
    text_input::Style {
        background: Background::Color(theme.bg),
        border: Border::default().rounded(2).color(Color::WHITE).width(1.0),
        icon: Color::BLACK,
        placeholder: theme.text,
        value: theme.text,
        selection: Color::WHITE,
    }
}

pub fn progress_bar_default(theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(Color::BLACK),
        bar: Background::Color(theme.sound_bg),
        border: Border::default().rounded(2),
    }
}

pub fn svg_default(_theme: &Theme, _status: svg::Status) -> svg::Style {
    svg::Style {
        color: Some(Color::WHITE),
    }
}

pub fn text_default(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.text),
    }
}

pub fn container_transparent(_theme: &Theme) -> container::Style {
    container::Style::default()
}

pub fn container_opaque(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme.sound_bg)),
        border: Border::default().rounded(2),
        ..Default::default()
    }
}

pub fn button_sound(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        text_color: theme.text,
        background: Some(Background::Color(match status {
            button::Status::Hovered => theme.hovered_sound_bg,
            _ => theme.sound_bg,
        })),
        border: Border::default().rounded(2),
        ..Default::default()
    }
}

pub fn scrollable_default(theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(theme.text),
            border: Border::default().rounded(2),
        },
    };

    scrollable::Style {
        container: container_transparent(theme),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(Color::WHITE),
            border: Border::default(),
            shadow: Shadow::default(),
            icon: Color::BLACK,
        },
    }
}
