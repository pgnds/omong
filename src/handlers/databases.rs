use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use super::{select_next, select_prev};
use crate::app::{App, Screen};
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
        _ => {}
    }
}
