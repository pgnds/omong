use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::ListState};

use crate::app::{App, Overlay, PopupField, Screen};
use crate::db;
use crate::storage::{delete_connection, load_connections, save_connection, update_connection};
use crate::ui;

pub fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
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
                                    match db::connect(&uri) {
                                        Ok(client) => {
                                            let db_names = db::list_databases(&client);
                                            app.client = Some(client);
                                            app.connection_alias = Some(alias.to_string());
                                            app.error_message = None;
                                            app.screen = Screen::Databases {
                                                list_state: ListState::default(),
                                                names: db_names,
                                            }
                                        }
                                        Err(e) => app.error_message = Some(e),
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
                            KeyCode::Char('e') => {
                                if let Some(i) = app.connections_list_state.selected() {
                                    let (alias, uri) = &app.saved_connections[i];
                                    app.overlay = Some(Overlay::AddConnectionPopup {
                                        alias_buffer: alias.clone(),
                                        uri_buffer: uri.clone(),
                                        focus: PopupField::Alias,
                                        editing_alias: Some(alias.clone()),
                                    });
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
                                    let db_name = names[i].clone();

                                    if let Some(client) = &app.client {
                                        let collection_names =
                                            db::list_collections(client, &db_name);
                                        app.screen = Screen::Collections {
                                            list_state: ListState::default(),
                                            names: collection_names,
                                        }
                                    };

                                    app.database = Some(db_name);
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
                                    let coll_name = names[i].clone();
                                    if let (Some(client), Some(db_name)) =
                                        (&app.client, &app.database)
                                    {
                                        let docs =
                                            db::fetch_documents(client, db_name, &coll_name, 0);

                                        app.collection = Some(coll_name);
                                        app.screen = Screen::Documents {
                                            list_state: ListState::default(),
                                            docs,
                                            page: 0,
                                        }
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
