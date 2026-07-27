use crate::app::{
    App,
    gui::{Message, Theme},
};

pub mod app;

fn main() -> iced::Result {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    run_iced_app()
}

fn run_iced_app() -> iced::Result {
    iced::application::<App, Message, Theme, iced::Renderer>(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title("keysounds")
        .window_size(iced::Size::new(420.0, 600.0))
        .theme(|app: &App| Some(app.theme()))
        .run()
}
