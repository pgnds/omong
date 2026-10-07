mod app;
mod db;
mod event;
mod handlers;
mod storage;
mod ui;

use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};

use app::{App, Screen};

fn main() -> io::Result<()> {
    // terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // app
    let mut app = App::new();

    // main loop
    let result = event::run(&mut terminal, &mut app);

    // terminal teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let main_area = chunks[0];
    let status_area = chunks[1];

    // main content placeholder
    let _main_block = Block::default().borders(Borders::ALL);

    match app.screen {
        Screen::Connections => ui::connections::render(frame, main_area, app),
        Screen::Databases { .. } => ui::databases::render(frame, main_area, app),
        Screen::Collections { .. } => ui::collections::render(frame, main_area, app),
        Screen::Documents { .. } => ui::documents::render(frame, main_area, app),
    }

    ui::status_bar::render(frame, status_area, app);

    if app.overlay.is_some() {
        ui::popup::render(frame, app);
    }
}
