use super::{select_next, select_prev};

use crate::app::{App, ConfirmAction, EditRequest, Filter, Overlay, Screen};
use crate::{db, editor};

use mongodb::bson::{Bson, Document};
use ratatui::{Terminal, backend::CrosstermBackend};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use std::collections::HashSet;

use std::io;

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
        KeyCode::Char('d') | KeyCode::Char('D') => {
            let Some(id) = list_state
                .selected()
                .and_then(|i| docs.get(i))
                .and_then(|d| d.get("_id"))
                .cloned()
            else {
                return;
            };

            app.overlay = Some(Overlay::Confirm {
                message: format!("delete document '{}'?", id_label(&id)),
                action: ConfirmAction::DeleteDocument { id },
                yes_selected: false,
            });
        }
        KeyCode::Char('i') | KeyCode::Char('I') => {
            app.edit_requested = Some(EditRequest::Insert {});
        }
        _ => {}
    }
}

fn id_label(id: &Bson) -> String {
    match id {
        Bson::ObjectId(oid) => oid.to_hex(),
        Bson::String(s) => s.clone(),
        Bson::Binary(b) => {
            let hex: String = b.bytes.iter().map(|byte| format!("{:02x}", byte)).collect();
            format!("binary({:?}) {}", b.subtype, hex)
        }
        other => other.to_string(),
    }
}

pub fn insert(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) {
    let (
        Some(client),
        Some(db_name),
        Some(coll_name),
        Screen::Documents {
            list_state,
            docs,
            page,
            filter,
        },
    ) = (
        app.client.clone(),
        app.database.clone(),
        app.collection.clone(),
        &mut app.screen,
    )
    else {
        return;
    };

    let new_document = match editor::edit(terminal, &Document::default()) {
        Ok(Some(doc)) => doc,
        Ok(None) => return,
        Err(e) => {
            app.error_message = Some(e);
            return;
        }
    };

    match db::insert_document(&client, &db_name, &coll_name, &new_document) {
        Ok(..) => {
            app.error_message = None;
            match db::fetch_documents(&client, &db_name, &coll_name, &filter.doc, *page) {
                Ok(new_docs) => {
                    *docs = new_docs;
                    list_state.select(if docs.is_empty() { None } else { Some(0) });
                }
                Err(e) => app.error_message = Some(e),
            }
        }
        Err(e) => app.error_message = Some(format!("insertion failed: {}", e)),
    }
}
