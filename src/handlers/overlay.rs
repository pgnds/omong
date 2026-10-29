use crossterm::event::{KeyCode, KeyEvent};

use super::id_label;
use crate::app::{App, ConfirmAction, Filter, Overlay, PopupField, Screen, StatusMessage};
use crate::db;
use crate::secrets;
use crate::storage::{
    Connection, delete_connection, load_connections, save_connection, update_connection,
};

pub fn handle(app: &mut App, key: KeyEvent) {
    match app.overlay {
        Some(Overlay::FilterPrompt { .. }) => handle_filter(app, key),
        Some(Overlay::Confirm { .. }) => handle_confirm(app, key),
        Some(Overlay::AddConnectionPopup { .. }) => handle_add_connection(app, key),
        None => {}
    }
}

fn handle_add_connection(app: &mut App, key: KeyEvent) {
    let Some(Overlay::AddConnectionPopup {
        alias_buffer,
        uri_buffer,
        focus,
        editing_id,
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
                app.message = Some(StatusMessage::Error(
                    "alias and uri are both required".to_string(),
                ));
            } else if mongodb::options::ConnectionString::parse(uri_buffer.trim()).is_err() {
                app.message = Some(StatusMessage::Error("not valid mongo uri".to_string()));
            } else if let Some(id) = editing_id {
                if let Some(mut edited) = app
                    .saved_connections
                    .iter()
                    .find(|c| c.id() == *id)
                    .cloned()
                {
                    edited.alias = alias_buffer.trim().to_string();

                    match secrets::save_uri(edited.id(), uri_buffer.trim())
                        .map_err(|e| e.to_string())
                        .and_then(|_| {
                            update_connection(&app.saved_connections, &edited)
                                .map_err(|e| e.to_string())
                        }) {
                        Ok(connections) => {
                            app.saved_connections = connections;
                            app.message = None;
                            app.overlay = None;
                        }
                        Err(e) => {
                            app.message = Some(StatusMessage::Error(format!(
                                "failed to save connection: {}",
                                e
                            )));
                        }
                    }
                } else {
                    app.message = Some(StatusMessage::Error(
                        "failed to find connection".to_string(),
                    ));
                }
            } else {
                let connection = Connection::new(alias_buffer.trim().to_string());

                // try to store the uri in the keychain first
                match secrets::save_uri(connection.id(), uri_buffer.trim())
                    .map_err(|e| e.to_string())
                    .and_then(|_| save_connection(&connection).map_err(|e| e.to_string()))
                {
                    Ok(_) => {
                        app.saved_connections = load_connections();
                        app.overlay = None;
                        app.message = None;
                    }
                    Err(e) => {
                        app.message = Some(StatusMessage::Error(format!(
                            "failed to save connection: {}",
                            e
                        )));
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
            app.message = None;
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
            app.message = Some(StatusMessage::Error(format!("invalid filter: {}", e)));
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
            app.message = Some(StatusMessage::Error(format!("query failed: {}", e)));
            return;
        }
    };

    let new_count = db::count_documents(client, db_name, coll, &filter_doc).unwrap_or_default();

    if let Screen::Documents {
        docs,
        list_state,
        page,
        count,
        filter,
    } = &mut app.screen
    {
        *docs = new_docs;
        *page = 0;
        *count = new_count;
        *filter = Filter {
            text,
            doc: filter_doc,
        };
        list_state.select(if docs.is_empty() { None } else { Some(0) });
    }
    app.overlay = None;
    app.message = None;
}

fn handle_confirm(app: &mut App, key: KeyEvent) {
    let Some(Overlay::Confirm { yes_selected, .. }) = &mut app.overlay else {
        return;
    };

    match key.code {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
            *yes_selected = !*yes_selected
        }
        KeyCode::Enter => {
            if *yes_selected {
                confirm(app);
            } else {
                app.overlay = None;
            }
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => confirm(app),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.overlay = None,
        _ => {}
    }
}

fn confirm(app: &mut App) {
    if let Some(Overlay::Confirm { action, .. }) = app.overlay.take() {
        run_confirmed(app, action);
    }
}

fn run_confirmed(app: &mut App, action: ConfirmAction) {
    match action {
        ConfirmAction::DeleteConnection { id } => {
            match secrets::delete_uri(id)
                .map_err(|e| e.to_string())
                .and_then(|_| {
                    delete_connection(&app.saved_connections, id).map_err(|e| e.to_string())
                }) {
                Ok(connections) => {
                    app.saved_connections = connections;

                    let len = app.saved_connections.len();
                    if len == 0 {
                        app.connections_list_state.select(None);
                    } else if let Some(i) = app.connections_list_state.selected()
                        && i >= len
                    {
                        app.connections_list_state.select(Some(len - 1));
                    }
                }
                Err(e) => {
                    app.message = Some(StatusMessage::Error(format!(
                        "failed to delete connection: {}",
                        e
                    )))
                }
            }
        }
        ConfirmAction::DeleteDocument { id } => {
            let (Some(client), Some(db_name), Some(coll)) =
                (&app.client, &app.database, &app.collection)
            else {
                return;
            };

            if let Err(e) = db::delete_document(client, db_name, coll, &id) {
                app.message = Some(StatusMessage::Error(format!(
                    "failed to delete document: {}",
                    e
                )));
                return;
            }

            app.message = Some(StatusMessage::Info(format!(
                "successfully deleted document: {}",
                id_label(&id)
            )));

            let Screen::Documents {
                docs,
                list_state,
                page,
                count,
                filter,
            } = &mut app.screen
            else {
                return;
            };

            let mut new_page = *page;
            let mut result = db::fetch_documents(client, db_name, coll, &filter.doc, new_page);

            if matches!(&result, Ok(d) if d.is_empty()) && new_page > 0 {
                new_page -= 1;
                result = db::fetch_documents(client, db_name, coll, &filter.doc, new_page);
            }

            match result {
                Ok(new_docs) => {
                    let selected = list_state.selected().unwrap_or(0);
                    *docs = new_docs;
                    *page = new_page;
                    *count = count.saturating_sub(1);
                    if docs.is_empty() {
                        list_state.select(None);
                    } else {
                        list_state.select(Some(selected.min(docs.len() - 1)));
                    }
                }
                Err(e) => {
                    app.message = Some(StatusMessage::Error(format!(
                        "document reload failed: {}",
                        e
                    )))
                }
            }
        }
    }
}
