use crate::app::{
    App,
    config::filter::{FilterProperty, FilterType, PropVal},
    gui::{
        Message,
        view::{Element, theme},
    },
};
use iced::widget::{column, row, slider, space, text};

macro_rules! create_change_message {
    ($ctx:expr, $type:ident) => {
        move |v| {
            let mut prop = $ctx.prop;
            prop.val = PropVal::$type(v);
            Message::ChangeFilterProperty($ctx.filter_idx, prop)
        }
    };
}

impl App {
    pub(super) fn filter_properties<'a>(&'a self, i: usize, filter: &'a FilterType) -> Element<'a> {
        column(filter.properties().iter().map(|prop| {
            let ctx = Ctx {
                filter_idx: i,
                prop: *prop,
            };

            let name = text(prop.name())
                .style(theme::text_filter_property_name)
                .size(14);

            let changer = match prop.val {
                PropVal::F32(val) => self.f32_slider(ctx, val),
                PropVal::I32(val) => self.i32_slider(ctx, val),
            };

            let value_text = text(prop.fmt())
                .style(theme::text_filter_property_value)
                .size(14);

            column([
                row([name.into(), space::horizontal().into(), value_text.into()]).into(),
                changer,
            ])
            .into()
        }))
        .spacing(4)
        .into()
    }

    fn f32_slider(&self, ctx: Ctx, val: f32) -> Element<'_> {
        slider(ctx.prop.range_f32(), val, create_change_message!(ctx, F32))
            .step(ctx.prop.step_f32())
            .into()
    }

    fn i32_slider(&self, ctx: Ctx, val: i32) -> Element<'_> {
        slider(0..=1000, val, create_change_message!(ctx, I32)).into()
    }
}

struct Ctx {
    filter_idx: usize,
    prop: FilterProperty,
}
