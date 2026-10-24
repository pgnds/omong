use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use super::{select_next, select_prev};
use crate::app::{App, ConfirmAction, Overlay, PopupField, Screen, StatusMessage};
use crate::db;
use crate::secrets;

pub fn handle(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.overlay = Some(Overlay::AddConnectionPopup {
                alias_buffer: String::new(),
                uri_buffer: String::new(),
                focus: PopupField::Alias,
                editing_id: None,
            });
        }
        KeyCode::Down => select_next(&mut app.connections_list_state, app.saved_connections.len()),
        KeyCode::Up => select_prev(&mut app.connections_list_state, app.saved_connections.len()),
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if let Some(i) = app.connections_list_state.selected() {
                let connection = app.saved_connections[i].clone();
                match secrets::get_uri(connection.id()) {
                    Ok(uri) => {
                        app.overlay = Some(Overlay::AddConnectionPopup {
                            alias_buffer: connection.alias.clone(),
                            uri_buffer: uri,
                            focus: PopupField::Alias,
                            editing_id: Some(connection.id()),
                        });
                    }
                    Err(e) => {
                        app.message = Some(StatusMessage::Error(format!(
                            "failed to read uri from keychain: {}",
                            e
                        )))
                    }
                }
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let Some(i) = app.connections_list_state.selected() {
                let connection = app.saved_connections[i].clone();
                app.overlay = Some(Overlay::Confirm {
                    message: format!("delete connection '{}'?", connection.alias),
                    action: ConfirmAction::DeleteConnection {
                        id: connection.id(),
                    },
                    yes_selected: false,
                });
            }
        }
        KeyCode::Enter => {
            if let Some(i) = app.connections_list_state.selected() {
                let connection = app.saved_connections[i].clone();
                match secrets::get_uri(connection.id())
                    .map_err(|e| e.to_string())
                    .and_then(|uri| db::connect(&uri))
                {
                    Ok(client) => {
                        let names = db::list_databases(&client);
                        app.client = Some(client);
                        app.connection_alias = Some(connection.alias);
                        app.message = None;
                        app.screen = Screen::Databases {
                            list_state: ListState::default(),
                            names,
                        }
                    }
                    Err(e) => app.message = Some(StatusMessage::Error(e)),
                }
            }
        }
        _ => {}
    }
}
