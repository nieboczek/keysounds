pub use app::App;
use ratatui::crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, Stdout},
    sync::mpsc,
};

mod app;
mod hotkey_handler;

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = mpsc::channel();
    hotkey_handler::start(sender);

    let mut app: App = App::new(receiver);
    let mut terminal = init_terminal()?;

    app.run(&mut terminal)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
