use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::app::{App, Screen};
use crate::tree;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
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
    let items: Vec<ListItem> = lines
        .iter()
        .map(|l| {
            let marker = match (l.is_container, l.expanded) {
                (true, true) => "▾ ",
                (true, false) => "▸ ",
                (false, _) => "  ",
            };
            let prefix = format!("{}{}", "  ".repeat(l.depth), marker);

            let mut spans = vec![
                Span::raw(prefix),
                Span::styled(l.label.clone(), Style::default().fg(Color::Cyan)),
                Span::raw(":"),
            ];
            if !(l.is_container && l.expanded) {
                spans.push(Span::raw(format!(" {}", l.summary)));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Document (←/→ collapse/expand, +/- all, e edit, Esc back)")
                .borders(Borders::ALL)
                .border_type(BorderType::Thick),
        )
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

    frame.render_stateful_widget(list, area, list_state);
}
