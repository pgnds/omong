use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use super::{select_next, select_prev};
use crate::app::{App, Screen};
use crate::db;

pub fn handle(app: &mut App, key: KeyEvent) {
    let Screen::Collections { list_state, names } = &mut app.screen else {
        return;
    };

    match key.code {
        KeyCode::Down => select_next(list_state, names.len()),
        KeyCode::Up => select_prev(list_state, names.len()),
        KeyCode::Enter => {
            let Some(i) = list_state.selected() else {
                return;
            };
            let collection_name = names[i].clone();

            if let (Some(client), Some(db_name)) = (&app.client, &app.database) {
                let docs = db::fetch_documents(client, db_name, &collection_name, 0);
                app.screen = Screen::Documents {
                    list_state: ListState::default(),
                    docs,
                    page: 0,
                };
            }
            app.collection = Some(collection_name);
        }
        _ => {}
    }
}
