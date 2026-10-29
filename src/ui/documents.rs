use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::app::{App, Screen};
use crate::db::PAGE_SIZE;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    if let Screen::Documents {
        list_state,
        docs,
        page,
        count,
        ..
    } = &mut app.screen
    {
        let items: Vec<ListItem> = docs
            .iter()
            .map(|doc| ListItem::new(doc.to_string()))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(
                        "{} documents (page {}/{})",
                        count,
                        *page + 1,
                        count.div_ceil(PAGE_SIZE).max(1),
                    ))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
            )
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

        frame.render_stateful_widget(list, area, list_state);
    }
}
