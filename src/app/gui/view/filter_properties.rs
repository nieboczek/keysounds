use crate::app::{
    App,
    config::filter::{FilterProperty, FilterType, PropVal},
    gui::{
        Message,
        view::{Element, theme},
    },
};
use iced::widget::{column, row, slider, space, text};

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

            let value_text = match prop.val {
                PropVal::F32(val) => text(val)
                    .style(theme::text_filter_property_value)
                    .size(14)
                    .into(),
                PropVal::I32(val) => text(val)
                    .style(theme::text_filter_property_value)
                    .size(14)
                    .into(),
            };

            column([
                row([name.into(), space::horizontal().into(), value_text]).into(),
                changer,
            ])
            .into()
        }))
        .spacing(4)
        .into()
    }

    fn f32_slider(&self, ctx: Ctx, val: f32) -> Element<'_> {
        slider(ctx.prop.range_f32(), val, move |v| {
            ctx.create_change_message(PropVal::F32(v))
        })
        .into()
    }

    fn i32_slider(&self, ctx: Ctx, val: i32) -> Element<'_> {
        slider(0..=1000, val, move |v| {
            ctx.create_change_message(PropVal::I32(v))
        })
        .into()
    }
}

struct Ctx {
    filter_idx: usize,
    prop: FilterProperty,
}

impl Ctx {
    fn create_change_message(&self, new_value: PropVal) -> Message {
        let mut prop = self.prop;
        prop.val = new_value;
        Message::ChangeFilterProperty(self.filter_idx, prop)
    }
}
