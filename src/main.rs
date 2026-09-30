mod app;
mod db;
mod event;
mod storage;

use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
};

use app::{App, Overlay, PopupField, Screen};

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
        Screen::Connections => render_connections_box(frame, main_area, app),
        Screen::Databases { .. } => render_database_list(frame, main_area, app),
        Screen::Collections { .. } => render_collection_list(frame, main_area, app),
        Screen::Documents { .. } => render_documents_list(frame, main_area, app),
    }

    render_status_bar(frame, status_area, app);

    if app.overlay.is_some() {
        render_app_popup(frame, app);
    }
}

fn render_connections_box(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .title("connections")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick);

    if app.saved_connections.is_empty() {
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let hint = Paragraph::new("Ctrl+A to add a new connection").alignment(Alignment::Center);

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(inner);

        frame.render_widget(hint, vertical[1]);
        return;
    }

    let items: Vec<ListItem> = app
        .saved_connections
        .iter()
        .map(|(alias, _uri)| ListItem::new(alias.as_str()))
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

    frame.render_stateful_widget(list, area, &mut app.connections_list_state);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let mut status_text = format!(
        " conn: {} | db: {} | coll: {} ",
        app.connection_alias.as_deref().unwrap_or("-"),
        app.database.as_deref().unwrap_or("-"),
        app.collection.as_deref().unwrap_or("-"),
    );

    if let Some(err) = &app.error_message {
        status_text.push_str(&format!(" | ERROR: {} ", err));
    }

    let status_bar = Paragraph::new(Line::from(status_text))
        .style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_widget(status_bar, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0]);

    horizontal[0]
}

fn render_app_popup(frame: &mut Frame, app: &App) {
    let Some(Overlay::AddConnectionPopup {
        alias_buffer,
        uri_buffer,
        focus,
        editing_alias,
    }) = &app.overlay
    else {
        return;
    };

    let popup_area = centered_rect(60, 10, frame.area());

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(if editing_alias.is_some() {
            "edit connection"
        } else {
            "new connection"
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let alias_style = if matches!(focus, PopupField::Alias) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let uri_style = if matches!(focus, PopupField::Uri) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let alias_line = Paragraph::new(format!("alias: {}_", alias_buffer)).style(alias_style);
    let uri_line = Paragraph::new(format!("uri: {}_", uri_buffer)).style(uri_style);

    frame.render_widget(alias_line, fields[0]);
    frame.render_widget(uri_line, fields[1]);
}

fn render_database_list(frame: &mut Frame, area: Rect, app: &mut App) {
    if let Screen::Databases { list_state, names } = &mut app.screen {
        let items: Vec<ListItem> = names.iter().map(|db| ListItem::new(db.as_str())).collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("databases")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
            )
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

        frame.render_stateful_widget(list, area, list_state);
    }
}

fn render_collection_list(frame: &mut Frame, area: Rect, app: &mut App) {
    if let Screen::Collections { list_state, names } = &mut app.screen {
        let items: Vec<ListItem> = names
            .iter()
            .map(|coll| ListItem::new(coll.as_str()))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("collections")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
            )
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

        frame.render_stateful_widget(list, area, list_state);
    }
}

fn render_documents_list(frame: &mut Frame, area: Rect, app: &mut App) {
    if let Screen::Documents {
        list_state,
        docs,
        page,
    } = &mut app.screen
    {
        let items: Vec<ListItem> = docs
            .iter()
            .map(|doc| ListItem::new(doc.to_string()))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("documents (page {})", *page + 1))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
            )
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

        frame.render_stateful_widget(list, area, list_state);
    }
}
