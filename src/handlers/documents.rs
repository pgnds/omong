use super::{select_next, select_prev};

use crate::app::{App, Filter, Overlay, Screen};
use crate::db;

use mongodb::bson::Document;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use std::collections::HashSet;

pub fn handle(app: &mut App, key: KeyEvent) {
    let Screen::Documents {
        list_state,
        docs,
        page,
        filter,
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
                let new_docs = db::fetch_documents(client, db_name, coll, &filter.doc, next)
                    .unwrap_or_default();
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
                *docs = db::fetch_documents(client, db_name, coll, &filter.doc, prev)
                    .unwrap_or_default();
                *page = prev;
                list_state.select(Some(0));
            }
        }
        KeyCode::Esc if !filter.text.is_empty() => {
            if let (Some(client), Some(db_name), Some(coll)) =
                (&app.client, &app.database, &app.collection)
            {
                match db::fetch_documents(client, db_name, coll, &Document::new(), 0) {
                    Ok(new_docs) => {
                        *docs = new_docs;
                        *filter = Filter::default();
                        *page = 0;
                        list_state.select(if docs.is_empty() { None } else { Some(0) });
                    }
                    Err(e) => app.error_message = Some(e),
                }
            }
        }
        KeyCode::Esc => {
            let (Some(client), Some(db_name)) = (&app.client, &app.database) else {
                return;
            };
            let coll_names = db::list_collections(client, db_name);

            let mut state = ListState::default();
            if let Some(pos) = app
                .collection
                .as_ref()
                .and_then(|c| coll_names.iter().position(|n| n == c))
            {
                state.select(Some(pos));
            }

            app.screen = Screen::Collections {
                list_state: state,
                names: coll_names,
            };
            app.collection = None;
        }
        KeyCode::Enter => {
            let Some(doc) = list_state.selected().and_then(|i| docs.get(i)).cloned() else {
                return;
            };

            let previous = std::mem::replace(&mut app.screen, Screen::Connections);
            app.screen = Screen::Detail {
                doc,
                expanded: HashSet::new(),
                list_state: ListState::default().with_selected(Some(0)),
                back: Box::new(previous),
            };
        }
        KeyCode::Char('/') => {
            app.overlay = Some(Overlay::FilterPrompt {
                input: filter.text.clone(),
            });
        }
        _ => {}
    }
}
