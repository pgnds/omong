use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::app::{App, Overlay, Screen};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let bar_style = Style::default().bg(Color::Blue).fg(Color::White);

    let indicators = format!(
        " conn: {} | db: {} | coll: {} ",
        app.connection_alias.as_deref().unwrap_or("-"),
        app.database.as_deref().unwrap_or("-"),
        app.collection.as_deref().unwrap_or("-"),
    );
    let right_width = indicators.chars().count() as u16;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(right_width)])
        .split(area);

    frame.render_widget(Block::default().style(bar_style), area);
    frame.render_widget(Paragraph::new(left_line(app)), chunks[0]);
    frame.render_widget(
        Paragraph::new(indicators).alignment(Alignment::Right),
        chunks[1],
    );
}

fn left_line(app: &App) -> Line<'static> {
    let error_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    if let Some(Overlay::FilterPrompt { input }) = &app.overlay {
        let mut spans = vec![
            Span::styled("/", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}_", input)),
        ];
        if let Some(err) = &app.error_message {
            spans.push(Span::styled(format!("E: {}", err), error_style));
        }
        return Line::from(spans);
    }

    if let Some(err) = &app.error_message {
        return Line::from(Span::styled(format!("E: {}", err), error_style));
    }

    if let Screen::Documents { filter, .. } = &app.screen {
        if filter.text.is_empty() {
            return Line::from(Span::styled(
                " / filter",
                Style::default().add_modifier(Modifier::DIM),
            ));
        }
        return Line::from(format!("filter: {}", filter.text));
    }

    Line::default()
}
