use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use super::{id_label, select_next, select_prev};
use crate::app::{App, EditRequest, Screen, StatusMessage};
use crate::tree::{self, TreeLine};

use mongodb::bson::Document;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::{db, editor};

pub fn handle(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        let current = std::mem::replace(&mut app.screen, Screen::Connections);
        match current {
            Screen::Detail { back, .. } => app.screen = *back,
            other => app.screen = other,
        }
        return;
    }

    let Screen::Detail {
        doc,
        expanded,
        list_state,
        ..
    } = &mut app.screen
    else {
        return;
    };

    let lines = tree::flatten(doc, expanded);
    let current = list_state
        .selected()
        .and_then(|i| lines.get(i))
        .map(|l| (l.path.clone(), l.is_container, l.expanded));

    let mut changed = false;
    match key.code {
        KeyCode::Down => select_next(list_state, lines.len()),
        KeyCode::Up => select_prev(list_state, lines.len()),
        KeyCode::Enter => {
            if let Some((path, true, open)) = &current {
                if *open {
                    expanded.remove(path);
                } else {
                    expanded.insert(path.clone());
                }
                changed = true;
            }
        }
        KeyCode::Right => {
            if let Some((path, true, false)) = &current {
                expanded.insert(path.clone());
                changed = true;
            }
        }
        KeyCode::Left => {
            if let Some((path, true, true)) = &current {
                expanded.remove(path);
                changed = true;
            }
        }
        KeyCode::Char('+') => {
            *expanded = tree::all_container_paths(doc);
            changed = true;
        }
        KeyCode::Char('-') => {
            expanded.clear();
            changed = true;
        }
        KeyCode::Char('e') => app.edit_requested = Some(EditRequest::Update { doc: doc.clone() }),
        _ => {}
    }

    if changed && let Some((path, _, _)) = current {
        let new_lines = tree::flatten(doc, expanded);
        reselect(&new_lines, list_state, path);
    }
}

// after the visible lines change, put the cursor back to the same field.
// if that field is now hidden, fall back to it's closest visible ancestor.
fn reselect(lines: &[TreeLine], list_state: &mut ListState, mut path: String) {
    loop {
        if let Some(pos) = lines.iter().position(|l| l.path == path) {
            list_state.select(Some(pos));
            return;
        }
        match path.rfind('.') {
            Some(p) => path.truncate(p),
            None => {
                list_state.select(if lines.is_empty() { None } else { Some(0) });
                return;
            }
        }
    }
}

pub fn edit(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    original: Document,
) {
    let (Some(client), Some(db_name), Some(coll_name)) = (
        app.client.clone(),
        app.database.clone(),
        app.collection.clone(),
    ) else {
        return;
    };

    let edited = match editor::edit(terminal, &original) {
        Ok(Some(doc)) => doc,
        Ok(None) => return,
        Err(e) => {
            app.message = Some(StatusMessage::Error(e));
            return;
        }
    };

    match db::replace_document(&client, &db_name, &coll_name, &original, edited.clone()) {
        Ok(()) => {
            app.message = Some(StatusMessage::Info(format!(
                "successfully updated document {}",
                original.get("_id").map(id_label).unwrap_or_default()
            )));
            if let Screen::Detail { doc, back, .. } = &mut app.screen {
                if let Screen::Documents {
                    docs, list_state, ..
                } = back.as_mut()
                    && let Some(i) = list_state.selected()
                {
                    docs[i] = edited.clone();
                }
                *doc = edited;
            }
        }
        Err(e) => app.message = Some(StatusMessage::Error(format!("update failed: {}", e))),
    }
}
