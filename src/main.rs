use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use mongodb::bson::doc;

const PAGE_SIZE: u64 = 20;

enum PopupField {
    Alias,
    Uri,
}

enum Screen {
    Connections,
    Databases {
        list_state: ListState,
        names: Vec<String>,
    },
    Collections {
        list_state: ListState,
        names: Vec<String>,
    },
    Documents {
        list_state: ListState,
        docs: Vec<mongodb::bson::Document>,
        page: u64,
    },
}

enum Overlay {
    AddConnectionPopup {
        alias_buffer: String,
        uri_buffer: String,
        focus: PopupField,
        editing_alias: Option<String>,
    },
}

pub struct App {
    screen: Screen,
    overlay: Option<Overlay>,

    client: Option<mongodb::sync::Client>,
    error_message: Option<String>,

    saved_connections: Vec<(String, String)>,
    connections_list_state: ListState,

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
        screen: Screen::Connections,
        overlay: None,
        client: None,
        error_message: None,
        saved_connections: load_connections(),
        connections_list_state: ListState::default(),
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

            if let Some(Overlay::AddConnectionPopup {
                alias_buffer,
                uri_buffer,
                focus,
                editing_alias,
            }) = &mut app.overlay
            {
                match key.code {
                    KeyCode::Esc => {
                        app.overlay = None;
                    }
                    KeyCode::Tab => {
                        *focus = match focus {
                            PopupField::Alias => PopupField::Uri,
                            PopupField::Uri => PopupField::Alias,
                        }
                    }
                    KeyCode::Backspace => match focus {
                        PopupField::Alias => {
                            alias_buffer.pop();
                        }
                        PopupField::Uri => {
                            uri_buffer.pop();
                        }
                    },
                    KeyCode::Char(c) => match focus {
                        PopupField::Alias => {
                            alias_buffer.push(c);
                        }
                        PopupField::Uri => {
                            uri_buffer.push(c);
                        }
                    },
                    KeyCode::Enter => {
                        if alias_buffer.is_empty() || uri_buffer.is_empty() {
                            app.error_message = Some("alias and uri are both required".to_string());
                        } else if let Some(old_alias) = editing_alias {
                            match update_connection(
                                &app.saved_connections,
                                old_alias,
                                alias_buffer,
                                uri_buffer.trim(),
                            ) {
                                Ok(connections) => {
                                    app.saved_connections = connections;
                                    app.error_message = None;
                                    app.overlay = None;
                                }
                                Err(e) => {
                                    app.error_message =
                                        Some(format!("failed to save connection: {}", e));
                                }
                            }
                        } else {
                            match save_connection(alias_buffer.trim(), uri_buffer.trim()) {
                                Ok(_) => {
                                    app.saved_connections = load_connections();
                                    app.overlay = None;
                                    app.error_message = None;
                                    app.overlay = None;
                                }
                                Err(e) => {
                                    app.error_message =
                                        Some(format!("failed to save connection: {}", e));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.overlay = None;
                    }
                    _ => match &mut app.screen {
                        Screen::Connections => match key.code {
                            KeyCode::Down => {
                                let len = app.saved_connections.len();
                                if len > 0 {
                                    let i = match app.connections_list_state.selected() {
                                        Some(i) => (i + 1) % len,
                                        None => 0,
                                    };
                                    app.connections_list_state.select(Some(i));
                                }
                            }
                            KeyCode::Up => {
                                let len = app.saved_connections.len();
                                if len > 0 {
                                    let i = match app.connections_list_state.selected() {
                                        Some(i) => (i + len - 1) % len,
                                        None => 0,
                                    };
                                    app.connections_list_state.select(Some(i));
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(i) = app.connections_list_state.selected() {
                                    let (alias, uri) = &app.saved_connections[i];
                                    match mongodb::sync::Client::with_uri_str(uri.trim()) {
                                        Ok(client) => {
                                            match client
                                                .database("admin")
                                                .run_command(doc! {"ping": 1})
                                                .run()
                                            {
                                                Ok(_) => {
                                                    let db_names = client
                                                        .list_database_names()
                                                        .run()
                                                        .unwrap_or_default();
                                                    app.client = Some(client);
                                                    app.connection_alias = Some(alias.to_string());
                                                    app.error_message = None;
                                                    app.screen = Screen::Databases {
                                                        list_state: ListState::default(),
                                                        names: db_names,
                                                    };
                                                }
                                                Err(e) => {
                                                    app.error_message =
                                                        Some(format!("ping failed: {}", e));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            app.error_message = Some(format!("invalid uri: {}", e));
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('d') => {
                                if let Some(i) = app.connections_list_state.selected() {
                                    let (alias, _) = &app.saved_connections[i];
                                    match delete_connection(&app.saved_connections, alias) {
                                        Ok(connections) => {
                                            app.saved_connections = connections;
                                            let len = app.saved_connections.len();
                                            if len == 0 {
                                                app.connections_list_state.select(None);
                                            } else if i >= len {
                                                app.connections_list_state.select(Some(len - 1))
                                            }
                                        }
                                        Err(e) => {
                                            app.error_message =
                                                Some(format!("failed to delete connection: {}", e));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        },
                        Screen::Databases { list_state, names } => match key.code {
                            KeyCode::Down => {
                                let len = names.len();
                                if len > 0 {
                                    let i = match list_state.selected() {
                                        Some(i) => (i + 1) % len,
                                        None => 0,
                                    };
                                    list_state.select(Some(i));
                                }
                            }
                            KeyCode::Up => {
                                let len = names.len();
                                if len > 0 {
                                    let i = match list_state.selected() {
                                        Some(i) => (i + len - 1) % len,
                                        None => 0,
                                    };
                                    list_state.select(Some(i));
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(i) = list_state.selected() {
                                    app.database = Some(names[i].clone());

                                    if let (Some(client), Some(database_name)) =
                                        (&app.client, &app.database)
                                    {
                                        let collection_names = client
                                            .database(database_name)
                                            .list_collection_names()
                                            .run()
                                            .unwrap_or_default();

                                        app.screen = Screen::Collections {
                                            list_state: ListState::default(),
                                            names: collection_names,
                                        }
                                    };
                                }
                            }
                            _ => {}
                        },
                        Screen::Collections { list_state, names } => match key.code {
                            KeyCode::Down => {
                                let len = names.len();
                                if len > 0 {
                                    let i = match list_state.selected() {
                                        Some(i) => (i + 1) % len,
                                        None => 0,
                                    };
                                    list_state.select(Some(i));
                                }
                            }
                            KeyCode::Up => {
                                let len = names.len();
                                if len > 0 {
                                    let i = match list_state.selected() {
                                        Some(i) => (i + len - 1) % len,
                                        None => 0,
                                    };
                                    list_state.select(Some(i));
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(i) = list_state.selected() {
                                    app.collection = Some(names[i].clone());
                                    let docs = fetch_documents(app, 0);
                                    app.screen = Screen::Documents {
                                        list_state: ListState::default(),
                                        docs,
                                        page: 0,
                                    }
                                }
                            }
                            _ => {}
                        },
                        Screen::Documents {
                            list_state,
                            docs,
                            page: _,
                        } => match key.code {
                            KeyCode::Down => {
                                let len = docs.len();
                                if len > 0 {
                                    let i = match list_state.selected() {
                                        Some(i) => (i + 1) % len,
                                        None => 0,
                                    };
                                    list_state.select(Some(i));
                                }
                            }
                            KeyCode::Up => {
                                let len = docs.len();
                                if len > 0 {
                                    let i = match list_state.selected() {
                                        Some(i) => (i + len - 1) % len,
                                        None => 0,
                                    };
                                    list_state.select(Some(i));
                                }
                            }
                            _ => {}
                        },
                    },
                }
            }
        }
    }
    Ok(())
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

fn delete_connection(
    connections: &[(String, String)],
    alias: &str,
) -> io::Result<Vec<(String, String)>> {
    let remaining: Vec<(String, String)> = connections
        .iter()
        .filter(|(a, _)| a != alias)
        .cloned()
        .collect();

    let path = connections_file_path();
    let mut content = String::new();
    for (a, u) in &remaining {
        content.push_str(&format!("{}|{}\n", a, u));
    }
    std::fs::write(path, content)?;

    Ok(remaining)
}

fn update_connection(
    connections: &[(String, String)],
    old_alias: &str,
    new_alias: &str,
    new_uri: &str,
) -> io::Result<Vec<(String, String)>> {
    let mut updated: Vec<(String, String)> = connections.to_vec();
    for (alias, uri) in updated.iter_mut() {
        if alias == old_alias {
            *alias = new_alias.to_string();
            *uri = new_uri.to_string();
            break;
        }
    }

    let path = connections_file_path();
    let mut content = String::new();
    for (alias, uri) in &updated {
        content.push_str(&format!("{}|{}\n", alias, uri));
    }
    std::fs::write(path, content)?;

    Ok(updated)
}

fn fetch_documents(app: &App, page: u64) -> Vec<mongodb::bson::Document> {
    let (Some(client), Some(db), Some(collection)) = (&app.client, &app.database, &app.collection)
    else {
        return Vec::new();
    };

    client
        .database(db)
        .collection::<mongodb::bson::Document>(collection)
        .find(doc! {})
        .skip(page * PAGE_SIZE)
        .limit(PAGE_SIZE as i64)
        .run()
        .map(|cursor| cursor.filter_map(Result::ok).collect())
        .unwrap_or_default()
}
