use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .title("connections")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick);

    if app.saved_connections.is_empty() {
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let hint = Paragraph::new("Ctrl+A to add a new connection").alignment(Alignment::Center);

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(inner);

        frame.render_widget(hint, vertical[1]);
        return;
    }

    let items: Vec<ListItem> = app
        .saved_connections
        .iter()
        .map(|(alias, _uri)| ListItem::new(alias.as_str()))
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

    frame.render_stateful_widget(list, area, &mut app.connections_list_state);
}
