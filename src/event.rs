use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::app::{App, Screen};
use crate::handlers;
use crate::ui;

pub fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui(frame, app))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        };

        if app.overlay.is_some() {
            handlers::overlay::handle(app, key);
            continue;
        }

        if key.code == KeyCode::Char('q') {
            break;
        }

        match app.screen {
            Screen::Connections => handlers::connections::handle(app, key),
            Screen::Databases { .. } => handlers::databases::handle(app, key),
            Screen::Collections { .. } => handlers::collections::handle(app, key),
            Screen::Documents { .. } => handlers::documents::handle(app, key),
            Screen::Detail { .. } => handlers::detail::handle(app, key),
        }

        if app.edit_requested {
            app.edit_requested = false;
            handlers::detail::edit(terminal, app);
        }
    }
    Ok(())
}
