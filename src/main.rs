use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment, Flex, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, BorderType, Clear, List, ListItem, ListState},
    Frame, Terminal,
};

enum PopupField {
    Alias,
    Uri,
}

pub struct App {
    show_add_popup: bool,
    
    alias_buffer: String,
    uri_buffer: String,
    popup_focus: PopupField,
    
    client: Option<mongodb::sync::Client>,
    error_message: Option<String>,
    
    saved_connections: Vec<(String, String)>,
    connections_list_state: ListState,
    databases: Vec<String>,
    //db_list_state: ListState,

    connection_alias: Option<String>,
    database: Option<String>,
    collection: Option<String>,
}

fn main() -> io::Result<()> {
    // terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // app
    let mut app = App {
        show_add_popup: false,
        alias_buffer: String::new(),
        uri_buffer: String::new(),
        popup_focus: PopupField::Alias,
        client: None,
        error_message: None,
        saved_connections: load_connections(),
        connections_list_state: ListState::default(),
        databases: Vec::new(),
        //db_list_state: ListState::default(),
        connection_alias: None,
        database: None,
        collection: None,
    };

    // main loop
    let result = run(&mut terminal, &mut app);

    // terminal teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui(frame, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.show_add_popup {
                match key.code {
                    KeyCode::Esc => {
                        app.show_add_popup = false;
                        app.alias_buffer.clear();
                        app.uri_buffer.clear();
                        app.popup_focus = PopupField::Alias;
                    }
                    KeyCode::Tab => {
                        app.popup_focus = match app.popup_focus {
                            PopupField::Alias => PopupField::Uri,
                            PopupField::Uri => PopupField::Alias,
                        }
                    }
                    KeyCode::Backspace => {
                        match app.popup_focus {
                            PopupField::Alias => { app.alias_buffer.pop(); }
                            PopupField::Uri => { app.uri_buffer.pop(); }
                        }
                    }
                    KeyCode::Char(c) => {
                        match app.popup_focus {
                            PopupField::Alias => { app.alias_buffer.push(c); }
                            PopupField::Uri => { app.uri_buffer.push(c); }
                        }
                    }
                    KeyCode::Enter => {
                        if app.alias_buffer.is_empty() || app.uri_buffer.is_empty() {
                            app.error_message = Some("alias and uri are both required".to_string());
                        } else {
                            match save_connection(&app.alias_buffer.trim(), app.uri_buffer.trim()) {
                                Ok(_) => {
                                    app.saved_connections = load_connections();
                                    app.show_add_popup = false;
                                    app.error_message = None;
                                    app.alias_buffer.clear();
                                    app.uri_buffer.clear();
                                    app.popup_focus = PopupField::Alias;
                                }
                                Err(e) => {
                                    app.error_message = Some(format!("failed to save connection: {}", e));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                match key.code{
                    KeyCode::Char('q') => break,
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.show_add_popup = true;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let main_area = chunks[0];
    let status_area = chunks[1];

    // main content placeholder
    let _main_block = Block::default().borders(Borders::ALL);

    if app.client.is_some() {
        render_database_list(frame, main_area, app);
    } else {
        render_connections_box(frame, main_area);
    }

    render_status_bar(frame, status_area, app);
    
    if app.show_add_popup {
        render_app_popup(frame, app);
    }
}

fn render_connections_box(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("connections")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let hint = Paragraph::new("Ctrl+A to add a new connection")
        .alignment(Alignment::Center);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
        .split(inner);

    frame.render_widget(hint, vertical[1]);
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
    let popup_area = centered_rect(60, 10, frame.area());

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("new connection")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);
    
    let fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let alias_style = if matches!(app.popup_focus, PopupField::Alias) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let uri_style = if matches!(app.popup_focus, PopupField::Uri) {
        Style::default().fg(Color::Yellow) 
    } else {
        Style::default().fg(Color::White)
    };

    let alias_line = Paragraph::new(format!("alias: {}_", app.alias_buffer)).style(alias_style);
    let uri_line = Paragraph::new(format!("uri: {}_", app.uri_buffer)).style(uri_style);

    frame.render_widget(alias_line, fields[0]);
    frame.render_widget(uri_line, fields[1]);
}

fn render_database_list(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title("databases")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick);
    
    let items: Vec<ListItem> = app
        .databases
        .iter()
        .map(|db| ListItem::new(db.as_str()))
        .collect();

    let list = List::new(items).block(block);

    frame.render_widget(list, area);
}

fn connections_file_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    std::path::PathBuf::from(home).join(".config/omong/connections")
}

fn load_connections() -> Vec<(String, String)> {
    let path = connections_file_path();
    let content = std::fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter_map(|line| line.split_once('|'))
        .map(|(alias, uri)| (alias.to_string(), uri.to_string()))
        .collect()
}

fn save_connection(alias: &str, uri: &str) -> io::Result<()> {
    let path = connections_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    writeln!(file, "{}|{}", alias, uri)
}
