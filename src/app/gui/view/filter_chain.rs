use crate::app::{
    App,
    gui::{Message, view::Element},
};
use iced::widget::{Column, Row, button, column, row, scrollable, text, text_input};

impl App {
    // TODO: actually apply the changes. this would require a refactor of the core,
    //       i don't want to touch that right now

    pub(super) fn filter_chain_page(&self) -> Element<'_> {
        let tabs = row(self
            .config
            .keybinds
            .iter()
            .filter_map(|k| match k.action {
                crate::app::Action::FilterPreset(v) => Some(&v),
                _ => None,
            })
            .map(|c| c));

        let sound_list: Element<'_> = if self.config.sounds.is_empty() {
            text("No sounds configured").into()
        } else {
            let mut content = Column::new().spacing(8);
            let mut current_row = Row::new().spacing(8);
            let mut count = 0;

            for (i, sound) in self.get_search_results() {
                let btn = button(text(sound.name.as_str()).size(14))
                    .width(128)
                    .height(128)
                    .on_press(Message::PlaySound(i));

                current_row = current_row.push(btn);
                count += 1;

                if count % 3 == 0 {
                    content = content.push(current_row);
                    current_row = Row::new().spacing(8);
                }
            }

            if count % 3 != 0 {
                content = content.push(current_row);
            }

            scrollable(content).into()
        };

        column([search.into(), sound_list]).into()
    }
}
