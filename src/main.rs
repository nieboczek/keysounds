use crate::app::{
    App,
    gui::{Message, Theme},
};

pub mod app;
pub mod hotkey_handler;

fn main() -> iced::Result {
    env_logger::init();

    iced::application::<App, Message, Theme, iced::Renderer>(
        || {
            let channel = hotkey_handler::start();
            (App::new(channel), iced::Task::none())
        },
        App::update,
        App::view,
    )
    .subscription(App::subscription)
    .title("keysounds")
    .window_size(iced::Size::new(420.0, 600.0))
    .theme(|app: &App| Some(app.theme()))
    .run()
}
