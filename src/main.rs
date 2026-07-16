use crate::app::App;

pub mod app;
pub mod hotkey_handler;

fn main() -> iced::Result {
    env_logger::init();

    iced::application(
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
    .centered()
    .run()
}
