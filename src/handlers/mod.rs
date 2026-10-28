pub mod collections;
pub mod connections;
pub mod databases;
pub mod detail;
pub mod documents;
pub mod overlay;

use mongodb::bson::Bson;
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

pub fn id_label(id: &Bson) -> String {
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
