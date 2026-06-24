use std::io;

pub mod app;
pub mod hotkey_handler;

fn main() -> io::Result<()> {
    let action_channel = hotkey_handler::start();
    let mut terminal = ratatui::try_init()?;
    let mut app = app::App::new(action_channel);

    app.run(&mut terminal)?;

    let _ = terminal.show_cursor();
    ratatui::try_restore()
}
