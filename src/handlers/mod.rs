pub mod collections;
pub mod connections;
pub mod databases;
pub mod documents;
pub mod overlay;

use ratatui::widgets::ListState;

pub fn select_next(state: &mut ListState, len: usize) {
    if len == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => (i + 1) % len,
        None => 0,
    };
    state.select(Some(i));
}

pub fn select_prev(state: &mut ListState, len: usize) {
    if len == 0 {
        return;
    }
    let i = match state.selected() {
        Some(i) => (i + len - 1) % len,
        None => 0,
    };
    state.select(Some(i));
}
