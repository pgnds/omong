use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use mongodb::bson::Document;

use super::{select_next, select_prev};
use crate::app::{App, Filter, Screen};
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
                let docs =
                    db::fetch_documents(client, db_name, &collection_name, &Document::new(), 0)
                        .unwrap_or_default();
                app.screen = Screen::Documents {
                    list_state: ListState::default(),
                    docs,
                    page: 0,
                    filter: Filter::default(),
                };
            }
            app.collection = Some(collection_name);
        }
        KeyCode::Esc => {
            let Some(client) = &app.client else {
                return;
            };
            let db_names = db::list_databases(client);

            let mut state = ListState::default();
            if let Some(pos) = app
                .database
                .as_ref()
                .and_then(|d| db_names.iter().position(|n| n == d))
            {
                state.select(Some(pos));
            }

            app.screen = Screen::Databases {
                list_state: state,
                names: db_names,
            };
            app.database = None;
            app.collection = None;
        }
        _ => {}
    }
}
