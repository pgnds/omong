use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, Filter, Overlay, PopupField, Screen};
use crate::db;
use crate::storage::{load_connections, save_connection, update_connection};

pub fn handle(app: &mut App, key: KeyEvent) {
    if matches!(app.overlay, Some(Overlay::FilterPrompt { .. })) {
        handle_filter(app, key);
    } else {
        handle_add_connection(app, key);
    }
}

fn handle_add_connection(app: &mut App, key: KeyEvent) {
    let Some(Overlay::AddConnectionPopup {
        alias_buffer,
        uri_buffer,
        focus,
        editing_alias,
    }) = &mut app.overlay
    else {
        return;
    };

    match key.code {
        KeyCode::Esc => app.overlay = None,
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
                        app.error_message = Some(format!("failed to save connection: {}", e));
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
                        app.error_message = Some(format!("failed to save connection: {}", e));
                    }
                }
            }
        }
        _ => {}
    }
}

fn handle_filter(app: &mut App, key: KeyEvent) {
    let Some(Overlay::FilterPrompt { input }) = &mut app.overlay else {
        return;
    };

    match key.code {
        KeyCode::Esc => {
            app.overlay = None;
            app.error_message = None;
        }
        KeyCode::Backspace => {
            input.pop();
        }
        KeyCode::Char(c) => input.push(c),
        KeyCode::Enter => {
            let text = input.trim().to_string();
            apply_filter(app, text);
        }
        _ => {}
    }
}

fn apply_filter(app: &mut App, text: String) {
    let filter_doc = match db::parse_filter(&text) {
        Ok(d) => d,
        Err(e) => {
            app.error_message = Some(format!("invalid filter: {}", e));
            return;
        }
    };

    let (Some(client), Some(db_name), Some(coll)) = (&app.client, &app.database, &app.collection)
    else {
        return;
    };

    let new_docs = match db::fetch_documents(client, db_name, coll, &filter_doc, 0) {
        Ok(d) => d,
        Err(e) => {
            app.error_message = Some(format!("query failed: {}", e));
            return;
        }
    };

    if let Screen::Documents {
        docs,
        list_state,
        page,
        filter,
    } = &mut app.screen
    {
        *docs = new_docs;
        *page = 0;
        *filter = Filter {
            text,
            doc: filter_doc,
        };
        list_state.select(if docs.is_empty() { None } else { Some(0) });
    }
    app.overlay = None;
    app.error_message = None;
}
