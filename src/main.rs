pub use app::App;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    text::Line,
};
use std::io;

mod app;
mod hotkey_handler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app: App = App;

    hotkey_handler::start(&mut app);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut selected = 0;
    let selectable_indices = [1, 3, 5];

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(frame.area());

            let lines: Vec<Line> = (0..7)
                .map(|i| {
                    let is_selected = Some(i) == selectable_indices.get(selected).copied();
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    let prefix = if is_selected { "*" } else { " " };
                    Line::from(vec![
                        Span::styled(prefix.to_string(), style),
                        Span::raw(format!(" Line {}", i)),
                    ])
                })
                .collect();

            let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));
            frame.render_widget(paragraph, chunks[0]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        if selected + 1 < selectable_indices.len() {
                            selected += 1;
                        }
                    }
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
