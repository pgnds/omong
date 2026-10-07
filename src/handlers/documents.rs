use crossterm::event::{KeyCode, KeyEvent};

use super::{select_next, select_prev};
use crate::app::{App, Screen};
use crate::db;

pub fn handle(app: &mut App, key: KeyEvent) {
    let Screen::Documents {
        list_state,
        docs,
        page,
    } = &mut app.screen
    else {
        return;
    };

    match key.code {
        KeyCode::Down => select_next(list_state, docs.len()),
        KeyCode::Up => select_prev(list_state, docs.len()),
        KeyCode::Right => {
            let next = *page + 1;
            if let (Some(client), Some(db_name), Some(coll)) =
                (&app.client, &app.database, &app.collection)
            {
                let new_docs = db::fetch_documents(client, db_name, coll, next);
                if !new_docs.is_empty() {
                    *page = next;
                    *docs = new_docs;
                    list_state.select(Some(0));
                }
            }
        }
        KeyCode::Left if *page > 0 => {
            let prev = *page - 1;
            if let (Some(client), Some(db_name), Some(coll)) =
                (&app.client, &app.database, &app.collection)
            {
                *docs = db::fetch_documents(client, db_name, coll, prev);
                *page = prev;
                list_state.select(Some(0));
            }
        }
        _ => {}
    }
}
