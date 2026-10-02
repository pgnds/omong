use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
};

use crate::app::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut status_text = format!(
        " conn: {} | db: {} | coll: {} ",
        app.connection_alias.as_deref().unwrap_or("-"),
        app.database.as_deref().unwrap_or("-"),
        app.collection.as_deref().unwrap_or("-"),
    );

    if let Some(err) = &app.error_message {
        status_text.push_str(&format!(" | ERROR: {} ", err));
    }

    let status_bar = Paragraph::new(Line::from(status_text))
        .style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_widget(status_bar, area);
}
