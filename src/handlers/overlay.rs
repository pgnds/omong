use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, Overlay, PopupField};
use crate::storage::{load_connections, save_connection, update_connection};

pub fn handle(app: &mut App, key: KeyEvent) {
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
