use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::app::{App, Overlay, PopupField};

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0]);

    horizontal[0]
}

pub fn render(frame: &mut Frame, app: &App) {
    match &app.overlay {
        Some(Overlay::AddConnectionPopup { .. }) => render_add_connection(frame, app),
        Some(Overlay::Confirm { message, .. }) => render_confirm(frame, message),
        Some(Overlay::FilterPrompt { .. }) | None => {}
    }
}

fn render_add_connection(frame: &mut Frame, app: &App) {
    let Some(Overlay::AddConnectionPopup {
        alias_buffer,
        uri_buffer,
        focus,
        editing_alias,
    }) = &app.overlay
    else {
        return;
    };

    let popup_area = centered_rect(60, 10, frame.area());

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(if editing_alias.is_some() {
            "edit connection"
        } else {
            "new connection"
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let alias_style = if matches!(focus, PopupField::Alias) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let uri_style = if matches!(focus, PopupField::Uri) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let alias_line = Paragraph::new(format!("alias: {}_", alias_buffer)).style(alias_style);
    let uri_line = Paragraph::new(format!("uri: {}_", uri_buffer)).style(uri_style);

    frame.render_widget(alias_line, fields[0]);
    frame.render_widget(uri_line, fields[1]);
}

fn render_confirm(frame: &mut Frame, message: &str) {
    let area = centered_rect(50, 5, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title("Confirm")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Red))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = Text::from(vec![
        Line::from(message.to_string()),
        Line::from(""),
        Line::from(Span::styled(
            "y: yes    n / Esc: no",
            Style::default().fg(Color::Yellow),
        )),
    ]);

    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}
