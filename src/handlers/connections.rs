use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use super::{select_next, select_prev};
use crate::app::{App, Overlay, PopupField, Screen};
use crate::db;
use crate::storage::delete_connection;

pub fn handle(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.overlay = Some(Overlay::AddConnectionPopup {
                alias_buffer: String::new(),
                uri_buffer: String::new(),
                focus: PopupField::Alias,
                editing_alias: None,
            });
        }
        KeyCode::Down => select_next(&mut app.connections_list_state, app.saved_connections.len()),
        KeyCode::Up => select_prev(&mut app.connections_list_state, app.saved_connections.len()),
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if let Some(i) = app.connections_list_state.selected() {
                let (alias, uri) = app.saved_connections[i].clone();
                app.overlay = Some(Overlay::AddConnectionPopup {
                    alias_buffer: alias.clone(),
                    uri_buffer: uri,
                    focus: PopupField::Alias,
                    editing_alias: Some(alias),
                });
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let Some(i) = app.connections_list_state.selected() {
                let alias = app.saved_connections[i].0.clone();
                match delete_connection(&app.saved_connections, &alias) {
                    Ok(connections) => {
                        app.saved_connections = connections;
                        let len = app.saved_connections.len();
                        if len == 0 {
                            app.connections_list_state.select(None);
                        } else {
                            app.connections_list_state.select(Some(len - 1));
                        }
                    }
                    Err(e) => app.error_message = Some(format!("failed to deleted: {}", e)),
                }
            }
        }
        KeyCode::Enter => {
            if let Some(i) = app.connections_list_state.selected() {
                let uri = app.saved_connections[i].1.clone();
                match db::connect(&uri) {
                    Ok(client) => {
                        let names = db::list_databases(&client);
                        app.client = Some(client);
                        app.error_message = None;
                        app.screen = Screen::Databases {
                            list_state: ListState::default(),
                            names,
                        }
                    }
                    Err(e) => app.error_message = Some(e),
                }
            }
        }
        _ => {}
    }
}
