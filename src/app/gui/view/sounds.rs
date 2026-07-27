use crate::app::{
    App,
    gui::{Message, view::Element},
};
use iced::widget::{Column, Row, button, column, scrollable, text, text_input};

impl App {
    pub(super) fn sounds_page(&self) -> Element<'_> {
        let search = text_input("Search Sounds...", &self.search)
            .on_input(Message::SearchInput)
            .on_submit(Message::SearchSubmit);

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
