use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use super::{select_next, select_prev};
use crate::app::{App, ConfirmAction, Overlay, PromptAction, Screen};
use crate::db;

pub fn handle(app: &mut App, key: KeyEvent) {
    let Screen::Databases { list_state, names } = &mut app.screen else {
        return;
    };

    match key.code {
        KeyCode::Down => select_next(list_state, names.len()),
        KeyCode::Up => select_prev(list_state, names.len()),
        KeyCode::Enter => {
            let Some(i) = list_state.selected() else {
                return;
            };
            let db_name = names[i].clone();

            if let Some(client) = &app.client {
                let collection_names = db::list_collections(client, &db_name);
                app.screen = Screen::Collections {
                    list_state: ListState::default(),
                    names: collection_names,
                };
            }
            app.database = Some(db_name);
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.overlay = Some(Overlay::Prompt {
                title: "New database".to_string(),
                action: PromptAction::NewDatabase,
                inputs: vec![("name", String::new()), ("collection name", String::new())],
                focus_index: 0,
            });
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            let Some(name) = list_state.selected().and_then(|i| names.get(i).cloned()) else {
                return;
            };

            app.overlay = Some(Overlay::Confirm {
                message: format!("drop database '{}'?", name),
                action: ConfirmAction::DropDatabase { name },
                yes_selected: false,
            });
        }
        KeyCode::Esc => {
            app.client = None;
            app.database = None;
            app.collection = None;
            app.message = None;
            app.connection_alias = None;
            app.screen = Screen::Connections;
        }
        _ => {}
    }
}
