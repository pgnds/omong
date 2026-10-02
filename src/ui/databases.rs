use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::app::{App, Screen};

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    if let Screen::Databases { list_state, names } = &mut app.screen {
        let items: Vec<ListItem> = names.iter().map(|db| ListItem::new(db.as_str())).collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("databases")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
            )
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

        frame.render_stateful_widget(list, area, list_state);
    }
}
